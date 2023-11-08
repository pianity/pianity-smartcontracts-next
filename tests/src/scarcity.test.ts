import { it, expect, beforeAll, afterAll } from "vitest";
import Arlocal from "arlocal";
import { Contract, LoggerFactory, Warp, WarpFactory } from "warp-contracts";
import { Wallet } from "warp-contracts/lib/types/contract/testing/Testing";

import { Parameters as Erc1155State } from "erc1155/State";
import { Action as Erc1155Action } from "erc1155/Action";
import { ContractError as Erc1155Error } from "erc1155/ContractError";
import { ReadResponse as Erc1155ReadResponse } from "erc1155/ReadResponse";
import { Parameters as ScarcityState } from "scarcity/State";
import { Action as ScarcityAction } from "scarcity/Action";
import { ContractError as ScarcityError } from "scarcity/ContractError";
import { ReadResponse as ScarcityReadResponse } from "scarcity/ReadResponse";
// import { State as ShuffleState } from "shuffle/State";
// import { Action as ShuffleAction } from "shuffle/Action";
// import { ContractError as ShuffleError } from "shuffle/ContractError";

import {
    // UNIT,
    deployContract,
    createInteractor,
    // expectOk,
    // expectError,
    Interactor,
    generateWallet,
    Viewer,
    createViewer,
    expectError,
    expectOk,
} from "@/utils";
import { DeployPlugin } from "warp-contracts-plugin-deploy";

const UNIT = 1_000_000;

let arlocal: Arlocal;
let warp: Warp;

let op: Wallet;
let user: Wallet;
// let shuffleBuyer: Wallet;

let erc1155Contract: Contract<Erc1155State>;
let erc1155TxId: string;
let erc1155Interact: Interactor<Erc1155Action, Erc1155Error>;
let erc1155View: Viewer<Erc1155Action, Erc1155ReadResponse, Erc1155State, Erc1155Error>;

// let shuffleContract: Contract<ShuffleState, ShuffleError>;
// let shuffleTxId: string;
// let shuffleInteract: Interactor<ShuffleAction, ShuffleError>;

let scarcityContract: Contract<ScarcityState>;
let scarcityTxId: string;
let scarcityInteract: Interactor<ScarcityAction, ScarcityError>;
let scarcityView: Viewer<ScarcityAction, ScarcityReadResponse, ScarcityState, ScarcityError>;

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
    LoggerFactory.INST.logLevel("debug", "WASM:Rust");
    // LoggerFactory.INST.logLevel("debug", "ContractHandler");

    arlocal = new Arlocal(1985, false, `./arlocal.scarcity.db`, false);
    await arlocal.start();
    warp = WarpFactory.forLocal(1985, undefined, { inMemory: true, dbLocation: "/dev/null" }).use(
        new DeployPlugin(),
    );
    op = await generateWallet();
    user = await generateWallet();
    // TODO: Should use `user` instead and fix the tests to correctly deduce user's balance from
    // erc1155's state when doing transfers
    // shuffleBuyer = await generateWallet();

    await warp.testing.addFunds(op.jwk);
    await warp.testing.addFunds(user.jwk);

    const erc1155InitState: Erc1155State = {
        name: "TEST-ERC1155",
        canEvolve: false,
        initialState: {
            settings: {
                defaultToken: "DOL",
                superOperators: [op.address],
                operators: [],
                proxies: [],
                allowFreeTransfer: true,
                paused: false,
                canEvolve: false,
            },
            tickerNonce: 0,
            tokens: {
                DOL: {
                    ticker: "DOL",
                    balances: {
                        [op.address]: `${opBaseBalance}`,
                        [user.address]: `${userBaseBalance}`,
                        // [shuffleBuyer.address]: `${100 * UNIT}`,
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
                    approves: {
                        [op.address]: true,
                    },
                },
                // [shuffleBuyer.address]: {
                //     [op.address]: true,
                // },
            },
        },
    };
    erc1155TxId = (await deployContract(warp, op.jwk, "erc1155", erc1155InitState)).contractTxId;
    erc1155Contract = warp
        .contract<Erc1155State>(erc1155TxId)
        .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
        .connect(op.jwk);
    erc1155Interact = createInteractor<Erc1155Action, Erc1155Error>(warp, erc1155Contract, op.jwk);
    erc1155View = createViewer<Erc1155Action, Erc1155ReadResponse, Erc1155State, Erc1155Error>(
        erc1155Contract,
    );

    // const shuffleInitState: ShuffleState = {
    //     name: "TEST-SHUFFLES",
    //     settings: {
    //         superOperators: [op.address],
    //         operators: [],
    //         erc1155: erc1155TxId,
    //         custodian: op.address,
    //         boostCap: 1,
    //         boostPriceModifier: 1,
    //         boostToken: "DOL",
    //         paused: false,
    //     },
    //     shuffles: {},
    // };
    // shuffleTxId = (await deployContract(warp, op.jwk, "shuffle", shuffleInitState)).contractTxId;
    // shuffleContract = warp
    //     .contract<ShuffleState, ShuffleError>(shuffleTxId)
    //     .setEvaluationOptions({
    //         internalWrites: true,
    //         throwOnInternalWriteError: false,
    //         ignoreExceptions: false,
    //     })
    //     .connect(op.jwk);
    // shuffleInteract = createInteractor<ShuffleAction, ShuffleError>(warp, shuffleContract, op.jwk, {
    //     vrf: true,
    // });

    const scarcityInitState: ScarcityState = {
        name: "TEST-SCARCITY",
        canEvolve: false,
        initialState: {
            settings: {
                superOperators: [op.address],
                operators: [],
                erc1155: erc1155TxId,
                custodian: op.address,
                paused: false,
            },
            attachedRoyalties: {},
        },
    };
    scarcityTxId = (await deployContract(warp, op.jwk, "scarcity", scarcityInitState)).contractTxId;
    scarcityContract = warp
        .contract<ScarcityState>(scarcityTxId)
        .setEvaluationOptions({ internalWrites: true, throwOnInternalWriteError: false })
        .connect(op.jwk);
    scarcityInteract = createInteractor<ScarcityAction, ScarcityError>(
        warp,
        scarcityContract,
        op.jwk,
    );
    scarcityView = createViewer<ScarcityAction, ScarcityReadResponse, ScarcityState, ScarcityError>(
        scarcityContract,
    );

    console.log(
        "OP:",
        op.address,
        "\nUSER:",
        user.address,
        "\nERC1155:",
        erc1155TxId,
        // "\nSHUFFLE:",
        // shuffleTxId,
        "\nSCARCITY:",
        scarcityTxId,
    );
}, 25_000);

afterAll(async () => {
    await arlocal.stop();
});

it("initialize Erc1155", async () => {
    const stateBefore = (await erc1155Contract.readState()).cachedValue.state;
    expect(stateBefore.initialState).toBeTruthy();

    expectOk(await erc1155Interact({ function: "initialize" }));

    const stateAfter = (await erc1155Contract.readState()).cachedValue.state;
    expect(stateAfter.initialState).toBeNull();
});

it("fail if Scarcity is not initialized", async () => {
    const stateBefore = (await scarcityContract.readState()).cachedValue.state;
    expect(stateBefore.initialState).toBeTruthy();

    const result = await scarcityInteract({
        function: "mintNft",
        scarcity: "unique",
        rate: 1_000_000,
        royalties: { [op.address]: 1_000_000 },
    });

    expectError(result, { kind: "ContractUninitialized" });
});

it("initialize Scarcity", async () => {
    const stateBefore = (await scarcityContract.readState()).cachedValue.state;
    expect(stateBefore.initialState).toBeTruthy();

    expectOk(await scarcityInteract({ function: "initialize" }));

    const stateAfter = (await scarcityContract.readState()).cachedValue.state;
    expect(stateAfter.initialState).toBeNull();
});

it("fail if already initialized", async () => {
    const stateBefore = (await scarcityContract.readState()).cachedValue.state;
    expect(stateBefore.initialState).toBeNull();

    expectError(await scarcityInteract({ function: "initialize" }), {
        kind: "ContractAlreadyInitialized",
    });
});

it("activate the Scarcity contract as a proxy of Erc1155", async () => {
    expectOk(
        await erc1155Interact({
            function: "configure",
            proxies: [scarcityTxId],
        }),
    );

    const settings = await erc1155View({ function: "readSettings" });
    expectOk(settings);
    expect(settings.result.proxies).toEqual([scarcityTxId]);

    // const { state } = (await erc1155Contract.readState()).cachedValue;
    // expect(state.settings.proxies).toEqual([scarcityTxId, shuffleTxId]);
});

it("mint a free NFT and distribute it to a unknown address", async () => {
    const nftBaseId = "PANTERA";
    const nftId = `1-UNIQUE-${nftBaseId}`;
    const unknownAddress = "unknown-address-1243132423";

    expectOk(
        await scarcityInteract({
            function: "mintNft",
            scarcity: "unique",
            royalties: { [op.address]: 1_000_000 },
            rate: 1_000_000,
            baseId: nftBaseId,
        }),
    );

    expectOk(
        await scarcityInteract({
            function: "transfer",
            from: op.address,
            to: unknownAddress,
            tokenId: nftId,
            price: "0",
        }),
    );

    const unknownAddressBalance = await erc1155View({
        function: "balanceOf",
        target: unknownAddress,
        tokenId: nftId,
    });

    expectOk(unknownAddressBalance);

    expect(unknownAddressBalance.result.balance).toEqual("1");
});

it("attach fees to an NFT", async () => {
    const fees: NonNullable<ScarcityState["initialState"]>["attachedRoyalties"][0] = {
        baseId: nft1BaseId,
        royalties: {
            [op.address]: UNIT,
        },
        rate: nftRate,
    };

    const params = {
        function: "attachRoyalties",
        ...fees,
        baseId: nft1BaseId,
    } as const;

    expectOk(await scarcityInteract(params));

    const royalties = await scarcityView({ function: "getAttachedRoylaties", baseId: nft1BaseId });
    expectOk(royalties);
    expect(royalties.result).toEqual(fees);
});

it("return correct error type on bad tranfer", async () => {
    const result = await scarcityInteract({
        function: "transfer",
        tokenId: nft1Id,
        from: op.address,
        to: user.address,
        price: `${opBaseBalance + UNIT}`,
    });

    expectError(result, {
        kind: "Erc1155Error",
        data: {
            kind: "ContractError",
            data: {
                kind: "OwnerBalanceNotEnough",
                data: user.address,
            },
        },
    });
});

it("op should sell the NFT and pay the shareholders", async () => {
    expectOk(
        await scarcityInteract({
            function: "transfer",
            from: op.address,
            to: user.address,
            tokenId: nft1Id,
            price: `${nftPrice}`,
        }),
    );

    const nft1Balance = await erc1155View({
        function: "balanceOf",
        target: user.address,
        tokenId: nft1Id,
    });
    expectOk(nft1Balance);
    expect(nft1Balance.result.balance).toBe("1");

    const opDolBalance = await erc1155View({
        function: "balanceOf",
        target: op.address,
        tokenId: "DOL",
    });
    expectOk(opDolBalance);
    expect(opDolBalance.result.balance).toBe(`${opBaseBalance + nftPrice}`);

    const userDolBalance = await erc1155View({
        function: "balanceOf",
        target: user.address,
        tokenId: "DOL",
    });
    expectOk(userDolBalance);
    expect(userDolBalance.result.balance).toBe(`${userBaseBalance - nftPrice}`);
});

it("mint nft", async () => {
    const interaction = await scarcityInteract({
        function: "mintNft",
        scarcity: "legendary",
        royalties: {
            [op.address]: UNIT,
        },
        rate: nftRate,
    });

    expectOk(interaction);

    const nftBaseId = interaction.originalTxId;

    const attachedRoyalties = await scarcityView({
        function: "getAttachedRoylaties",
        baseId: nftBaseId,
    });
    expectOk(attachedRoyalties);
    expect(attachedRoyalties.result).toBeDefined();

    // const { state: scarcityState } = (await scarcityContract.readState()).cachedValue;
    // expect(scarcityState.allAttachedRoyalties[nftBaseId]).toBeDefined();

    // const { state: erc1155State } = (await erc1155Contract.readState()).cachedValue;

    for (let i = 0; i < 10; i++) {
        const nftId = `${i + 1}-LEGENDARY-${nftBaseId}`;

        const nft = await erc1155View({
            function: "getToken",
            tokenId: nftId,
        });
        expectOk(nft);
        expect(nft.result).toBeDefined();

        // expect(erc1155State.tokens[nftId]).toBeDefined();
    }

    const nft = await erc1155View({
        function: "getToken",
        tokenId: `11-LEGENDARY-${nftBaseId}`,
    });
    expectError(nft);

    // expect(erc1155State.tokens[`11-LEGENDARY-${nftBaseId}`]).toBeUndefined();
});

it("should mint with a custom baseId", async () => {
    const ticker = "CUSTOM_TICKER";

    await scarcityInteract({
        function: "mintNft",
        scarcity: "legendary",
        royalties: {
            [op.address]: UNIT,
        },
        rate: nftRate,
        baseId: ticker,
    });

    const attachedRoyalties = await scarcityView({
        function: "getAttachedRoylaties",
        baseId: ticker,
    });

    expectOk(attachedRoyalties);
    expect(attachedRoyalties.result.baseId).toBe(ticker);

    // const { state: scarcityState } = (await scarcityContract.readState()).cachedValue;
    // expect(scarcityState.allAttachedRoyalties[ticker]).toBeDefined();

    // const { state: erc1155State } = (await erc1155Contract.readState()).cachedValue;

    for (let i = 0; i < 10; i++) {
        const tokenId = `${i + 1}-LEGENDARY-${ticker}`;

        const nft = await erc1155View({
            function: "getToken",
            tokenId,
        });
        expectOk(nft);
    }

    expectError(await erc1155View({ function: "getToken", tokenId: `11-LEGENDARY-${ticker}` }));
});

// TODO: Test the case where user buys an NFT that doesn't have an edition count of 1, 10, 100 or
// 1000. (there is an NFT with 50 editions in the old contract).
