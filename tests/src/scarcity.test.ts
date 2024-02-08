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

import {
    deployContract,
    createInteractor,
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
let bank: Wallet;

let erc1155Contract: Contract<Erc1155State>;
let erc1155TxId: string;
let erc1155Interact: Interactor<Erc1155Action, Erc1155Error>;
let erc1155View: Viewer<Erc1155Action, Erc1155ReadResponse, Erc1155State, Erc1155Error>;

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
    LoggerFactory.INST.logLevel("debug", "ContractHandler");

    arlocal = new Arlocal(1986, false, `./arlocal.scarcity.db`, false);
    await arlocal.start();
    warp = WarpFactory.forLocal(1986, undefined, { inMemory: true, dbLocation: "/dev/null" }).use(
        new DeployPlugin(),
    );
    op = await generateWallet();
    user = await generateWallet();
    bank = await generateWallet();

    await warp.testing.addFunds(op.jwk);
    await warp.testing.addFunds(user.jwk);
    await warp.testing.addFunds(bank.jwk);

    const erc1155InitState: Erc1155State = {
        name: "TEST-ERC1155",
        canEvolve: false,
        initialState: {
            settings: {
                defaultToken: "DOL",
                superOperators: [op.address, bank.address],
                operators: [],
                proxies: [],
                allowFreeTransfer: true,
                paused: false,
            },
            tickerNonce: 0,
            tokens: {
                DOL: {
                    ticker: "DOL",
                    balances: {
                        [op.address]: `${opBaseBalance}`,
                        [user.address]: `${userBaseBalance}`,
                        [bank.address]: `9999999999999999999`,
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
}, 40_000);

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

// it("proxy transfer should transfer simple tokens successfully", async () => {
//     expectOk(
//         await scarcityInteract({
//             function: "proxyTransfer",
//             from: bank.address,
//             target: "test-proxy-transfer",
//             tokenId: "DOL",
//             qty: "123",
//         }),
//     );
//
//     const balance = await erc1155View({
//         function: "balanceOf",
//         target: "test-proxy-transfer",
//         tokenId: "DOL",
//     });
//
//     expectOk(balance);
//     expect(balance.result.balance).toEqual("123");
// });

// it("proxy transfer should fail on nfts", async () => {
//     expectError(
//         await scarcityInteract({
//             function: "proxyTransfer",
//             from: op.address,
//             target: "test-proxy-transfer",
//             tokenId: nft1Id,
//             qty: "123",
//         }),
//         { kind: "CantUseProxyTransferForNft", data: nft1Id },
//     );
// });

// it("proxy transfer should fail if token has royalties", async () => {
//     expectOk(
//         await scarcityInteract({
//             function: "attachRoyalties",
//             baseId: "DOL",
//             rate: 1_000_000,
//             royalties: {
//                 [bank.address]: 1_000_000,
//             },
//         }),
//     );
//
//     expectError(
//         await scarcityInteract({
//             function: "proxyTransfer",
//             from: op.address,
//             target: "test-proxy-transfer",
//             tokenId: "DOL",
//             qty: "123",
//         }),
//         { kind: "CantUseProxyTransferOnTokenWithRoyalties", data: "DOL" },
//     );
//
//     expectOk(
//         await scarcityInteract({
//             function: "removeAttachedRoyalties",
//             baseId: "DOL",
//         }),
//     );
// });

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
            target: unknownAddress,
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

    const royalties = await scarcityView({ function: "getRoyalties", baseId: nft1BaseId });
    expectOk(royalties);
    expect(royalties.result[1]).toEqual(fees);
});

it("return correct error type on bad tranfer", async () => {
    const result = await scarcityInteract({
        function: "transfer",
        tokenId: nft1Id,
        from: op.address,
        target: user.address,
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
            target: user.address,
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
        function: "getRoyalties",
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
}, 10_000);

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
        function: "getRoyalties",
        baseId: ticker,
    });

    expectOk(attachedRoyalties);
    expect(attachedRoyalties.result[0]).toBe(ticker);

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
}, 10_000);

it("should mint an nft, sell it and pay shareholders", async () => {
    const randomId = Math.random().toString(36).substring(7);
    const share1 = `${randomId}-1`;
    const price = 100;
    const buyer = await generateWallet();
    const rebuyer = await generateWallet();

    await warp.testing.addFunds(buyer.jwk);
    await warp.testing.addFunds(rebuyer.jwk);

    expectOk(
        await erc1155Interact(
            {
                function: "setApprovalForAll",
                operator: op.address,
                approved: true,
            },
            { wallet: buyer.jwk },
        ),
    );
    expectOk(
        await erc1155Interact(
            {
                function: "setApprovalForAll",
                operator: op.address,
                approved: true,
            },
            { wallet: rebuyer.jwk },
        ),
    );

    expectOk(
        await erc1155Interact({
            function: "transfer",
            qty: "1",
            from: op.address,
            target: buyer.address,
        }),
    );

    expectOk(
        await erc1155Interact({
            function: "transfer",
            qty: "1",
            from: buyer.address,
            target: op.address,
        }),
    );

    const mint = await scarcityInteract({
        function: "mintNft",
        scarcity: "unique",
        royalties: {
            [share1]: UNIT,
        },
        rate: nftRate,
    });
    expectOk(mint);
    const nftId = `1-UNIQUE-${mint.originalTxId}`;

    expectOk(
        await erc1155Interact(
            {
                function: "transfer",
                from: bank.address,
                target: buyer.address,
                qty: price.toString(),
            },
            { wallet: bank.jwk },
        ),
    );

    console.log(JSON.stringify(await erc1155View({ function: "getAllTokens" }), null, 2));

    expectOk(
        await scarcityInteract({
            function: "transfer",
            tokenId: nftId,
            target: buyer.address,
            from: op.address,
            price: price.toString(),
        }),
    );

    {
        const tokensRaw = await erc1155View({
            function: "getAllTokens",
        });
        expectOk(tokensRaw);
        const tokens = new Map(tokensRaw.result);
        expect(tokens.get(nftId)?.balances[buyer.address]).toEqual("1");
        expect(tokens.get("DOL")?.balances[buyer.address]).toBeUndefined();
        expect(tokens.get("DOL")?.balances[share1]).toEqual(price.toString());
    }

    expectOk(
        await erc1155Interact(
            {
                function: "transfer",
                from: bank.address,
                target: rebuyer.address,
                qty: price.toString(),
            },
            { wallet: bank.jwk },
        ),
    );

    expectOk(
        await scarcityInteract({
            function: "transfer",
            from: buyer.address,
            target: rebuyer.address,
            price: price.toString(),
            tokenId: nftId,
        }),
    );

    {
        const tokensRaw = await erc1155View({
            function: "getAllTokens",
        });
        expectOk(tokensRaw);
        const tokens = new Map(tokensRaw.result);
        expect(tokens.get(nftId)?.balances[buyer.address]).toBeUndefined();
        console.log(`*****UNIT: ${UNIT}, nftRate: ${nftRate}`);
        expect(tokens.get("DOL")?.balances[buyer.address]).toEqual(
            (price * ((UNIT - nftRate) / UNIT)).toString(),
        );
        expect(tokens.get(nftId)?.balances[rebuyer.address]).toEqual("1");
        expect(tokens.get("DOL")?.balances[rebuyer.address]).toBeUndefined();
        expect(tokens.get("DOL")?.balances[share1]).toEqual(
            (price + price * (nftRate / UNIT)).toString(),
        );
    }
}, 50_000);

// // TODO: Test the case where user buys an NFT that doesn't have an edition count of 1, 10, 100 or
// // 1000. (there is an NFT with 50 editions in the old contract).
