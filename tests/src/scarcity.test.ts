import { it, expect, beforeAll, afterAll } from "@jest/globals";
import Arlocal from "arlocal";
import { Contract, LoggerFactory, Warp, WarpFactory } from "warp-contracts";
import { Wallet } from "warp-contracts/lib/types/contract/testing/Testing";

import { State as Erc1155State } from "erc1155/State";
import { Action as Erc1155Action } from "erc1155/Action";
import { ContractError as Erc1155Error } from "erc1155/ContractError";
import { State as ScarcityState } from "scarcity/State";
import { Action as ScarcityAction } from "scarcity/Action";
import { ContractError as ScarcityError } from "scarcity/ContractError";

import { UNIT, deployContract, createInteractor, expectOk, expectError } from "@/utils";

let arlocal: Arlocal;
let warp: Warp;

let op: Wallet;
let user: Wallet;

let erc1155Contract: Contract<Erc1155State, Erc1155Error>;
let erc1155TxId: string;
let erc1155Interact: ReturnType<typeof createInteractor<Erc1155Action, Erc1155State, Erc1155Error>>;

let scarcityContract: Contract<ScarcityState, ScarcityError>;
let scarcityTxId: string;
let scarcityInteract: ReturnType<
    typeof createInteractor<ScarcityAction, ScarcityState, ScarcityError>
>;

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
    // LoggerFactory.INST.logLevel("error", "WASM:Rust");
    // LoggerFactory.INST.logLevel("error", "ContractHandler");

    arlocal = new Arlocal(1985, false, `./arlocal.scarcity.db`, false);
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
        .contract<Erc1155State, Erc1155Error>(erc1155TxId)
        .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
        .connect(op.jwk);
    erc1155Interact = createInteractor<Erc1155Action, Erc1155State, Erc1155Error>(
        warp,
        erc1155Contract,
        op.jwk,
    );

    const scarcityInitState: ScarcityState = {
        name: "TEST-SCARCITY",
        settings: {
            superOperators: [op.address],
            operators: [],
            erc1155: erc1155TxId,
            custodian: op.address,
            exchangeToken: "DOL",
        },
        nfts: {},
    };

    scarcityTxId = (await deployContract(warp, op.jwk, "scarcity", scarcityInitState)).contractTxId;
    scarcityContract = warp
        .contract<ScarcityState, ScarcityError>(scarcityTxId)
        .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
        .connect(op.jwk);
    scarcityInteract = createInteractor<ScarcityAction, ScarcityState, ScarcityError>(
        warp,
        scarcityContract,
        op.jwk,
    );

    console.log(
        `OP: ${op.address}\nUSER: ${user.address}\nSCARCITY: ${scarcityTxId}\nERC1155:`,
        erc1155TxId,
    );
}, 25_000);

afterAll(async () => {
    await arlocal.stop();
});

it("should activate the Scarcity contract on the Erc1155 one", async () => {
    await erc1155Interact({
        function: "configure",
        proxies: [scarcityTxId],
    });

    const { state } = (await erc1155Contract.readState()).cachedValue;
    expect(state.settings.proxies).toEqual([scarcityTxId]);
});

it("should mint a free NFT and distribute it to a unknown address", async () => {
    const nftBaseId = "PANTERA";
    const nftId = `1-UNIQUE-${nftBaseId}`;
    const unknownAddress = "unknown-address-1243132423";

    await scarcityInteract({
        function: "mintNft",
        scarcity: "unique",
        fees: { [op.address]: 1_000_000 },
        rate: 1_000_000,
        ticker: nftBaseId,
    });

    await scarcityInteract({
        function: "transfer",
        to: unknownAddress,
        nftId,
        price: "0",
    });

    const { state } = (await erc1155Contract.readState()).cachedValue;
    expect(state.tokens[nftId].balances[unknownAddress]).toEqual("1");
});

it("should attach fees to an NFT", async () => {
    const fees: ScarcityState["nfts"][0] = {
        baseId: nft1BaseId,
        fees: {
            [op.address]: UNIT,
        },
        rate: nftRate,
    };

    await scarcityInteract({
        function: "createFee",
        ...fees,
        nftBaseId: nft1BaseId,
    });

    const { state } = (await scarcityContract.readState()).cachedValue;
    expect(state.nfts[nft1BaseId]).toEqual(fees);
});

it("should return correct error type", async () => {
    const interaction = await scarcityInteract({
        function: "transfer",
        nftId: nft1Id,
        to: user.address,
        price: `${opBaseBalance + UNIT}`,
    });

    expectError(interaction?.type);

    const notEnoughBalanceError: ScarcityError = {
        kind: "Erc1155Error",
        data: {
            kind: "ContractError",
            data: {
                kind: "OwnerBalanceNotEnough",
                data: user.address,
            },
        },
    };

    expect(interaction.error).toEqual(notEnoughBalanceError);
});

it("op should sell the NFT and pay the shareholders", async () => {
    await scarcityInteract({
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
    const interaction = await scarcityInteract({
        function: "mintNft",
        scarcity: "legendary",
        fees: {
            [op.address]: UNIT,
        },
        rate: nftRate,
    });

    expectOk(interaction?.type);

    const nftBaseId = interaction.originalTxId;

    const { state: scarcityState } = (await scarcityContract.readState()).cachedValue;
    expect(scarcityState.nfts[nftBaseId]).toBeDefined();

    const { state: erc1155State } = (await erc1155Contract.readState()).cachedValue;

    for (let i = 0; i < 10; i++) {
        const nftId = `${i + 1}-LEGENDARY-${nftBaseId}`;
        expect(erc1155State.tokens[nftId]).toBeDefined();
    }

    expect(erc1155State.tokens[`11-LEGENDARY-${nftBaseId}`]).toBeUndefined();
});

it("should mint with a custom ticker", async () => {
    const ticker = "CUSTOM_TICKER";

    await scarcityInteract({
        function: "mintNft",
        scarcity: "legendary",
        fees: {
            [op.address]: UNIT,
        },
        rate: nftRate,
        ticker,
    });

    const { state: scarcityState } = (await scarcityContract.readState()).cachedValue;
    expect(scarcityState.nfts[ticker]).toBeDefined();

    const { state: erc1155State } = (await erc1155Contract.readState()).cachedValue;

    for (let i = 0; i < 10; i++) {
        const tokenId = `${i + 1}-LEGENDARY-${ticker}`;

        expect(erc1155State.tokens[tokenId]).toBeDefined();
    }

    expect(erc1155State.tokens[`11-LEGENDARY-${ticker}`]).toBeUndefined();
});

// TODO: Test the case where user buys an NFT that doesn't have an edition count of 1, 10, 100 or
// 1000. (there is an NFT with 50 editions in the old contract).
