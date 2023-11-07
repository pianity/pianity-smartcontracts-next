import { readFileSync } from "node:fs";

import { expect } from "vitest";
import Arweave from "arweave";
import { JWKInterface } from "arweave/node/lib/wallet";
import {
    Contract,
    ContractError,
    InteractionResult,
    Tag,
    WARP_TAGS,
    Warp,
    WriteInteractionOptions,
    WriteInteractionResponse,
    // WriteInteractionResponseFailure,
    // WriteInteractionResponseSuccess,
} from "warp-contracts";
import Irys from "@irys/sdk";

import { Parameters as Erc1155State } from "erc1155/State";
import { Action as Erc1155Action } from "erc1155/Action";
import { ArweaveSigner } from "warp-contracts-plugin-deploy";
import { Parameters as ScarcityState } from "scarcity/State";
// import { State as ShuffleState } from "shuffle/State";
// import { State as LockState } from "lock/State";
//
// export const UNIT = 1_000_000;

// export function expectOk(result: WriteInteractionResponse | null) {
//     result.
//     if (result?.type !== "ok") {
//         console.log("interaction is error:", JSON.stringify(result, undefined, 2));
//     }
//
//     expect(result?.type).toEqual("ok");
// }

export async function expectError<ERROR>(promise: Promise<unknown>, expectedError?: ERROR) {
    try {
        await promise;
        throw new Error("Promise didn't throw");
    } catch (error) {
        expect(error).toBeInstanceOf(ContractError);
        if (error instanceof ContractError) {
            expect(error.error).toEqual(expectedError);
        }
    }

    // if (result?.type !== "error") {
    //     console.log("interaction is ok:", JSON.stringify(result, undefined, 2));
    // }
    //
    // expect(result?.type).toEqual("error");
    //
    // if (expectedError) {
    //     expect((result as WriteInteractionResponseFailure<ERROR>).error).toEqual(expectedError);
    // }
}

type ContractName = "erc1155" | "scarcity" | "shuffle" | "lock";

export async function deployInitState(arweave: Arweave, wallet: JWKInterface, state: string) {
    const address = await arweave.wallets.jwkToAddress(wallet);
    const irys = new Irys({ url: "https://node1.irys.xyz", token: "arweave", key: wallet });

    const balance = await irys.getBalance(address);
    console.log("irys fund:", balance.toString());
    // console.log("funding:", await irys.fund(irys.utils.toAtomic(0.2)));

    const tx = await irys.upload(state, {
        tags: [{ name: "Content-Type", value: "application/json" }],
    });

    return tx.id;
}

export async function deployInitTx<T extends ContractName>(
    warp: Warp,
    wallet: JWKInterface,
    contract: T,
    initState: string,
) {
    const wasmDir = `../${contract}/implementation/pkg`;
    const wasmGluecode = `${wasmDir}/rust-contract.js`;
    const wasmPath = `${wasmDir}/rust-contract_bg.wasm`;

    // const initStateTx = await deployInitState(warp.arweave, wallet, JSON.stringify(initState));
    const initStateTx = await warp.arweave.createTransaction({ data: initState }, wallet);
    await warp.arweave.transactions.sign(initStateTx, wallet);
    console.log(await warp.arweave.transactions.post(initStateTx));
    await warp.testing.mineBlock();
    await warp.testing.mineBlock();
    await warp.testing.mineBlock();
    await warp.testing.mineBlock();

    const initStateTxId = initStateTx.id;

    // const deployment = await warp.deploy(
    //     {
    //         initStateTx,
    //         // wallet: new ArweaveSigner(wallet),
    //         wallet,
    //         wasmSrcCodeDir: wasmDir,
    //         wasmGlueCode: wasmGluecode,
    //         src: readFileSync(wasmPath),
    //         evaluationManifest: {
    //             evaluationOptions: {
    //                 useKVStorage: true,
    //             },
    //         },
    //     } as any,
    //     true,
    // );

    const deployment = await warp.deploy(
        {
            wallet,
            initState: "",
            wasmSrcCodeDir: wasmDir,
            wasmGlueCode: wasmGluecode,
            src: readFileSync(wasmPath),
            tags: [new Tag(WARP_TAGS.INIT_STATE_TX, initStateTxId)],
            data: {
                "Content-Type": "text/html",
                body: "<h1>hello</h1>",
            },
            evaluationManifest: {
                evaluationOptions: {
                    useKVStorage: true,
                },
            },
        },
        true,
    );

    return deployment;
}

export async function deployMainnetContract<T extends ContractName>(
    warp: Warp,
    wallet: JWKInterface,
    contract: T,
    initStateTxId: string,
) {
    const wasmDir = `../${contract}/implementation/pkg`;
    const wasmGluecode = `${wasmDir}/rust-contract.js`;
    const wasmPath = `${wasmDir}/rust-contract_bg.wasm`;

    // const initStateTx = await deployInitState(warp.arweave, wallet, JSON.stringify(initState));
    // const initStateTx = "IMgRH_JDL9vw2XlM2pvBnPOYkgR9q9Lfncg2irqg050";

    // const deployment = await warp.deploy(
    //     {
    //         initStateTx,
    //         // wallet: new ArweaveSigner(wallet),
    //         wallet,
    //         wasmSrcCodeDir: wasmDir,
    //         wasmGlueCode: wasmGluecode,
    //         src: readFileSync(wasmPath),
    //         evaluationManifest: {
    //             evaluationOptions: {
    //                 useKVStorage: true,
    //             },
    //         },
    //     } as any,
    //     true,
    // );

    const deployment = await warp.deploy(
        {
            wallet,
            initState: "",
            wasmSrcCodeDir: wasmDir,
            wasmGlueCode: wasmGluecode,
            src: readFileSync(wasmPath),
            tags: [new Tag(WARP_TAGS.INIT_STATE_TX, initStateTxId)],
            data: {
                "Content-Type": "text/html",
                body: "<h1>hello</h1>",
            },
            evaluationManifest: {
                evaluationOptions: {
                    useKVStorage: true,
                },
            },
        },
        true,
    );

    return deployment;
}

export async function deployContract<T extends ContractName>(
    warp: Warp,
    opWallet: JWKInterface,
    contract: T,
    initState: T extends "erc1155"
        ? Erc1155State
        : T extends "scarcity"
        ? ScarcityState
        : // : T extends "shuffle"
          // ? ShuffleState
          // : T extends "lock"
          // ? LockState
          never,
) {
    const wasmDir = `../${contract}/implementation/pkg`;
    const wasmGluecode = `${wasmDir}/rust-contract.js`;
    const wasmPath = `${wasmDir}/rust-contract_bg.wasm`;

    const deployment = await warp.deploy({
        wallet: opWallet,
        initState: JSON.stringify(initState),
        wasmSrcCodeDir: wasmDir,
        wasmGlueCode: wasmGluecode,
        src: readFileSync(wasmPath),
        evaluationManifest: {
            evaluationOptions: {
                useKVStorage: true,
            },
        },
    });

    return deployment;
}

export type Interactor<ACTION> = (
    interaction: ACTION,
    options?: { wallet?: JWKInterface } & WriteInteractionOptions,
) => Promise<WriteInteractionResponse | null>;

export function createInteractor<ACTION>(
    warp: Warp,
    contract: Contract<unknown>,
    defaultWallet: JWKInterface,
    defaultOptions: WriteInteractionOptions = {},
): Interactor<ACTION> {
    defaultOptions = { strict: true, ...defaultOptions };

    return async (
        interaction: ACTION,
        options: { wallet?: JWKInterface } & WriteInteractionOptions = {},
    ) => {
        if (options.wallet) {
            contract.connect(options.wallet);
        } else {
            contract.connect(defaultWallet);
        }

        const now = Date.now();
        const interactionResult = await contract.writeInteraction(interaction, {
            ...defaultOptions,
            ...options,
        });
        console.log("interaction took", Date.now() - now, "ms");

        await warp.testing.mineBlock();

        return interactionResult;
    };
}

type UnionToIntersection<U> = (U extends any ? (k: U) => void : never) extends (k: infer I) => void
    ? I
    : never;

export type Viewer<Action, Result, State> = ReturnType<typeof createViewer<Action, Result, State>>;

export function createViewer<Action, Result, State>(contract: Contract<State>) {
    type Results = UnionToIntersection<Result>;
    type FunctionsWithResults = keyof Results;

    return function view<T extends Action & { function: FunctionsWithResults }>(
        action: T,
        // ): Promise<InteractionResult<State, Results[T["function"]]>> {
    ): Promise<InteractionResult<State, Pick<Results, T["function"]>>> {
        return contract.viewState(action);
    };
}

export type Wallet = { jwk: JWKInterface; address: string };

export async function generateWallet(): Promise<Wallet> {
    const arweave = Arweave.init({});
    const jwk = await arweave.wallets.generate();
    const address = await arweave.wallets.jwkToAddress(jwk);

    return {
        jwk,
        address,
    };
}

export function range(end: number, start?: number): number[] {
    if (start) {
        return Array.from({ length: end - start }, (_, i) => start + i);
    } else {
        return [...Array(end).keys()];
    }
}

export function dbg<T>(args: T): T {
    console.log(args);
    return args;
}
