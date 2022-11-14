import { readFileSync } from "node:fs";

import Arweave from "arweave";
import { JWKInterface } from "arweave/node/lib/wallet";
import { Contract, Warp, WriteInteractionOptions } from "warp-contracts";
import { expect } from "@jest/globals";

import { State as Erc1155State } from "erc1155/State";
import { Action as Erc1155Action } from "erc1155/Action";
import { State as ScarcityState } from "scarcity/State";
import { State as ShuffleState } from "shuffle/State";

export const UNIT = 1_000_000;

export function expectOk(resultType: string | undefined): asserts resultType is "ok" {
    expect(resultType).toEqual("ok");
}

export function expectError(resultType: string | undefined): asserts resultType is "error" {
    expect(resultType).toEqual("error");
}

type ContractName = "erc1155" | "scarcity" | "shuffle" | "test-multi-read";

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

export function createInteractor<ACTION, STATE, ERROR>(
    warp: Warp,
    contract: Contract<STATE, ERROR>,
    defaultWallet: JWKInterface,
    defaultOptions: WriteInteractionOptions = {},
) {
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
