import { it, expect, beforeAll, afterAll } from "vitest";
import Arlocal from "arlocal";
import { Contract, LoggerFactory, Warp, WarpFactory } from "warp-contracts";
import { Wallet } from "warp-contracts/lib/types/contract/testing/Testing";

import { Parameters as Erc1155State } from "erc1155/State";
import { ContractError as Erc1155Error } from "erc1155/ContractError";
import { Action as Erc1155Action } from "erc1155/Action";
import { ReadResponse as Erc1155ReadResponse } from "erc1155/ReadResponse";
import * as Lock from "lock/index";

import {
    UNIT,
    deployContract,
    createInteractor,
    Interactor,
    Viewer,
    createViewer,
    generateWallet,
    expectOk,
    x,
    expectError,
} from "@/utils";
import { DeployPlugin } from "warp-contracts-plugin-deploy";
import BigNumber from "bignumber.js";
import { PgSortKeyCache, PgSortKeyCacheOptions } from "warp-contracts-postgres";

let arlocal: Arlocal;
let warp: Warp;

let op: Wallet;
let user: Wallet;
let bank: Wallet;
let user2: Wallet;

let erc1155Contract: Contract<Erc1155State>;
let erc1155TxId: string;
let erc1155Interact: Interactor<Erc1155Action, Erc1155Error>;
let erc1155View: Viewer<Erc1155Action, Erc1155ReadResponse, Erc1155State, Erc1155Error>;

let lockContract: Contract<Lock.Parameters>;
let lockTxId: string;
let lockInteract: Interactor<Lock.Action, Lock.ContractError>;
let lockView: Viewer<Lock.Action, Lock.ReadResponse, Lock.Parameters, Lock.ContractError>;

const opBaseBalance = 100 * UNIT;
const userBaseBalance = 100 * UNIT;

beforeAll(async () => {
    LoggerFactory.INST.logLevel("error");
    LoggerFactory.INST.logLevel("debug", "WASM:Rust");
    LoggerFactory.INST.logLevel("debug", "ContractHandler");

    arlocal = new Arlocal(1987, false, `./arlocal.lock.db`, false);
    await arlocal.start();
    warp = WarpFactory.forLocal(1987, undefined, { inMemory: true, dbLocation: "/dev/null" }).use(
        new DeployPlugin(),
    );
    op = await generateWallet();
    user = await generateWallet();
    user2 = await generateWallet();
    bank = await generateWallet();

    await warp.testing.addFunds(op.jwk);
    await warp.testing.addFunds(user.jwk);
    await warp.testing.addFunds(user2.jwk);
    await warp.testing.addFunds(bank.jwk);

    const erc1155InitState: Erc1155State = {
        name: "TEST-ERC1155",
        initialState: {
            settings: {
                superOperators: [op.address, bank.address],
                operators: [],
                proxies: [],
                allowFreeTransfer: true,
                paused: false,
                defaultToken: "DOL",
            },
            tickerNonce: 0,
            tokens: {
                DOL: {
                    ticker: "DOL",
                    balances: {
                        [op.address]: `${opBaseBalance}`,
                        [user.address]: `${userBaseBalance}`,
                        [user2.address]: "0",
                        [bank.address]: "9999999999999999999",
                        // [bank.address]: "100",
                    },
                },
            },
            approvals: {
                [user.address]: {
                    approves: { [op.address]: true },
                },
            },
        },
        canEvolve: false,
    };

    erc1155TxId = (await deployContract(warp, op.jwk, "erc1155", erc1155InitState)).contractTxId;
    erc1155Contract = warp
        .contract<Erc1155State>(erc1155TxId)
        .setEvaluationOptions({
            internalWrites: true,
            throwOnInternalWriteError: false,
            mineArLocalBlocks: false,
        })
        .connect(op.jwk);
    erc1155Interact = createInteractor<Erc1155Action, Erc1155Error>(warp, erc1155Contract, op.jwk);
    erc1155View = createViewer<Erc1155Action, Erc1155ReadResponse, Erc1155State, Erc1155Error>(
        erc1155Contract,
    );

    const lockInitState: Lock.Parameters = {
        name: "TEST-LOCK",
        initialState: {
            settings: {
                superOperators: [op.address, bank.address, user2.address],
                operators: [],
                erc1155: erc1155TxId,
                paused: false,
            },
            vault: {},
        },
        canEvolve: false,
    };

    lockTxId = (await deployContract(warp, op.jwk, "lock", lockInitState)).contractTxId;
    lockContract = warp
        .contract<Lock.Parameters>(lockTxId)
        .setEvaluationOptions({
            internalWrites: true,
            throwOnInternalWriteError: false,
            mineArLocalBlocks: false,
        })
        .connect(op.jwk);
    lockInteract = createInteractor<Lock.Action, Lock.ContractError>(warp, lockContract, op.jwk);
    lockView = createViewer<Lock.Action, Lock.ReadResponse, Lock.Parameters, Lock.ContractError>(
        lockContract,
    );

    console.log(
        `OP: ${op.address}`,
        `\nUSER: ${user.address}`,
        `\nUSER2: ${user2.address}`,
        `\nBANK: ${bank.address}`,
        `\nLOCK: ${lockTxId}`,
        `\nERC1155: ${erc1155TxId}`,
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

it("initialize Lock", async () => {
    const stateBefore = (await lockContract.readState()).cachedValue.state;
    expect(stateBefore.initialState).toBeTruthy();

    expectOk(await lockInteract({ function: "initialize" }));

    const stateAfter = (await lockContract.readState()).cachedValue.state;
    expect(stateAfter.initialState).toBeNull();
});

it("configure Lock as proxy of Erc1155", async () => {
    const settings = await erc1155View({ function: "readSettings" });
    expectOk(settings);

    expectOk(
        await erc1155Interact({
            function: "configure",
            proxies: [...settings.result.proxies, lockTxId],
            superOperators: [...settings.result.superOperators, lockTxId],
        }),
    );
});

it("correctly set balance to 0 after emptying it using erc1155 directly", async () => {
    const tokenId = "DOL";
    const qty = "10";

    expectOk(
        await erc1155Interact(
            {
                function: "transfer",
                target: user2.address,
                tokenId,
                qty,
            },
            { wallet: bank.jwk },
        ),
    );

    const balance = await erc1155View({ function: "balanceOf", target: user2.address, tokenId });
    expectOk(balance);
    expect(balance.result.balance).toBe(qty);

    expectOk(
        await erc1155Interact(
            { function: "transfer", target: bank.address, tokenId, qty },
            { wallet: user2.jwk },
        ),
    );

    const balanceAfter = await erc1155View({
        function: "balanceOf",
        target: user2.address,
        tokenId,
    });
    expectOk(balanceAfter);
    expect(balanceAfter.result.balance).toBe("0");
});

it("should execute a cliff transferLock correctly", async () => {
    const target = user2.address;
    const tokenId = "DOL";
    const qty = "10";

    const interaction = await lockInteract(
        {
            function: "transferLocked",
            tokenId,
            target,
            duration: 2,
            qty,
            method: "cliff",
        },
        { wallet: bank.jwk },
    );
    expectOk(interaction);

    await warp.testing.mineBlock();
    await warp.testing.mineBlock();

    expectOk(await lockInteract({ function: "unlock" }));

    {
        const balances = await erc1155View({
            function: "getToken",
            tokenId,
        });
        expectOk(balances);
        expect(balances.result[1].balances[target]).toEqual(qty);
        const contractBalance = balances.result[1].balances[lockTxId];
        expect(contractBalance).toBeUndefined();
    }
}, 60_000);

function transferToVault(
    input: Lock.Actions["transferLocked"],
    {
        startedAt,
        currentBlock,
        from,
    }: {
        startedAt: number;
        currentBlock: number;
        from: string;
    },
): Lock.LockedBalance {
    if (input.method === "cliff") {
        const value: Lock.LockedBalance = {
            type: "cliff",
            at: startedAt,
            duration: input.duration,
            qty: input.qty,
            from,
            tokenId: "DOL",
        };

        return value;
    } else {
        const value: Lock.LockedBalance = {
            type: "linear",
            at: startedAt,
            duration: input.duration,
            qty: input.qty,
            from,
            tokenId: "DOL",
            unlocked: BigNumber(input.qty)
                .times((currentBlock - startedAt) / input.duration)
                .decimalPlaces(7, BigNumber.ROUND_HALF_UP)
                .decimalPlaces(0, BigNumber.ROUND_DOWN)
                .toFixed(),
        };

        return value;
    }
}

it(
    "fuzzy test locks",
    async () => {
        const target = "fuzz-test-receiver";
        const tokenId = "DOL";

        {
            const balance = await erc1155View({
                function: "balanceOf",
                target,
                tokenId,
            });
            expectOk(balance);
            expect(balance.result.balance).toEqual("0");

            const vault = await lockView({ function: "getVault", owner: target });
            expectError(vault, { kind: "OwnerHasNoVault", data: target });
        }

        await lockInteract({ function: "unlock" });

        let totalUnlocked = "0";

        for (let i = 0; i < 50; i++) {
            const duration = Math.floor(Math.random() * 10) + 1;
            const qty = Math.floor(Math.random() * 999999999).toFixed();
            const method = i % 2 === 0 ? "cliff" : "linear";

            const input: Lock.Actions["transferLocked"] = {
                function: "transferLocked",
                tokenId,
                target,
                duration,
                qty,
                method,
            };
            const interaction = await lockInteract(input, { wallet: bank.jwk });
            expectOk(interaction);

            const startedAt = (await warp.arweave.network.getInfo()).height;

            for (let block = 0; block < duration; block++) {
                const height = (await warp.arweave.network.getInfo()).height;

                const vault = await lockView({ function: "getVault", owner: target });
                expectOk(vault);
                expect(vault.result[1]).toHaveLength(1);
                const expectedVault = transferToVault(input, {
                    startedAt,
                    currentBlock: startedAt + block,
                    from: bank.address,
                });
                expect(vault.result[1][0]).toEqual(expectedVault);

                const balance = await erc1155View({
                    function: "balanceOf",
                    target,
                    tokenId,
                });
                expectOk(balance);
                const expectedBalance =
                    expectedVault.type === "cliff"
                        ? totalUnlocked
                        : BigNumber(totalUnlocked).plus(expectedVault.unlocked).toFixed();
                expect(balance.result.balance).toEqual(expectedBalance);

                expectOk(await lockInteract({ function: "unlock" }));
            }

            expectOk(await lockInteract({ function: "unlock" }));

            totalUnlocked = BigNumber(totalUnlocked).plus(qty).toFixed();

            {
                const balance = await erc1155View({
                    function: "getToken",
                    tokenId,
                });
                expectOk(balance);
                expect(balance.result[1].balances[target]).toEqual(totalUnlocked);
                expect(balance.result[1].balances[lockTxId]).toBeUndefined();

                const vault = await lockView({ function: "getVault", owner: target });
                expectError(vault, { kind: "OwnerHasNoVault", data: target });
            }
        }
    },
    20 * 60_000,
);
