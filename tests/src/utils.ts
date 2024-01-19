import { readFileSync } from "node:fs";

import { expect } from "vitest";
import Arweave from "arweave";
import { JWKInterface } from "arweave/node/lib/wallet";
import {
    Contract,
    ContractError,
    InteractionResult,
    InteractionResultType,
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
import { Parameters as ScarcityState } from "scarcity/State";
import { Parameters as LockState } from "lock/State";
import { Signer } from "warp-arbundles";
import { ArweaveSigner } from "warp-contracts-plugin-deploy";

export const UNIT = 1_000_000;

export function expectOk(result: {
    type: InteractionResultType;
}): asserts result is { type: "ok" } {
    const stack = new Error().stack;

    try {
        expect(
            result?.type,
            `interaction isn't ok: ${JSON.stringify(result, undefined, 2)}`,
        ).toEqual("ok");
    } catch (error) {
        console.log(stack);
        throw error;
    }
}

export function expectError<ERROR, const NARROWED_ERROR extends ERROR | undefined = undefined>(
    result: { type: InteractionResultType; error?: ERROR },
    expectedError?: NARROWED_ERROR,
): asserts result is { type: "error"; error: NARROWED_ERROR } {
    if (result?.type !== "error") {
        console.log("interaction is ok:", JSON.stringify(result, undefined, 2));
    }

    const stack = new Error().stack;

    try {
        expect(
            result?.type,
            `interaction isn't error: ${JSON.stringify(result, undefined, 2)}`,
        ).toEqual("error");

        if (expectedError) {
            expect((result as InteractionFailure<ERROR>).error).toEqual(expectedError);
        }
    } catch (error) {
        console.log(stack);
        throw error;
    }
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
        : T extends "lock"
        ? LockState
        : // : T extends "shuffle"
          // ? ShuffleState
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

export type InteractionSuccess = { type: "ok" } & WriteInteractionResponse;
export type InteractionFailure<T> = { type: "error"; error: T };
export type SafeWriteInteractionResponse<T> = InteractionSuccess | InteractionFailure<T>;

export type Interactor<ACTION, ERROR> = (
    interaction: ACTION,
    options?: { wallet?: JWKInterface | Signer; mine?: boolean } & WriteInteractionOptions,
) => Promise<SafeWriteInteractionResponse<ERROR>>;

export function createInteractor<ACTION, ERROR>(
    warp: Warp,
    contract: Contract<unknown>,
    defaultWallet: JWKInterface,
    defaultOptions: WriteInteractionOptions = {},
): Interactor<ACTION, ERROR> {
    defaultOptions = { strict: true, ...defaultOptions };

    return async (
        interaction: ACTION,
        options: { wallet?: JWKInterface | Signer; mine?: boolean } & WriteInteractionOptions = {},
    ) => {
        if (options.wallet) {
            contract.connect(options.wallet);
        } else {
            contract.connect(defaultWallet);
        }
        options.mine ??= true;

        const now = Date.now();
        try {
            console.log("CALLING WRITE INTERACTION");
            const interactionResult = await contract.writeInteraction(interaction, {
                ...defaultOptions,
                ...options,
            });
            console.log("interaction took", Date.now() - now, "ms");

            if (options.mine) {
                console.log("MINING A BLOCK");
                await warp.testing.mineBlock();
            }

            return { type: "ok", ...interactionResult! };
        } catch (error) {
            if (error instanceof ContractError) {
                return { type: "error", error: error.error };
            } else {
                throw error;
            }
        }
    };
}

type UnionToIntersection<U> = (U extends any ? (k: U) => void : never) extends (k: infer I) => void
    ? I
    : never;

export type Viewer<Action, Result, State, Error> = ReturnType<
    typeof createViewer<Action, Result, State, Error>
>;

export function createViewer2<Action, Result, State>(contract: Contract<State>) {
    type Results = UnionToIntersection<Result>;
    type FunctionsWithResults = keyof Results;

    return function view<T extends Action & { function: FunctionsWithResults }>(
        action: T,
        // ): Promise<InteractionResult<State, Results[T["function"]]>> {
    ): Promise<InteractionResult<State, Pick<Results, T["function"]>>> {
        return contract.viewState(action);
    };
}

export function createViewer<Action, Result, State, Error>(contract: Contract<State>) {
    type Results = UnionToIntersection<Result>;
    type FunctionsWithResults = keyof Results;

    // type CorrectInteractionResult<T extends Action & { function: FunctionsWithResults }> =
    //     InteractionResult<State, Pick<Results, T["function"]>>;
    // type ViewResult<T extends Action & { function: FunctionsWithResults }> =
    //     | { type: "ok"; result: CorrectInteractionResult<T>["result"] }
    //     | { type: "error"; error: Error };

    // return async function view<const T extends Action & { function: FunctionsWithResults }>(
    //     action: T,
    // ): Promise<ViewResult<T>> {
    return async function view<T extends Action & { function: FunctionsWithResults }>(
        action: T,
    ): Promise<
        | {
              type: "ok";
              result: InteractionResult<
                  State,
                  Pick<Results, T["function"]>
              >["result"][T["function"]];
          }
        | { type: "error"; error: Error }
    > {
        const view = await contract.viewState(action);

        if (view.type === "ok") {
            return { type: "ok", result: (view.result as any)[action.function] };
        } else {
            return { type: "error", error: view.error as any };
        }
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

export function x<T>(fn: () => T): T {
    return fn();
}

export function sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function waitBlocks(arweave: Arweave, blocks: number) {
    const height = (await arweave.network.getInfo()).height;

    while ((await arweave.network.getInfo()).height < height + blocks) {
        await sleep(1000);
    }
}
