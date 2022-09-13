import { readFileSync, writeFileSync } from "node:fs";

import ArLocal from "arlocal";
import Arweave from "arweave";
import { JWKInterface } from "arweave/node/lib/wallet";

import { Contract, LoggerFactory, Warp, WarpFactory } from "warp-contracts";
// import { LoggerFactory, Warp, WarpNodeFactory } from "warp-contracts";

import { State as Erc1155State } from "erc1155/State";
import { Action as Erc1155Action } from "erc1155/Action";
import { State as FeeState } from "fee/State";
import { Action as FeeAction } from "fee/Action";

LoggerFactory.INST.logLevel("error");
LoggerFactory.INST.logLevel("debug", "WASM:Rust");
LoggerFactory.INST.logLevel("debug", "ContractHandler");

const PROJECT_PATH = "./src/testRustContracts";

// const OP_ADDRESS = "iZy3I69Ms58VWw2gV0LesHE23MDnQiFD6FC-dyhyiwY";
// const OP_WALLET = JSON.parse(readFileSync(`${PROJECT_PATH}/${OP_ADDRESS}.json`).toString());

// const OP_ADDRESS = "kX4Z5hj5znsyxwAIG9yvOf40u3EXTnGv-jL9ALTqPSg";
// const OP_WALLET = JSON.parse(readFileSync("/home/noom/Wallets/others/arbank.json").toString());

const UNIT = 1_000_000;

function createSmartweaveEnv(arweave: Arweave) {
    // const client = WarpNodeFactory.forTesting(arweave);
    const client = WarpFactory.forLocal();
    // .useWarpGateway({ confirmed: true })
    // .useArweaveGateway()
    // .build();

    return client;
}

async function genWallet(arweave: Arweave): Promise<{ wallet: JWKInterface; address: string }> {
    const wallet = await arweave.wallets.generate();
    const address = await arweave.wallets.getAddress(wallet);

    return { wallet, address };
}

async function fund(arweave: Arweave, address: string, amount = "999999999999999999") {
    console.log(`funding ${address} with ${amount}...`);
    await arweave.api.get(`/mint/${address}/${amount}`);
}

async function mine(arweave: Arweave) {
    console.log("mining...");
    await arweave.api.get("/mine");
}

async function deployERC1155(client: Warp, opWallet: JWKInterface, opAddress: string) {
    const wasmDir = "../erc1155/implementation/pkg";
    const wasmGluecode = `${wasmDir}/rust-contract.js`;
    const wasmPath = `${wasmDir}/rust-contract_bg.wasm`;

    const initState: Erc1155State = {
        name: "TEST-ERC1155",
        settings: {
            superOperator: opAddress,
            operators: [],
            transferProxies: [],
        },
        tokens: {
            DOL: {
                ticker: "DOL",
                balances: {
                    [opAddress]: `${100 * UNIT}`,
                },
            },
        },
        approvals: {},
    };

    console.log("deploying ERC1155...");

    const deployment = await client.createContract.deploy({
        wallet: opWallet,
        initState: JSON.stringify(initState),
        wasmSrcCodeDir: wasmDir,
        wasmGlueCode: wasmGluecode,
        src: readFileSync(wasmPath),
    });

    console.log("deployed ERC1155 at:", deployment.contractTxId);

    return deployment;
}

async function deployFee(
    client: Warp,
    erc1155TxId: string,
    opWallet: JWKInterface,
    opAddress: string,
) {
    const wasmDir = "../fee/implementation/pkg";
    const wasmGluecode = `${wasmDir}/rust-contract.js`;
    const wasmPath = `${wasmDir}/rust-contract_bg.wasm`;

    const initState: FeeState = {
        name: "TEST-FEE",
        settings: {
            superOperator: opAddress,
            operators: [],
            authorizedAddresses: [],
            erc1155: erc1155TxId,
            custodian: opAddress,
            exchangeToken: "DOL",
        },
        tokens: {},
    };

    console.log("deploying FEE...");

    const deployment = await client.createContract.deploy({
        wallet: opWallet,
        initState: JSON.stringify(initState),
        wasmSrcCodeDir: wasmDir,
        wasmGlueCode: wasmGluecode,
        src: readFileSync(wasmPath),
    });

    console.log("deployed FEE at:", deployment.contractTxId);

    return deployment;
}

// async function deployTest(client: Warp) {
//     console.log("deploying...");
//
//     function handle(state: { i: 0 }, action: any) {
//         console.log("Hello world");
//         state.i += 1;
//         return { state };
//     }
//
//     const deployment = await client.createContract.deploy({
//         wallet: OP_WALLET,
//         initState: JSON.stringify({ i: 0 }),
//         src: `export ${handle.toString()}`,
//     });
//
//     console.log("deployment", deployment);
// }

function createInteractor<ACTION>(contract: Contract, defaultWallet: JWKInterface) {
    return (interaction: ACTION, withWallet?: JWKInterface) => {
        if (withWallet) {
            contract.connect(withWallet);
        } else {
            contract.connect(defaultWallet);
        }

        return contract.writeInteraction(interaction, { strict: true });
    };
}

export default async function testRustContracts() {
    const arweave = Arweave.init({
        host: "localhost",
        port: 1984,
        protocol: "http",
    });

    const client = createSmartweaveEnv(arweave);

    const arlocal = new ArLocal(1984, false, `${PROJECT_PATH}/arlocal.db`, false);
    await arlocal.start();

    console.log("Generate op...");
    const op = await genWallet(arweave);
    await fund(arweave, op.address);
    await mine(arweave);

    await fund(arweave, op.address);
    await mine(arweave);

    const { contractTxId: erc1155TxId } = await deployERC1155(client, op.wallet, op.address);
    await mine(arweave);

    const { contractTxId: feeTxId } = await deployFee(client, erc1155TxId, op.wallet, op.address);
    await mine(arweave);

    const erc1155Contract = client
        .contract<Erc1155State>(erc1155TxId)
        .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
        .connect(op.wallet);
    const feeContract = client
        .contract<FeeState>(feeTxId)
        .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
        .connect(op.wallet);

    const erc1155Interact = createInteractor<Erc1155Action>(erc1155Contract, op.wallet);
    const feeInteract = createInteractor<FeeAction>(feeContract, op.wallet);

    console.log("Activate the Fee proxy contract...");
    await erc1155Interact({
        function: "configure",
        transferProxies: [feeTxId, op.address],
    });
    await mine(arweave);

    console.log("Mint an NFT...");
    const mintResponse = await erc1155Interact({
        function: "mint",
        prefix: "NFT",
        qty: "1",
    });
    await mine(arweave);

    const tokenId = `NFT-${mintResponse?.originalTxId}`;

    console.log("Create the fees...");
    await feeInteract({
        function: "createFee",
        tokenId,
        fees: {
            [op.address]: UNIT,
        },
        rate: UNIT * 0.1,
    });
    await mine(arweave);

    console.log("Generate a user...");
    const user = await genWallet(arweave);
    await fund(arweave, user.address);
    await mine(arweave);

    console.log("Transfer some DOL to user...");
    await erc1155Interact({
        function: "transfer",
        from: op.address,
        tokenId: "DOL",
        to: user.address,
        qty: `${5 * UNIT}`,
    });
    await mine(arweave);

    console.log("Approve op for user...");
    await erc1155Interact(
        {
            function: "setApprovalForAll",
            operator: op.address,
            approved: true,
        },
        user.wallet,
    );

    console.log("Buy the nft...");
    await feeInteract({
        function: "transfer",
        to: user.address,
        price: `${1 * UNIT}`,
        tokenId,
    });
    await mine(arweave);

    const { state: erc1155State } = (await erc1155Contract.readState()).cachedValue;
    const { state: feeState } = (await feeContract.readState()).cachedValue;
    console.log(JSON.stringify(erc1155State, undefined, 2));
    console.log(JSON.stringify(feeState, undefined, 2));

    for (const token of Object.values(erc1155State.tokens)) {
        console.log(token);
    }

    console.log("stopping...");

    await arlocal.stop();
}

testRustContracts();
