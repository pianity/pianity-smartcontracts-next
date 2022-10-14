import { readFileSync, writeFileSync } from "node:fs";

import { it, expect, beforeAll, afterAll } from "@jest/globals";
import Arlocal from "arlocal";
import {
    ArweaveGatewayInteractionsLoader,
    Contract,
    defaultCacheOptions,
    EvaluationOptions,
    GQLNodeInterface,
    LexicographicalInteractionsSorter,
    LoggerFactory,
    sleep,
    Warp,
    WarpFactory,
} from "warp-contracts";
import Arweave from "arweave";
import { JWKInterface } from "arweave/node/lib/wallet";

import { State as Erc1155State } from "erc1155/State";
import { Action as Erc1155Action } from "erc1155/Action";
import { State as PacksState } from "packs/State";
import { Action as PacksAction } from "packs/Action";
import { ContractError as PacksError } from "packs/ContractError";
import { ContractError as FeeError } from "fee/ContractError";

import {
    UNIT,
    createInteractor,
    deployContract,
    Wallet,
    range,
    dbg,
    generateWallet,
} from "@/utils";

let arlocal: Arlocal;
let warp: Warp;

let op: Wallet;
let user: Wallet;

let erc1155Contract: Contract<Erc1155State>;
let erc1155TxId: string;
let erc1155Interact: ReturnType<typeof createInteractor<Erc1155Action>>;

let packsContract: Contract<PacksState>;
let packsTxId: string;
let packsInteract: ReturnType<typeof createInteractor<PacksAction>>;

const nftPrice = 10 * UNIT;
const nftRate = 0.1 * UNIT;
const opBaseBalance = 100 * UNIT;
const userBaseBalance = 100 * UNIT;

function mintNfts(baseIds: string[]): Erc1155Action[] {
    return baseIds.flatMap((baseId, editions) =>
        range(editions * 10 || 1).map((i) => ({
            function: "mint",
            prefix: `${i + 1}`,
            ticker: baseId,
            qty: "1",
        })),
    );
}

beforeAll(async () => {
    LoggerFactory.INST.logLevel("error");
    // LoggerFactory.INST.logLevel("debug", "WASM:Rust");
    // LoggerFactory.INST.logLevel("debug", "ContractHandler");

    arlocal = new Arlocal(1986, false, `./arlocal.packs.db`, false);
    await arlocal.start();

    warp = WarpFactory.forLocal(1986, undefined, { inMemory: true, dbLocation: "/dev/null" });
    op = await warp.testing.generateWallet();
    user = await warp.testing.generateWallet();

    const erc1155InitState: Erc1155State = {
        name: "TEST-ERC1155",
        settings: {
            superOperator: op.address,
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
            "1-UNIQUE-NFT": {
                balances: {
                    [op.address]: "1",
                },
                ticker: "1-UNIQUE-NFT",
            },
            ...range(10)
                .map((i) => ({
                    balances: {
                        [op.address]: "1",
                    },
                    ticker: `${i + 1}-LEGENDARY-NFT`,
                }))
                .reduce((acc, cur) => ({ ...acc, [cur.ticker]: cur }), {}),
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
        .setEvaluationOptions({
            internalWrites: true,
            throwOnInternalWriteError: true,
            ignoreExceptions: false,
        })
        .connect(op.jwk);
    erc1155Interact = createInteractor<Erc1155Action>(warp, erc1155Contract, op.jwk);

    const packsInitState: PacksState = {
        name: "TEST-PACKS",
        settings: {
            superOperator: op.address,
            operators: [],
            erc1155: erc1155TxId,
            custodian: op.address,
            exchangeToken: "DOL",
        },
        packs: {},
    };

    packsTxId = (await deployContract(warp, op.jwk, "packs", packsInitState)).contractTxId;
    packsContract = warp
        .contract<PacksState>(packsTxId)
        .setEvaluationOptions({
            internalWrites: true,
            throwOnInternalWriteError: true,
            ignoreExceptions: false,
        })
        .connect(op.jwk);
    packsInteract = createInteractor<PacksAction>(warp, packsContract, op.jwk, {
        vrf: true,
    });

    console.log(
        `OP: ${op.address}\nUSER: ${user.address}\nERC1155: ${erc1155TxId}\nPACKS: ${packsTxId}`,
    );
}, 20_000);

afterAll(async () => {
    // await arlocal.stop();
});

it("should enable PACKS as a proxy to ERC1155", async () => {
    await erc1155Interact({
        function: "configure",
        proxies: [packsTxId],
    });

    const { state: erc1155State } = (await erc1155Contract.readState()).cachedValue;
    expect(erc1155State.settings.proxies).toEqual([packsTxId]);
});

it("should mint a pack", async () => {
    const txId = (
        await packsInteract({
            function: "mintPack",
            nfts: { legendary: ["UNIQUE-NFT", "LEGENDARY-NFT"] },
        })
    )?.originalTxId;

    const { state } = (await erc1155Contract.readState()).cachedValue;
    expect(state.tokens[`PACK-${txId}`]).toBeDefined();
}, 10_000);

it("should throw when minting a pack for the same nfts twice", async () => {
    const uniqueTicker = "UNIQUE_WHALE_NFT";
    const legendaryTicker = "LEGENDARY_WHALE_NFT";
    await erc1155Interact({
        function: "batch",
        actions: mintNfts([uniqueTicker, legendaryTicker]),
    });

    const packTicker = "WHALE";
    const expectedError: PacksError = {
        kind: "NftAlreadyPacked",
        data: [`PACK-${packTicker}`, uniqueTicker],
    };
    await expect(
        packsInteract({
            function: "batch",
            actions: [
                {
                    function: "mintPack",
                    nfts: { legendary: [uniqueTicker, legendaryTicker] },
                    ticker: packTicker,
                },
                {
                    function: "mintPack",
                    nfts: { legendary: [uniqueTicker, legendaryTicker] },
                },
            ],
        }),
    ).rejects.toEqual(expectedError);
}, 10_000);

it("should mint packs and open all of them", async () => {
    const uniqueTicker = "UNIQUE_FISH_NFT";
    const legendaryTicker = "LEGENDARY_FISH_NFT";
    await erc1155Interact({
        function: "batch",
        actions: mintNfts([uniqueTicker, legendaryTicker]),
    });

    const packTicker = "FISH";
    await packsInteract({
        function: "mintPack",
        ticker: packTicker,
        nfts: { legendary: [uniqueTicker, legendaryTicker] },
    });

    await erc1155Interact({
        function: "batch",
        actions: [
            {
                function: "mint",
                qty: "1",
                ticker: `PACK-${packTicker}`,
            },
            {
                function: "transfer",
                qty: "12",
                tokenId: `PACK-${packTicker}`,
                to: user.address,
            },
        ],
    });

    for (let i = 0; i < 11; i++) {
        await packsInteract({
            function: "openPack",
            packId: `PACK-${packTicker}`,
            owner: user.address,
        });
    }

    const noNftAvailable: PacksError = {
        kind: "NoNftAvailable",
        data: `PACK-${packTicker}`,
    };

    await expect(
        packsInteract({
            function: "openPack",
            packId: `PACK-${packTicker}`,
            owner: user.address,
        }),
    ).rejects.toEqual(noNftAvailable);
}, 70_000);
