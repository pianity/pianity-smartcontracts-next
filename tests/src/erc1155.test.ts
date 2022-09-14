import { describe, it, expect, test, beforeAll, afterAll } from "@jest/globals";
import Arlocal from "arlocal";
import { Contract, LoggerFactory, Warp, WarpFactory } from "warp-contracts";
import { Wallet } from "warp-contracts/lib/types/contract/testing/Testing";

import { State as State } from "erc1155/State";
import { Action as Action } from "erc1155/Action";

import { deployERC1155, createInteractor } from "@/utils";

describe("test erc1155 contract", () => {
    let warp: Warp;
    let arlocal: Arlocal;

    let op: Wallet;
    let user: Wallet;

    let contract: Contract<State>;
    let contractId: string;
    let interact: ReturnType<typeof createInteractor<Action>>;

    beforeAll(async () => {
        LoggerFactory.INST.logLevel("error");
        LoggerFactory.INST.logLevel("debug", "WASM:Rust");
        LoggerFactory.INST.logLevel("debug", "ContractHandler");

        warp = WarpFactory.forLocal(1984);
        arlocal = new Arlocal(1984, false, `./arlocal.db`, false);
        await arlocal.start();
        op = await warp.testing.generateWallet();
        console.log("op address:", op.address);
        user = await warp.testing.generateWallet();
        console.log("user address:", user.address);

        const initState = {
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
                        [op.address]: `200`,
                    },
                },
            },
            approvals: {},
        };

        contractId = (await deployERC1155(warp, op.jwk, initState)).contractTxId;
        contract = warp
            .contract<State>(contractId)
            .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
            .connect(op.jwk);
        interact = createInteractor<Action>(warp, contract, op.jwk);
    });

    afterAll(async () => {
        await arlocal.stop();
    });

    it("should transfer some tokens to user", async () => {
        await interact({
            function: "transfer",
            to: user.address,
            tokenId: "DOL",
            qty: "100",
        });

        const { state } = (await contract.readState()).cachedValue;
        expect(state.tokens.DOL.balances[op.address]).toBe("100");
        expect(state.tokens.DOL.balances[user.address]).toBe("100");
    });

    it("should mint an NFT", async () => {
        const mintResponse = await interact({
            function: "mint",
            prefix: "NFT",
            qty: "1",
        });

        const tokenId = `NFT-${mintResponse?.originalTxId}`;
        console.log(tokenId);

        const { state } = (await contract.readState()).cachedValue;
        expect(state.tokens[tokenId].balances[op.address]).toBe("1");
        expect(state.tokens[tokenId].ticker).toBe(tokenId);
    });
});
