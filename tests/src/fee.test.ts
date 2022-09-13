import { describe, it, expect, test, beforeAll, afterAll } from "@jest/globals";
import { JWKInterface } from "arweave/node/lib/wallet";
import Arlocal from "arlocal";
import {
    Contract,
    LoggerFactory,
    Warp,
    WarpFactory,
    WriteInteractionResponse,
} from "warp-contracts";
import { Wallet } from "warp-contracts/lib/types/contract/testing/Testing";

import { State as Erc1155State } from "erc1155/State";
import { Action as Erc1155Action } from "erc1155/Action";
import { State as FeeState } from "fee/State";
import { Action as FeeAction } from "fee/Action";

import { UNIT, deployERC1155, createInteractor, deployFee } from "@/utils";

describe("test fee contract", () => {
    let warp: Warp;
    let arlocal: Arlocal;

    let op: Wallet;
    let user: Wallet;

    let erc1155Contract: Contract<Erc1155State>;
    let erc1155TxId: string;
    let erc1155Interact: ReturnType<typeof createInteractor<Erc1155Action>>;

    let feeContract: Contract<FeeState>;
    let feeTxId: string;
    let feeInteract: ReturnType<typeof createInteractor<FeeAction>>;

    beforeAll(async () => {
        LoggerFactory.INST.logLevel("error");
        LoggerFactory.INST.logLevel("debug", "WASM:Rust");
        LoggerFactory.INST.logLevel("debug", "ContractHandler");

        warp = WarpFactory.forLocal(1985);
        arlocal = new Arlocal(1985, false, `./arlocal.2.db`, false);
        await arlocal.start();
        op = await warp.testing.generateWallet();
        console.log("op address:", op.address);
        user = await warp.testing.generateWallet();
        console.log("user address:", user.address);

        const erc1155InitState = {
            name: "TEST-ERC1155",
            settings: {
                superOperator: op.address,
                operators: [],
                transferProxies: [],
            },
            tokens: {
                DOL: {
                    ticker: "DOL",
                    balances: {
                        [op.address]: `${100 * UNIT}`,
                    },
                },
            },
            approvals: {},
        };

        erc1155TxId = (await deployERC1155(warp, op.jwk, erc1155InitState)).contractTxId;
        erc1155Contract = warp
            .contract<Erc1155State>(erc1155TxId)
            .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
            .connect(op.jwk);
        erc1155Interact = createInteractor<Erc1155Action>(warp, erc1155Contract, op.jwk);

        const feeInitState: FeeState = {
            name: "TEST-FEE",
            settings: {
                superOperator: op.address,
                operators: [],
                authorizedAddresses: [],
                erc1155: erc1155TxId,
                custodian: op.address,
                exchangeToken: "DOL",
            },
            tokens: {},
        };

        feeTxId = (await deployFee(warp, op.jwk, feeInitState)).contractTxId;
        feeContract = warp
            .contract<FeeState>(feeTxId)
            .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
            .connect(op.jwk);
        feeInteract = createInteractor<FeeAction>(warp, feeContract, op.jwk);
    }, 10_000);

    afterAll(async () => {
        await arlocal.stop();
    });

    it("should activate the fee contract on the erc1155 one", async () => {
        console.log("Activate the Fee proxy contract...");
        await erc1155Interact({
            function: "configure",
            transferProxies: [feeTxId, op.address],
        });

        const { cachedValue } = await erc1155Contract.readState();
        expect(cachedValue.state.settings.transferProxies).toEqual([feeTxId, op.address]);
    });

    it("should draw the rest of the owl", async () => {
        console.log("Mint an NFT...");
        const mintResponse = await erc1155Interact({
            function: "mint",
            prefix: "NFT",
            qty: "1",
        });

        const tokenId = `NFT-${mintResponse?.originalTxId}`;

        console.log("Create the fees...");
        await feeInteract({
            function: "createFee",
            tokenId,
            fees: {
                [op.address]: UNIT,
            },
            rate: UNIT * 0.1,
        });

        console.log("Transfer some DOL to user...");
        await erc1155Interact({
            function: "transfer",
            from: op.address,
            tokenId: "DOL",
            to: user.address,
            qty: `${5 * UNIT}`,
        });

        console.log("Approve op for user...");
        await erc1155Interact(
            {
                function: "setApprovalForAll",
                operator: op.address,
                approved: true,
            },
            user.jwk,
        );

        console.log("Buy the nft...");
        await feeInteract({
            function: "transfer",
            to: user.address,
            price: `${1 * UNIT}`,
            tokenId,
        });
    });
});
