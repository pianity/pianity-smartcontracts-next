import { readFileSync } from "node:fs";

import Arweave from "arweave";
import { JWKInterface } from "arweave/node/lib/wallet";
import { Contract, Warp, WriteInteractionOptions } from "warp-contracts";

import { State as Erc1155State } from "erc1155/State";
import { Action as Erc1155Action } from "erc1155/Action";
import { State as FeeState } from "fee/State";
import { State as PacksState } from "packs/State";

export const UNIT = 1_000_000;

type ContractName = "erc1155" | "fee" | "packs" | "test-multi-read";

export async function deployContract<T extends ContractName>(
    warp: Warp,
    opWallet: JWKInterface,
    contract: T,
    initState: T extends "erc1155"
        ? Erc1155State
        : T extends "fee"
        ? FeeState
        : T extends "packs"
        ? PacksState
        : T extends "test-multi-read"
        ? Record<string, never>
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

export function createInteractor<ACTION>(
    warp: Warp,
    contract: Contract,
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

        const interactionResult = await contract.writeInteraction(interaction, {
            ...defaultOptions,
            ...options,
        });

        // await warp.testing.mineBlock();

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

export function range(n: number): number[] {
    return [...Array(n).keys()];
}

export function dbg<T>(args: T): T {
    console.log(args);
    return args;
}
