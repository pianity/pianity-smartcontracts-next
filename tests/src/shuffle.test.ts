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
import { State as ShuffleState } from "shuffle/State";
import { Action as ShuffleAction } from "shuffle/Action";
import { ContractError as ShuffleError } from "shuffle/ContractError";

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

let shuffleContract: Contract<ShuffleState>;
let shuffleTxId: string;
let shuffleInteract: ReturnType<typeof createInteractor<ShuffleAction>>;

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

    arlocal = new Arlocal(1986, false, `./arlocal.shuffle.db`, false);
    await arlocal.start();

    warp = WarpFactory.forLocal(1986, undefined, { inMemory: true, dbLocation: "/dev/null" });
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

    const shuffleInitState: ShuffleState = {
        name: "TEST-SHUFFLES",
        settings: {
            superOperators: [op.address],
            operators: [],
            erc1155: erc1155TxId,
            custodian: op.address,
            exchangeToken: "DOL",
        },
        shuffles: {},
    };

    shuffleTxId = (await deployContract(warp, op.jwk, "shuffle", shuffleInitState)).contractTxId;
    shuffleContract = warp
        .contract<ShuffleState>(shuffleTxId)
        .setEvaluationOptions({
            internalWrites: true,
            throwOnInternalWriteError: true,
            ignoreExceptions: false,
        })
        .connect(op.jwk);
    shuffleInteract = createInteractor<ShuffleAction>(warp, shuffleContract, op.jwk, {
        vrf: true,
    });

    console.log(
        `OP: ${op.address}\nUSER: ${user.address}\nERC1155: ${erc1155TxId}\nSHUFFLE: ${shuffleTxId}`,
    );
}, 20_000);

afterAll(async () => {
    // await arlocal.stop();
});

it("should enable SHUFFLES as a proxy to ERC1155", async () => {
    await erc1155Interact({
        function: "configure",
        proxies: [shuffleTxId],
    });

    const { state: erc1155State } = (await erc1155Contract.readState()).cachedValue;
    expect(erc1155State.settings.proxies).toEqual([shuffleTxId]);
});

it("should mint a shuffle", async () => {
    const txId = (
        await shuffleInteract({
            function: "mintShuffle",
            nfts: { legendary: ["UNIQUE-NFT", "LEGENDARY-NFT"] },
        })
    )?.originalTxId;

    const { state } = (await erc1155Contract.readState()).cachedValue;
    expect(state.tokens[`SHUFFLE-${txId}`]).toBeDefined();
}, 10_000);

it("should throw when minting a shuffle for the same nfts twice", async () => {
    const uniqueTicker = "UNIQUE_WHALE_NFT";
    const legendaryTicker = "LEGENDARY_WHALE_NFT";
    await erc1155Interact({
        function: "batch",
        actions: mintNfts([uniqueTicker, legendaryTicker]),
    });

    const shuffleTicker = "WHALE";
    const expectedError: ShuffleError = {
        kind: "NftAlreadyInAShuffle",
        data: [`SHUFFLE-${shuffleTicker}`, uniqueTicker],
    };
    await expect(
        shuffleInteract({
            function: "batch",
            actions: [
                {
                    function: "mintShuffle",
                    nfts: { legendary: [uniqueTicker, legendaryTicker] },
                    ticker: shuffleTicker,
                },
                {
                    function: "mintShuffle",
                    nfts: { legendary: [uniqueTicker, legendaryTicker] },
                },
            ],
        }),
    ).rejects.toEqual(expectedError);
}, 10_000);

it("should mint shuffles and open all of them", async () => {
    const uniqueTicker = "UNIQUE_FISH_NFT";
    const legendaryTicker = "LEGENDARY_FISH_NFT";
    await erc1155Interact({
        function: "batch",
        actions: mintNfts([uniqueTicker, legendaryTicker]),
    });

    const shuffleTicker = "FISH";
    await shuffleInteract({
        function: "mintShuffle",
        ticker: shuffleTicker,
        nfts: { legendary: [uniqueTicker, legendaryTicker] },
    });

    await erc1155Interact({
        function: "batch",
        actions: [
            {
                function: "mint",
                qty: "1",
                ticker: `SHUFFLE-${shuffleTicker}`,
            },
            {
                function: "transfer",
                qty: "12",
                tokenId: `SHUFFLE-${shuffleTicker}`,
                to: user.address,
            },
        ],
    });

    for (let i = 0; i < 11; i++) {
        await shuffleInteract({
            function: "openShuffle",
            shuffleId: `SHUFFLE-${shuffleTicker}`,
            owner: user.address,
        });
    }

    const noNftAvailable: ShuffleError = {
        kind: "NoNftAvailable",
        data: `SHUFFLE-${shuffleTicker}`,
    };

    await expect(
        shuffleInteract({
            function: "openShuffle",
            shuffleId: `SHUFFLE-${shuffleTicker}`,
            owner: user.address,
        }),
    ).rejects.toEqual(noNftAvailable);
}, 70_000);
