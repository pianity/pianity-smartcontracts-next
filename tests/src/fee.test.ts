import { it, expect, beforeAll, afterAll } from "@jest/globals";
import Arlocal from "arlocal";
import { Contract, LoggerFactory, Warp, WarpFactory } from "warp-contracts";
import { Wallet } from "warp-contracts/lib/types/contract/testing/Testing";

import { State as Erc1155State } from "erc1155/State";
import { Action as Erc1155Action } from "erc1155/Action";
import { State as FeeState } from "fee/State";
import { Action as FeeAction } from "fee/Action";
import { ContractError as FeeError } from "fee/ContractError";

import { UNIT, deployContract, createInteractor } from "@/utils";

let arlocal: Arlocal;
let warp: Warp;

let op: Wallet;
let user: Wallet;

let erc1155Contract: Contract<Erc1155State>;
let erc1155TxId: string;
let erc1155Interact: ReturnType<typeof createInteractor<Erc1155Action>>;

let feeContract: Contract<FeeState>;
let feeTxId: string;
let feeInteract: ReturnType<typeof createInteractor<FeeAction>>;

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

    arlocal = new Arlocal(1985, false, `./arlocal.fee.db`, false);
    await arlocal.start();
    warp = WarpFactory.forLocal(1985, undefined, { inMemory: true, dbLocation: "/dev/null" });
    op = await warp.testing.generateWallet();
    user = await warp.testing.generateWallet();

    const erc1155InitState: Erc1155State = {
        name: "TEST-ERC1155",
        settings: {
            superOperators: [op.address],
            operators: [],
            proxies: [],
            allowFreeTransfer: true,
        },
        tokens: {
            DOL: {
                ticker: "DOL",
                balances: {
                    [op.address]: `${opBaseBalance}`,
                    [user.address]: `${userBaseBalance}`,
                },
            },
            [nft1Id]: {
                ticker: nft1Id,
                balances: {
                    [op.address]: "1",
                },
            },
            [nft2Id]: {
                ticker: nft2Id,
                balances: {
                    [op.address]: "1",
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
        .contract<Erc1155State>(erc1155TxId)
        .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
        .connect(op.jwk);
    erc1155Interact = createInteractor<Erc1155Action>(warp, erc1155Contract, op.jwk);

    const feeInitState: FeeState = {
        name: "TEST-FEE",
        settings: {
            superOperators: [op.address],
            operators: [],
            erc1155: erc1155TxId,
            custodian: op.address,
            exchangeToken: "DOL",
        },
        nfts: {},
    };

    feeTxId = (await deployContract(warp, op.jwk, "fee", feeInitState)).contractTxId;
    feeContract = warp
        .contract<FeeState>(feeTxId)
        .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
        .connect(op.jwk);
    feeInteract = createInteractor<FeeAction>(warp, feeContract, op.jwk);

    console.log(
        `OP: ${op.address}\nUSER: ${user.address}\nFEE: ${feeTxId}\nERC1155: ${erc1155TxId}`,
    );
}, 20_000);

afterAll(async () => {
    await arlocal.stop();
});

it("should activate the Fee contract on the Erc1155 one", async () => {
    await erc1155Interact({
        function: "configure",
        proxies: [feeTxId],
    });

    const { state } = (await erc1155Contract.readState()).cachedValue;
    expect(state.settings.proxies).toEqual([feeTxId]);
});

it("should mint a free NFT and distribute it to a unknown address", async () => {
    const nftBaseId = "PANTERA";
    const nftId = `1-UNIQUE-${nftBaseId}`;
    const unknownAddress = "unknown-address-1243132423";

    await feeInteract({
        function: "mintNft",
        scarcity: "unique",
        fees: { [op.address]: 1_000_000 },
        rate: 1_000_000,
        ticker: nftBaseId,
    });

    await feeInteract({
        function: "transfer",
        to: unknownAddress,
        nftId,
        price: "0",
    });

    const { state } = (await erc1155Contract.readState()).cachedValue;
    expect(state.tokens[nftId].balances[unknownAddress]).toEqual("1");
});

it("should attach fees to an NFT", async () => {
    const fees: FeeState["nfts"][0] = {
        baseId: nft1BaseId,
        fees: {
            [op.address]: UNIT,
        },
        rate: nftRate,
    };

    await feeInteract({
        function: "createFee",
        ...fees,
        nftBaseId: nft1BaseId,
    });

    const { state } = (await feeContract.readState()).cachedValue;
    expect(state.nfts[nft1BaseId]).toEqual(fees);
});

it("should throw correct error type", async () => {
    let error;

    try {
        await feeInteract({
            function: "transfer",
            nftId: nft1Id,
            to: user.address,
            price: `${opBaseBalance + UNIT}`,
        });
    } catch (caughtError) {
        error = caughtError;
    }

    const notEnoughBalanceError: FeeError = {
        kind: "Erc1155Error",
        data: {
            kind: "ContractError",
            data: {
                kind: "OwnerBalanceNotEnough",
                data: user.address,
            },
        },
    };

    expect(error).toEqual(notEnoughBalanceError);
});

it("op should sell the NFT and pay the shareholders", async () => {
    await feeInteract({
        function: "transfer",
        to: user.address,
        nftId: nft1Id,
        price: `${nftPrice}`,
    });

    const { state } = (await erc1155Contract.readState()).cachedValue;
    expect(state.tokens[nft1Id].balances[user.address]).toBe("1");
    expect(state.tokens.DOL.balances[op.address]).toBe(`${opBaseBalance + nftPrice}`);
    expect(state.tokens.DOL.balances[user.address]).toBe(`${userBaseBalance - nftPrice}`);
});

it("should mint nft", async () => {
    const nftBaseId = (
        await feeInteract({
            function: "mintNft",
            scarcity: "legendary",
            fees: {
                [op.address]: UNIT,
            },
            rate: nftRate,
        })
    )?.originalTxId;

    expect(nftBaseId).toBeDefined();
    if (!nftBaseId) {
        throw new Error("No nftBaseId");
    }

    const { state: feeState } = (await feeContract.readState()).cachedValue;
    expect(feeState.nfts[nftBaseId]).toBeDefined();

    const { state: erc1155State } = (await erc1155Contract.readState()).cachedValue;

    for (let i = 0; i < 10; i++) {
        const nftId = `${i + 1}-LEGENDARY-${nftBaseId}`;
        expect(erc1155State.tokens[nftId]).toBeDefined();
    }

    expect(erc1155State.tokens[`11-LEGENDARY-${nftBaseId}`]).toBeUndefined();
});

it("should mint with a custom ticker", async () => {
    const ticker = "CUSTOM_TICKER";

    await feeInteract({
        function: "mintNft",
        scarcity: "legendary",
        fees: {
            [op.address]: UNIT,
        },
        rate: nftRate,
        ticker,
    });

    const { state: feeState } = (await feeContract.readState()).cachedValue;
    expect(feeState.nfts[ticker]).toBeDefined();

    const { state: erc1155State } = (await erc1155Contract.readState()).cachedValue;

    for (let i = 0; i < 10; i++) {
        const tokenId = `${i + 1}-LEGENDARY-${ticker}`;

        expect(erc1155State.tokens[tokenId]).toBeDefined();
    }

    expect(erc1155State.tokens[`11-LEGENDARY-${ticker}`]).toBeUndefined();
});

// TODO: Test the case where user buys an NFT that doesn't have an edition count of 1, 10, 100 or
// 1000. (there is an NFT with 50 editions in the old contract).
