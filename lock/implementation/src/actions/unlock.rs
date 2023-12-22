use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action as Erc1155Action,
    error::ContractError as Erc1155ContractError,
    state::{Balance, BalancePrecision},
};

use warp_lock::{
    action::{ActionResult, HandlerResult, Unlock},
    error::ContractError,
    state::{Linear, LockedBalance, Parameters},
};

use crate::{
    actions::AsyncActionable,
    contract_utils::{
        foreign_call::ForeignContractCaller,
        js_imports::{Block, Contract},
    },
    state::State,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalWriteResult {
    #[serde(rename = "type")]
    result_type: String,
}

fn locked_balance_to_transfer(
    current_block: u32,
    lock_account: &str,
    balance: &LockedBalance,
    owner: &str,
) -> (Option<Erc1155Action::Action>, Option<LockedBalance>) {
    match balance {
        balance @ LockedBalance::Cliff(release) => {
            if release.at + release.duration <= current_block {
                (
                    Some(Erc1155Action::Action::Transfer(Erc1155Action::Transfer {
                        from: Some(lock_account.to_string()),
                        target: owner.to_string(),
                        token_id: Some(release.token_id.clone()),
                        qty: release.qty,
                    })),
                    None,
                )
            } else {
                (None, Some(balance.clone()))
            }
        }
        LockedBalance::Linear(balance) => {
            if balance.at + balance.duration <= current_block {
                (
                    Some(Erc1155Action::Action::Transfer(Erc1155Action::Transfer {
                        from: Some(lock_account.to_string()),
                        target: owner.to_string(),
                        token_id: Some(balance.token_id.clone()),
                        qty: Balance::new(balance.qty.value - balance.unlocked.value),
                    })),
                    None,
                )
            } else {
                let total_unlocked = (balance.qty.value as f64
                    * ((current_block - balance.at) as f64 / balance.duration as f64))
                    as BalancePrecision;

                let to_unlock = total_unlocked - balance.unlocked.value;

                let transfer = if to_unlock > 0 {
                    Some(Erc1155Action::Action::Transfer(Erc1155Action::Transfer {
                        from: Some(lock_account.to_string()),
                        target: owner.to_string(),
                        token_id: Some(balance.token_id.clone()),
                        qty: Balance::new(to_unlock),
                    }))
                } else {
                    None
                };

                (
                    transfer,
                    Some(LockedBalance::Linear(Linear {
                        token_id: balance.token_id.clone(),
                        from: balance.from.clone(),
                        qty: balance.qty,
                        at: balance.at,
                        duration: balance.duration,
                        unlocked: Balance::new(total_unlocked),
                    })),
                )
            }
        }
    }
}

#[async_trait(?Send)]
impl AsyncActionable for Unlock {
    async fn action(
        self,
        _caller: String,
        state: Parameters,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let lock_account = Contract::id();

        let current_block = Block::height() as u32;

        let (new_vault, transfers): (
            Vec<(String, Vec<LockedBalance>)>,
            Vec<Erc1155Action::Action>,
        ) = State::list_vault().await.iter().fold(
            (Vec::new(), Vec::new()),
            |(mut vault, mut all_transfers), (owner, balances)| {
                let (new_balances, mut transfers) = balances.iter().fold(
                    (Vec::new(), Vec::new()),
                    |(mut new_balances, mut transfers), balance| {
                        let (transfer, new_balance) = locked_balance_to_transfer(
                            current_block,
                            &lock_account,
                            &balance,
                            &owner,
                        );

                        if let Some(transfer) = transfer {
                            transfers.push(transfer);
                        }

                        if let Some(new_balance) = new_balance {
                            new_balances.push(new_balance);
                        }

                        (new_balances, transfers)
                    },
                );

                vault.push((owner.to_string(), new_balances));

                all_transfers.append(&mut transfers);

                (vault, all_transfers)
            },
        );

        // let vault: Vec<(String, Vec<LockedBalance>, Vec<Erc1155Action::Action>)> =
        //     State::list_vault()
        //         .await
        //         .iter()
        //         .fold(Vec::new(), |mut vault, (owner, balances)| {
        //             let (new_balances, transfers) = balances.iter().fold(
        //                 (Vec::new(), Vec::new()),
        //                 |(mut new_balances, mut transfers), balance| {
        //                     let (transfer, new_balance) = locked_balance_to_transfer(
        //                         current_block,
        //                         &lock_account,
        //                         &balance,
        //                         &owner,
        //                     );
        //
        //                     if let Some(transfer) = transfer {
        //                         transfers.push(transfer);
        //                     }
        //
        //                     if let Some(new_balance) = new_balance {
        //                         new_balances.push(new_balance);
        //                     }
        //
        //                     (new_balances, transfers)
        //                 },
        //             );
        //
        //             vault.push((owner.to_string(), new_balances, transfers));
        //
        //             vault
        //         });

        // let mut transfers = Vec::new();
        // for (owner, new_balances, expired_balances) in vault {
        //     for balance in expired_balances {
        //         transfers.push(Erc1155Action::Action::Transfer(Erc1155Action::Transfer {
        //             from: Some(lock_account.clone()),
        //             to: owner.clone(),
        //             token_id: Some(balance.token_id.clone()),
        //             qty: balance.qty,
        //         }));
        //     }
        //
        //     State::vault(&owner).set(&new_balances).await;
        // }

        for (owner, new_balances) in new_vault {
            if new_balances.len() == 0 {
                State::delete_vault(&owner).await;
            } else {
                State::vault(&owner).set(&new_balances).await;
            }
        }

        if transfers.len() > 0 {
            foreign_caller
                .write::<Erc1155ContractError, Erc1155Action::Action>(
                    &State::settings().erc1155().get().await,
                    Erc1155Action::Action::AsDirectCaller(Erc1155Action::AsDirectCaller {
                        action: Box::new(Erc1155Action::Action::Batch(Erc1155Action::Batch {
                            actions: transfers,
                        })),
                    }),
                )
                .await
                .or_else(|err| Err(ContractError::Erc1155Error(err)))?;
        }

        Ok(HandlerResult::None(state))
    }
}
