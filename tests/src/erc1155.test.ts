// type UnionToIntersection<U> = (U extends any ? (k: U) => void : never) extends (k: infer I) => void
//     ? I
//     : never;
//
// type Action =
//     | {
//           function: "balanceOf";
//           address: string;
//       }
//     | {
//           function: "isApprovedForAll";
//           address: string;
//       }
//     | {
//           function: "mint";
//           address: string;
//       };
//
// type Result =
//     | {
//           balanceOf: {
//               address: string;
//               balance: string;
//           };
//       }
//     | {
//           isApprovedForAll: {
//               address: string;
//               approved: boolean;
//           };
//       };
//
// // Higher-order function `createView`
// function createView<Result, Action>() {
//     type Results = UnionToIntersection<Result>;
//     type FunctionsWithResults = keyof Results;
//
//     return function view<T extends Action & { function: FunctionsWithResults }>(
//         action: T,
//     ): Results[T["function"]] {
//         return null as any;
//     };
// }
//
// // Usage
// const viewFunction = createView<Result, Action>();
// const result = viewFunction({ function: "balanceOf", address: "0x123" });

// const test = view({ function: "balanceOf", address: "123" });

// import { it, expect, test, beforeAll, afterAll } from "@jest/globals";
import { it, expect, test, beforeAll, afterAll } from "vitest";
import Arlocal from "arlocal";
import { Contract, LoggerFactory, Warp, WarpFactory } from "warp-contracts";
import { Wallet } from "warp-contracts/lib/types/contract/testing/Testing";
import { DeployPlugin } from "warp-contracts-plugin-deploy";

import { Parameters as State, Token } from "erc1155/State";
import { Action } from "erc1155/Action";
import { ReadResponse } from "erc1155/ReadResponse";
import { ContractError } from "erc1155/ContractError";

import {
    createInteractor,
    createViewer,
    deployContract,
    deployInitTx,
    deployMainnetContract,
    // expectError,
    // expectOk,
    generateWallet,
    Interactor,
    range,
    Viewer,
} from "@/utils";

let arlocal: Arlocal;
let warp: Warp;

let op: Wallet;
let user: Wallet;

let contract: Contract<State>;
let contractId: string;
let interact: Interactor<Action>;
let view: Viewer<Action, ReadResponse, State>;

beforeAll(async () => {
    LoggerFactory.INST.logLevel("error");
    LoggerFactory.INST.logLevel("error", "WASM:Rust");
    // LoggerFactory.INST.logLevel("debug", "ContractHandler");

    arlocal = new Arlocal(1984, false, `./arlocal.erc1155.db`, false);
    await arlocal.start();
    warp = WarpFactory.forLocal(1984, undefined, { inMemory: true, dbLocation: "/dev/null" }).use(
        new DeployPlugin(),
    );
    op = await generateWallet();
    user = await generateWallet();

    await warp.testing.addFunds(op.jwk);
    await warp.testing.addFunds(user.jwk);

    // const moreTokens: Record<string, Token> = {};
    // for (let i = 0; i < 100_000; i++) {
    //     const balances: Record<string, string> = {};
    //     for (let j = 0; j < 100; j++) {
    //         balances[`addess-${j}`] = "1000";
    //     }
    //     moreTokens[`PTY-${i}`] = {
    //         ticker: `PTY-${i}`,
    //         balances,
    //     };
    // }

    const balances: Record<string, string> = {};
    for (let j = 0; j < 100; j++) {
        balances[`addess-${j}`] = "1000";
    }

    console.log("deploying contract...");
    const time = Date.now();
    const initState: State = {
        name: "TEST-ERC1155",
        initialState: {
            tickerNonce: 0,
            settings: {
                defaultToken: "DOL",
                paused: false,
                superOperators: [op.address],
                operators: [],
                proxies: [],
                allowFreeTransfer: true,
                canEvolve: false,
            },
            tokens: {
                DOL: {
                    ticker: "DOL",
                    balances: {
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

    contractId = (await deployInitTx(warp, op.jwk, "erc1155", JSON.stringify(initState)))
        .contractTxId;
    // contractId = (await deployContract(warp, op.jwk, "erc1155", initState)).contractTxId;
    console.log("contract deployed in ", (Date.now() - time) / 1000, "seconds");
    contract = warp
        .contract<State>(contractId)
        .setEvaluationOptions({
            useKVStorage: true,
            internalWrites: false,
            throwOnInternalWriteError: false,
        })
        .connect(op.jwk);
    interact = createInteractor<Action>(warp, contract, op.jwk);
    view = createViewer<Action, ReadResponse, State>(contract);

    console.log("OP:", op.address, "\nUSER:", user.address, "\nERC1155:", contractId);
}, 120_000);

afterAll(async () => {
    await arlocal.stop();
});

it(
    "should transfer some tokens to user",
    async () => {
        console.log("###########################################>>>>>>>>>>>>>>>>>>> EXEC1");
        await interact(
            {
                function: "transfer",
                to: user.address,
                tokenId: "DOL",
                qty: "100",
            },
            {
                wallet: op.jwk,
            },
        );

        console.log("###########################################>>>>>>>>>>>>>>>>>>> EXEC2");
        const opBalance = await view({ function: "balanceOf", target: user.address });

        console.log("###########################################>>>>>>>>>>>>>>>>>>> EXEC3");
        const userBalance = await view({ function: "balanceOf", target: user.address });

        expect(opBalance.result.balanceOf.balance).toBe("100");
        expect(userBalance.result.balanceOf.balance).toBe("100");
    },
    { timeout: 5_000 },
);

// it("should not accept interactions when paused", async () => {
//     await interact({ function: "configure", paused: true });
//
//     await expect(interact({ function: "balanceOf", target: "" })).rejects.toThrow();
//
//     await interact({ function: "configure", paused: false });
// });

// it("should not accept interactions when paused", async () => {
//     expectOk(await interact({ function: "configure", paused: true }));
//
//     expectError(await interact({ function: "balanceOf", target: "" }), {
//         kind: "ContractIsPaused",
//     });
//
//     expectOk(await interact({ function: "configure", paused: false }));
// });
//
// it("should transfer some tokens to user", async () => {
//     await interact({
//         function: "transfer",
//         to: user.address,
//         tokenId: "DOL",
//         qty: "100",
//     });
//
//     const { state } = (await contract.readState()).cachedValue;
//     expect(state.tokens.DOL.balances[op.address]).toBe("100");
//     expect(state.tokens.DOL.balances[user.address]).toBe("100");
// });
//
// it("should mint an NFT", async () => {
//     const mintResponse = await interact({
//         function: "mint",
//         prefix: "NFT",
//         qty: "1",
//     });
//
//     expectOk(mintResponse);
//
//     const tokenId = `NFT-${mintResponse?.originalTxId}`;
//
//     const { state } = (await contract.readState()).cachedValue;
//     expect(state.tokens[tokenId].balances[op.address]).toBe("1");
// });
//
// it("should burn an NFT", async () => {
//     const mintResponse = await interact({
//         function: "mint",
//         prefix: "NFT",
//         qty: "1",
//     });
//
//     expectOk(mintResponse);
//
//     const tokenId = `NFT-${mintResponse.originalTxId}`;
//
//     {
//         const { state } = (await contract.readState()).cachedValue;
//         expect(state.tokens[tokenId].balances[op.address]).toBe("1");
//     }
//
//     await interact({
//         function: "burn",
//         tokenId,
//         qty: "1",
//     });
//
//     {
//         const { state } = (await contract.readState()).cachedValue;
//         expect(state.tokens[tokenId]).toBeUndefined();
//     }
// });
//
// it("should burn some tokens", async () => {
//     const tokenId = "PTY";
//
//     const mintResponse = await interact({
//         function: "mint",
//         baseId: tokenId,
//         qty: "100",
//     });
//
//     expectOk(mintResponse);
//
//     {
//         const { state } = (await contract.readState()).cachedValue;
//         expect(state.tokens[tokenId].balances[op.address]).toBe("100");
//         expect(calculateTotalQty(state.tokens[tokenId])).toBe("100");
//     }
//
//     await interact({
//         function: "burn",
//         tokenId,
//         qty: "50",
//     });
//
//     {
//         const { state } = (await contract.readState()).cachedValue;
//         expect(state.tokens[tokenId].balances[op.address]).toBe("50");
//         expect(calculateTotalQty(state.tokens[tokenId])).toBe("50");
//     }
//
//     await interact({
//         function: "burn",
//         tokenId,
//         qty: "50",
//     });
//
//     {
//         const { state } = (await contract.readState()).cachedValue;
//         expect(state.tokens[tokenId]).toBeUndefined();
//     }
// });
//
// it("should throw when non-op try to burn tokens", async () => {
//     const burnInteraction = await interact(
//         {
//             function: "burn",
//             tokenId: "DOL",
//             qty: "1",
//         },
//         { wallet: user.jwk },
//     );
//
//     expectError(burnInteraction, {
//         kind: "UnauthorizedAddress",
//         data: user.address,
//     });
// });
//
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
//     expect(state.errors[interaction.originalTxId]).toEqual({
//         kind: "UnauthorizedAddress",
//         data: user.address,
//     });
// });
//
// function calculateTotalQty(token: Token): string {
//     return (
//         Object.values(token.balances)
//             // TODO: Use BigInt instead of parseInt
//             .reduce((sum, balance) => sum + parseInt(balance), 0)
//             .toString()
//     );
// }
