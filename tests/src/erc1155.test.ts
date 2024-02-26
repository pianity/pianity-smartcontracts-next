import { it, expect, beforeAll, afterAll } from "vitest";
import Arlocal from "arlocal";
import { Contract, LoggerFactory, Warp, WarpFactory } from "warp-contracts";
import { Wallet } from "warp-contracts/lib/types/contract/testing/Testing";
import { DeployPlugin } from "warp-contracts-plugin-deploy";

import * as Erc1155 from "erc1155/index";

import {
    createInteractor,
    createViewer,
    deployContract,
    deployInitTx,
    deployMainnetContract,
    expectError,
    expectOk,
    // expectError,
    // expectOk,
    generateWallet,
    Interactor,
    range,
    Viewer,
} from "@/utils";
import { PgSortKeyCache, PgSortKeyCacheOptions } from "warp-contracts-postgres";

let arlocal: Arlocal;
let warp: Warp;

let bank: Wallet;
let op: Wallet;
let user: Wallet;

let contract: Contract<Erc1155.Parameters>;
let contractId: string;
let interact: Interactor<Erc1155.Action, Erc1155.ContractError>;
let view: Viewer<Erc1155.Action, Erc1155.ReadResponse, Erc1155.Parameters, Erc1155.ContractError>;

beforeAll(async () => {
    LoggerFactory.INST.logLevel("error");
    LoggerFactory.INST.logLevel("debug", "WASM:Rust");
    LoggerFactory.INST.logLevel("debug", "ContractHandler");

    arlocal = new Arlocal(1984, false, `./arlocal.erc1155.db`, false);
    await arlocal.start();
    warp = WarpFactory.forLocal(1984, undefined, { inMemory: true, dbLocation: "/dev/null" }).use(
        new DeployPlugin(),
    );
    bank = await generateWallet();
    op = await generateWallet();
    user = await generateWallet();

    await warp.testing.addFunds(bank.jwk);
    await warp.testing.addFunds(op.jwk);
    await warp.testing.addFunds(user.jwk);

    const balances: Record<string, string> = {};
    for (let j = 0; j < 100; j++) {
        balances[`addess-${j}`] = "1000";
    }

    console.log("deploying contract...");
    const time = Date.now();
    const initState: Erc1155.Parameters = {
        name: "TEST-ERC1155",
        initialState: {
            tickerNonce: 0,
            settings: {
                defaultToken: "DOL",
                paused: false,
                superOperators: [bank.address, op.address],
                operators: [],
                proxies: [],
                allowFreeTransfer: true,
            },
            tokens: {
                DOL: {
                    ticker: "DOL",
                    balances: {
                        [bank.address]: `99999999999999`,
                        [op.address]: `200`,
                        ...balances,
                    },
                },
                // ...moreTokens,
            },
            approvals: {},
        },
        canEvolve: false,
    };

    contractId = (await deployContract(warp, op.jwk, "erc1155", initState)).contractTxId;
    console.log("contract deployed in ", (Date.now() - time) / 1000, "seconds");
    contract = warp
        .contract<Erc1155.Parameters>(contractId)
        .setEvaluationOptions({
            useKVStorage: true,
            internalWrites: true,
            mineArLocalBlocks: false,
            // throwOnInternalWriteError: false,
        })
        .connect(op.jwk);
    interact = createInteractor<Erc1155.Action, Erc1155.ContractError>(warp, contract, op.jwk);
    view = createViewer<
        Erc1155.Action,
        Erc1155.ReadResponse,
        Erc1155.Parameters,
        Erc1155.ContractError
    >(contract);

    console.log("OP:", op.address, "\nUSER:", user.address, "\nERC1155:", contractId);
}, 120_000);

afterAll(async () => {
    await arlocal.stop();
});

function calculateTotalQty(token: Erc1155.Token): string {
    return (
        Object.values(token.balances)
            // TODO: Use BigInt instead of parseInt
            .reduce((sum, balance) => sum + parseInt(balance), 0)
            .toString()
    );
}

it("fail if contract is not initialized", async () => {
    const stateBefore = (await contract.readState()).cachedValue.state;
    expect(stateBefore.initialState).toBeTruthy();

    const result = await interact({
        function: "mint",
        qty: "1",
    });

    expectError(result, { kind: "ContractUninitialized" });
});

it("initialize contract", async () => {
    const stateBefore = (await contract.readState()).cachedValue.state;
    expect(stateBefore.initialState).toBeTruthy();

    expectOk(await interact({ function: "initialize" }));

    const stateAfter = (await contract.readState()).cachedValue.state;
    expect(stateAfter.initialState).toBeNull();
});

it("non-operators are not allowed to transfer when allowFreeTransfer is false", async () => {
    expectOk(
        await interact({
            function: "configure",
            allowFreeTransfer: false,
        }),
    );

    const transferResponse = await interact(
        {
            function: "transfer",
            target: "burn",
            tokenId: "DOL",
            qty: "1",
        },
        { wallet: user.jwk },
    );

    expectError(transferResponse, {
        kind: "UnauthorizedAddress",
        data: user.address,
    });

    expectOk(
        await interact({
            function: "configure",
            allowFreeTransfer: true,
        }),
    );
});

it("should get the default token", async () => {
    const token = await view({ function: "getToken" });
    expectOk(token);
    expect(token.result[0]).toEqual("DOL");
});

it("fail if contract is already initialized", async () => {
    const stateBefore = (await contract.readState()).cachedValue.state;
    expect(stateBefore.initialState).toBeNull();

    expectError(await interact({ function: "initialize" }), {
        kind: "ContractAlreadyInitialized",
    });
});

it("should not accept write interactions when paused", async () => {
    expectOk(await interact({ function: "configure", paused: true }));

    expectOk(await interact({ function: "balanceOf", target: "" }));

    expectError(await interact({ function: "mint", qty: "1" }), {
        kind: "ContractIsPaused",
    });

    expectOk(await interact({ function: "configure", paused: false }));
});

it("should transfer some tokens to user", async () => {
    await interact({
        function: "transfer",
        target: user.address,
        tokenId: "DOL",
        qty: "100",
    });

    const opBalance = await view({ function: "balanceOf", target: op.address });
    expectOk(opBalance);
    expect(opBalance.result.balance).toBe("100");

    const userBalance = await view({ function: "balanceOf", target: user.address });
    expectOk(userBalance);
    expect(userBalance.result.balance).toBe("100");
});

it("should mint an NFT", async () => {
    const mintResponse = await interact({
        function: "mint",
        prefix: "NFT",
        qty: "1",
    });

    expectOk(mintResponse);

    const tokenId = `NFT-${mintResponse?.originalTxId}`;

    const opBalance = await view({ function: "balanceOf", target: op.address, tokenId });
    expectOk(opBalance);
    expect(opBalance.result.balance).toBe("1");
});

it("should burn an NFT", async () => {
    const mintResponse = await interact({
        function: "mint",
        prefix: "NFT",
        qty: "1",
    });

    expectOk(mintResponse);

    const tokenId = `NFT-${mintResponse.originalTxId}`;

    {
        const opBalance = await view({ function: "balanceOf", target: op.address, tokenId });
        expectOk(opBalance);
        expect(opBalance.result.balance).toBe("1");
    }

    await interact({
        function: "burn",
        tokenId,
        qty: "1",
    });

    {
        const token = await view({ function: "getToken", tokenId });
        expectError(token, {
            kind: "TokenNotFound",
            data: tokenId,
        });
    }
});

it("should burn some tokens", async () => {
    const tokenId = "PTY";

    const mintResponse = await interact({
        function: "mint",
        baseId: tokenId,
        qty: "100",
    });

    expectOk(mintResponse);

    {
        const token = await view({ function: "getToken", tokenId });
        expectOk(token);
        expect(token.result[1].balances[op.address]).toBe("100");
        expect(calculateTotalQty(token.result[1])).toBe("100");
    }

    await interact({
        function: "burn",
        tokenId,
        qty: "50",
    });

    {
        const token = await view({ function: "getToken", tokenId });
        expectOk(token);
        expect(token.result[1].balances[op.address]).toBe("50");
        expect(calculateTotalQty(token.result[1])).toBe("50");
    }

    await interact({
        function: "burn",
        tokenId,
        qty: "50",
    });

    {
        const token = await view({ function: "getToken", tokenId });
        expectError(token, {
            kind: "TokenNotFound",
            data: tokenId,
        });
    }
});

it("should throw when non-op try to burn tokens", async () => {
    const burnInteraction = await interact(
        {
            function: "burn",
            tokenId: "DOL",
            qty: "1",
        },
        { wallet: user.jwk },
    );

    expectError(burnInteraction, {
        kind: "UnauthorizedAddress",
        data: user.address,
    });
});

// NOTE: Errors are not correctly stored with Pianity's Warp fork yet
// it("publish an invalid interaction with strict:false and read the state", async () => {
//     // This interaction is invalid because `mint` requires being an operator and `user` isn't
//     const interaction = await interact(
//         {
//             function: "mint",
//             qty: "1",
//         },
//         { wallet: user.jwk, strict: false },
//     );
//
//     expectOk(interaction);
//
//     const state = (await contract.readState()).cachedValue;
//
//     expect(state.errorMessages[interaction.originalTxId]).toEqual({
//         kind: "UnauthorizedAddress",
//         data: user.address,
//     });
// });
