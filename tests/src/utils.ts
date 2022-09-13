import { readFileSync } from "node:fs";

import { JWKInterface } from "arweave/node/lib/wallet";
import { Contract, Warp } from "warp-contracts";

import { State as Erc1155State } from "erc1155/State";
import { Action as Erc1155Action } from "erc1155/Action";
import { State as FeeState } from "fee/State";
import { Action as FeeAction } from "fee/Action";

export const UNIT = 1_000_000;

export async function deployERC1155(warp: Warp, opWallet: JWKInterface, initState: Erc1155State) {
    const wasmDir = "../erc1155/implementation/pkg";
    const wasmGluecode = `${wasmDir}/rust-contract.js`;
    const wasmPath = `${wasmDir}/rust-contract_bg.wasm`;

    console.log("deploying ERC1155...");

    const deployment = await warp.createContract.deploy({
        wallet: opWallet,
        initState: JSON.stringify(initState),
        wasmSrcCodeDir: wasmDir,
        wasmGlueCode: wasmGluecode,
        src: readFileSync(wasmPath),
    });

    console.log("deployed ERC1155 at:", deployment.contractTxId);

    return deployment;
}

export async function deployFee(warp: Warp, opWallet: JWKInterface, initState: FeeState) {
    const wasmDir = "../fee/implementation/pkg";
    const wasmGluecode = `${wasmDir}/rust-contract.js`;
    const wasmPath = `${wasmDir}/rust-contract_bg.wasm`;

    console.log("deploying FEE...");

    const deployment = await warp.createContract.deploy({
        wallet: opWallet,
        initState: JSON.stringify(initState),
        wasmSrcCodeDir: wasmDir,
        wasmGlueCode: wasmGluecode,
        src: readFileSync(wasmPath),
    });

    console.log("deployed FEE at:", deployment.contractTxId);

    return deployment;
}

export function createInteractor<ACTION>(
    warp: Warp,
    contract: Contract,
    defaultWallet: JWKInterface,
) {
    return async (interaction: ACTION, withWallet?: JWKInterface) => {
        if (withWallet) {
            contract.connect(withWallet);
        } else {
            contract.connect(defaultWallet);
        }

        const interactionResult = await contract.writeInteraction(interaction, { strict: true });

        await warp.testing.mineBlock();

        return interactionResult;
    };
}
