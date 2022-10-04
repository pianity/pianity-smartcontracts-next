import { describe, it, expect, test, beforeAll, afterAll } from "@jest/globals";
import Arlocal from "arlocal";
import { Contract, LoggerFactory, Warp, WarpFactory } from "warp-contracts";
import { Wallet } from "warp-contracts/lib/types/contract/testing/Testing";

import { State, Token } from "erc1155/State";
import { Action } from "erc1155/Action";

import { createInteractor, deployContract } from "@/utils";

let arlocal: Arlocal;
let warp: Warp;

let op: Wallet;
let user: Wallet;

let contract: Contract<State>;
let contractId: string;
let interact: ReturnType<typeof createInteractor<Action>>;

beforeAll(async () => {
    LoggerFactory.INST.logLevel("error");
    LoggerFactory.INST.logLevel("debug", "WASM:Rust");
    LoggerFactory.INST.logLevel("debug", "ContractHandler");

    arlocal = new Arlocal(1984, false, `./arlocal.erc1155.db`, false);
    await arlocal.start();
    warp = WarpFactory.forLocal(1984, undefined, { inMemory: true, dbLocation: "/dev/null" });
    op = await warp.testing.generateWallet();
    user = await warp.testing.generateWallet();

    const initState = {
        name: "TEST-ERC1155",
        settings: {
            superOperator: op.address,
            operators: [],
            proxies: [],
            allowFreeTransfer: true,
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

    contractId = (await deployContract(warp, op.jwk, "erc1155", initState)).contractTxId;
    contract = warp
        .contract<State>(contractId)
        .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
        .connect(op.jwk);
    interact = createInteractor<Action>(warp, contract, op.jwk);

    console.log(`OP: ${op.address}\nUSER: ${user.address}\nERC1155: ${contractId}`);
}, 20_000);

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

    const { state } = (await contract.readState()).cachedValue;
    expect(state.tokens[tokenId].balances[op.address]).toBe("1");
    expect(state.tokens[tokenId].ticker).toBe(tokenId);
});

it("should burn an NFT", async () => {
    const mintResponse = await interact({
        function: "mint",
        prefix: "NFT",
        qty: "1",
    });

    const tokenId = `NFT-${mintResponse?.originalTxId}`;

    {
        const { state } = (await contract.readState()).cachedValue;
        expect(state.tokens[tokenId].balances[op.address]).toBe("1");
        expect(state.tokens[tokenId].ticker).toBe(tokenId);
    }

    await interact({
        function: "burn",
        tokenId,
        qty: "1",
    });

    {
        const { state } = (await contract.readState()).cachedValue;
        expect(state.tokens[tokenId]).toBeUndefined();
    }
});

it("should burn some tokens", async () => {
    const tokenId = "PTY";

    const mintResponse = await interact({
        function: "mint",
        ticker: tokenId,
        qty: "100",
    });

    {
        const { state } = (await contract.readState()).cachedValue;
        console.log(JSON.stringify(state.tokens, undefined, 2));
        expect(state.tokens[tokenId].balances[op.address]).toBe("100");
        expect(state.tokens[tokenId].ticker).toBe(tokenId);
        expect(calculateTotalQty(state.tokens[tokenId])).toBe("100");
    }

    await interact({
        function: "burn",
        tokenId,
        qty: "50",
    });

    {
        const { state } = (await contract.readState()).cachedValue;
        console.log(JSON.stringify(state.tokens, undefined, 2));
        expect(state.tokens[tokenId].balances[op.address]).toBe("50");
        expect(calculateTotalQty(state.tokens[tokenId])).toBe("50");
    }

    await interact({
        function: "burn",
        tokenId,
        qty: "50",
    });

    {
        const { state } = (await contract.readState()).cachedValue;
        console.log(JSON.stringify(state.tokens, undefined, 2));
        expect(state.tokens[tokenId]).toBeUndefined();
    }
});

it("should throw when non-op try to burn tokens", async () => {
    const burnInteraction = interact(
        {
            function: "burn",
            tokenId: "DOL",
            qty: "1",
        },
        { wallet: user.jwk },
    );

    await expect(burnInteraction).rejects.toThrow();
});

function calculateTotalQty(token: Token): string {
    return (
        Object.values(token.balances)
            // TODO: Use BigInt instead of parseInt
            .reduce((sum, balance) => sum + parseInt(balance), 0)
            .toString()
    );
}
