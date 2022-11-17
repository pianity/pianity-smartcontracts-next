import { it, expect, beforeAll, afterAll } from "@jest/globals";
import Arlocal from "arlocal";
import { Contract, LoggerFactory, Warp, WarpFactory } from "warp-contracts";
import { Wallet } from "warp-contracts/lib/types/contract/testing/Testing";

import { State as Erc1155State } from "erc1155/State";
import { ContractError as Erc1155Error } from "erc1155/ContractError";
import { Action as Erc1155Action } from "erc1155/Action";
import { State as LockState } from "lock/State";
import { Action as LockAction } from "lock/Action";
import { ContractError as LockError } from "lock/ContractError";

import { UNIT, deployContract, createInteractor, Interactor } from "@/utils";

let arlocal: Arlocal;
let warp: Warp;

let op: Wallet;
let user: Wallet;

let erc1155Contract: Contract<Erc1155State, Erc1155Error>;
let erc1155TxId: string;
let erc1155Interact: Interactor<Erc1155Action, Erc1155Error>;

let lockContract: Contract<LockState, LockError>;
let lockTxId: string;
let lockInteract: Interactor<LockAction, LockError>;

const nft1BaseId = "NFT-0";
const nft1Id = `1-UNIQUE-${nft1BaseId}`;
const nft2BaseId = "NFT-1";
const nft2Id = `1-UNIQUE-${nft2BaseId}`;

const nftPrice = 10 * UNIT;
const nftRate = 0.1 * UNIT;
const opBaseBalance = 100 * UNIT;
const userBaseBalance = 100 * UNIT;

beforeAll(async () => {
    LoggerFactory.INST.logLevel("error");
    LoggerFactory.INST.logLevel("error", "WASM:Rust");
    LoggerFactory.INST.logLevel("error", "ContractHandler");

    arlocal = new Arlocal(1987, false, `./arlocal.lock.db`, false);
    await arlocal.start();
    warp = WarpFactory.forLocal(1987, undefined, { inMemory: true, dbLocation: "/dev/null" });
    op = await warp.testing.generateWallet();
    user = await warp.testing.generateWallet();

    const erc1155InitState: Erc1155State = {
        name: "TEST-ERC1155",
        settings: {
            superOperators: [op.address],
            operators: [],
            proxies: [],
            allowFreeTransfer: true,
            paused: false,
        },
        defaultToken: "DOL",
        tickerNonce: 0,
        tokens: {
            DOL: {
                ticker: "DOL",
                balances: {
                    [op.address]: `${opBaseBalance}`,
                    [user.address]: `${userBaseBalance}`,
                },
            },
        },
        approvals: {
            [user.address]: {
                [op.address]: true,
            },
        },
    };

    erc1155TxId = (await deployContract(warp, op.jwk, "erc1155", erc1155InitState)).contractTxId;
    erc1155Contract = warp
        .contract<Erc1155State, Erc1155Error>(erc1155TxId)
        .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
        .connect(op.jwk);
    erc1155Interact = createInteractor<Erc1155Action, Erc1155Error>(warp, erc1155Contract, op.jwk);

    const lockInitState: LockState = {
        name: "TEST-LOCK",
        settings: {
            superOperators: [op.address],
            operators: [],
            erc1155: erc1155TxId,
            paused: false,
        },
        vault: {},
    };

    lockTxId = (await deployContract(warp, op.jwk, "lock", lockInitState)).contractTxId;
    lockContract = warp
        .contract<LockState, LockError>(lockTxId)
        .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
        .connect(op.jwk);
    lockInteract = createInteractor<LockAction, LockError>(warp, lockContract, op.jwk);

    console.log(
        `OP: ${op.address}\nUSER: ${user.address}\nLOCK: ${lockTxId}\nERC1155: ${erc1155TxId}`,
    );
}, 20_000);

afterAll(async () => {
    await arlocal.stop();
});

it("should do stuff", async () => {
    await erc1155Interact(
        {
            function: "setApprovalForAll",
            approved: true,
            operator: lockTxId,
        },
        { wallet: user.jwk },
    );

    await lockInteract(
        {
            function: "transferLocked",
            tokenId: "DOL",
            duration: 2,
            qty: `${100 * UNIT}`,
            to: op.address,
        },
        // { wallet: user.jwk },
    );

    await warp.testing.mineBlock();
    await warp.testing.mineBlock();

    {
        const { state } = (await erc1155Contract.readState()).cachedValue;
        console.log(JSON.stringify(state, null, 2));
    }
    {
        const { state } = (await lockContract.readState()).cachedValue;
        console.log(JSON.stringify(state, null, 2));
    }

    await lockInteract(
        {
            function: "unlock",
        },
        { wallet: user.jwk },
    );

    {
        const { state } = (await erc1155Contract.readState()).cachedValue;
        console.log(JSON.stringify(state, null, 2));
    }
    {
        const { state } = (await lockContract.readState()).cachedValue;
        console.log(JSON.stringify(state, null, 2));
    }
});

// it("should activate the lock contract on the Erc1155 one", async () => {
//     await erc1155Interact({
//         function: "configure",
//         proxies: [lockTxId],
//     });
//
//     const { state } = (await erc1155Contract.readState()).cachedValue;
//     expect(state.settings.proxies).toEqual([lockTxId]);
// });
