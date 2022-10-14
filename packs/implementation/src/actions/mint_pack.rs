use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, State as Erc1155State},
};

use warp_packs::{
    action::{ActionResult, HandlerResult, MintPack},
    error::ContractError,
    state::{Pack, PackScarcity, State},
};

use crate::{actions::AsyncActionable, contract_utils::foreign_call::write_foreign_contract};
use crate::{
    contract_utils::{foreign_call::read_foreign_contract_state, js_imports::Transaction},
    utils::get_all_nfts_ids,
};

// TODO: Move me somewhere that makes sense and remove my duplicates in other actions
#[derive(Debug, Serialize, Deserialize)]
pub struct InternalWriteResult {
    #[serde(rename = "type")]
    result_type: String,
}

async fn verify_nfts(
    erc1155: &String,
    nfts: &PackScarcity,
    all_packs: &HashMap<String, Pack>,
) -> Result<(), ContractError> {
    let nfts_vec: Vec<String> = nfts.into();

    for (pack_id, pack) in all_packs.iter() {
        for nft in &nfts_vec {
            if Vec::from(&pack.nfts).contains(&nft) {
                return Err(ContractError::NftAlreadyPacked(
                    pack_id.clone(),
                    nft.clone(),
                ));
            }
        }
    }

    let tokens = read_foreign_contract_state::<Erc1155State>(erc1155)
        .await
        .map_err(|_err| ContractError::Erc1155ReadFailed)?
        .tokens;

    for id in get_all_nfts_ids(nfts) {
        if !tokens.contains_key(&id) {
            return Err(ContractError::TokenNotFound(id));
        }
    }

    Ok(())
}

#[async_trait(?Send)]
impl AsyncActionable for MintPack {
    async fn action(self, _caller: String, mut state: State) -> ActionResult {
        verify_nfts(&state.settings.erc1155, &self.nfts, &state.packs).await?;

        let total_editions = match &self.nfts {
            PackScarcity::Legendary(_) => 11,
            PackScarcity::Epic(_) => 111,
            PackScarcity::Rare(_) => 1111,
        };

        let prefix = "PACK";
        let pack_id = format!(
            "{}-{}",
            prefix,
            self.ticker.clone().unwrap_or_else(Transaction::id)
        );

        state.packs.insert(
            pack_id.clone(),
            Pack {
                id: pack_id,
                nfts: self.nfts,
            },
        );

        let erc1155_mint = Erc1155Action::Action::Mint(Erc1155Action::Mint {
            ticker: self.ticker,
            prefix: Some(prefix.to_string()),
            qty: Balance::new(total_editions),
        });

        write_foreign_contract::<InternalWriteResult, Erc1155ContractError, Erc1155Action::Action>(
            &state.settings.erc1155,
            erc1155_mint,
        )
        .await
        .map_err(|err| ContractError::Erc1155Error(err))?;

        Ok(HandlerResult::Write(state))
    }
}
