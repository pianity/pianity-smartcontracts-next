import { readFileSync } from "node:fs";

import Arweave from "arweave";
import { JWKInterface } from "arweave/node/lib/wallet";
import {
    Contract,
    Warp,
    WriteInteractionOptions,
    WriteInteractionResponse,
    WriteInteractionResponseFailure,
    WriteInteractionResponseSuccess,
} from "warp-contracts";
import { expect } from "@jest/globals";

import { State as Erc1155State } from "erc1155/State";
import { Action as Erc1155Action } from "erc1155/Action";
import { State as ScarcityState } from "scarcity/State";
import { State as ShuffleState } from "shuffle/State";
import { State as LockState } from "lock/State";

export const UNIT = 1_000_000;

export function expectOk(
    result: WriteInteractionResponse<unknown> | null,
): asserts result is WriteInteractionResponseSuccess {
    if (result?.type !== "ok") {
        console.log("interaction is error:", JSON.stringify(result, undefined, 2));
    }

    expect(result?.type).toEqual("ok");
}

export function expectError<ERROR>(
    result: WriteInteractionResponse<ERROR> | null,
    expectedError?: ERROR,
): asserts result is WriteInteractionResponseFailure<ERROR> {
    if (result?.type !== "error") {
        console.log("interaction is ok:", JSON.stringify(result, undefined, 2));
    }

    expect(result?.type).toEqual("error");

    if (expectedError) {
        expect((result as WriteInteractionResponseFailure<ERROR>).error).toEqual(expectedError);
    }
}

type ContractName = "erc1155" | "scarcity" | "shuffle" | "lock";

export async function deployContract<T extends ContractName>(
    warp: Warp,
    opWallet: JWKInterface,
    contract: T,
    initState: T extends "erc1155"
        ? Erc1155State
        : T extends "scarcity"
        ? ScarcityState
        : T extends "shuffle"
        ? ShuffleState
        : T extends "lock"
        ? LockState
        : never,
) {
    const wasmDir = `../${contract}/implementation/pkg`;
    const wasmGluecode = `${wasmDir}/rust-contract.js`;
    const wasmPath = `${wasmDir}/rust-contract_bg.wasm`;

    const deployment = await warp.createContract.deploy({
        wallet: opWallet,
        initState: JSON.stringify(initState),
        wasmSrcCodeDir: wasmDir,
        wasmGlueCode: wasmGluecode,
        src: readFileSync(wasmPath),
    });

    return deployment;
}

export type Interactor<ACTION, ERROR> = (
    interaction: ACTION,
    options?: { wallet?: JWKInterface } & WriteInteractionOptions,
) => Promise<WriteInteractionResponse<ERROR> | null>;

export function createInteractor<ACTION, ERROR>(
    warp: Warp,
    contract: Contract<unknown, ERROR>,
    defaultWallet: JWKInterface,
    defaultOptions: WriteInteractionOptions = {},
): Interactor<ACTION, ERROR> {
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
