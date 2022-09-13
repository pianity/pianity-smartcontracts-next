import fs from 'fs';

import ArLocal from 'arlocal';
import Arweave from 'arweave';
import { JWKInterface } from 'arweave/node/lib/wallet';
import {
  getTag,
  InteractionResult,
  LoggerFactory,
  PstContract,
  PstState,
  Warp,
  WarpNodeFactory,
  SmartWeaveTags,
} from 'warp-contracts';
import path from 'path';
import { addFunds, mineBlock } from '../utils';

jest.setTimeout(30000);

describe('Testing the Profit Sharing Token', () => {
  let contractSrc: Buffer;

  let wallet: JWKInterface;
  let walletAddress: string;

  let initialState: PstState;

  let arweave: Arweave;
  let arlocal: ArLocal;
  let warp: Warp;
  let pst: PstContract;
  let pst2: PstContract;

  let foreignContractTxId: string;
  let contractTxId: string;

  beforeAll(async () => {
    // note: each tests suit (i.e. file with tests that Jest is running concurrently
    // with another files has to have ArLocal set to a different port!)
    arlocal = new ArLocal(1820, false);
    await arlocal.start();

    arweave = Arweave.init({
      host: 'localhost',
      port: 1820,
      protocol: 'http',
    });

    LoggerFactory.INST.logLevel('error');
    LoggerFactory.INST.logLevel('debug', 'WASM:Rust');
    //LoggerFactory.INST.logLevel('debug', 'WasmContractHandlerApi');

    warp = WarpNodeFactory.forTesting(arweave);

    wallet = await arweave.wallets.generate();
    await addFunds(arweave, wallet);
    walletAddress = await arweave.wallets.jwkToAddress(wallet);

    contractSrc = fs.readFileSync(path.join(__dirname, '../pkg/rust-contract_bg.wasm'));
    const stateFromFile: PstState = JSON.parse(fs.readFileSync(path.join(__dirname, './data/token-pst.json'), 'utf8'));

    initialState = {
      ...stateFromFile,
      ...{
        owner: walletAddress,
        balances: {
          ...stateFromFile.balances,
          [walletAddress]: 555669,
        },
      },
    };

    // deploying contract using the new SDK.
    contractTxId = await warp.createContract.deploy({
      wallet,
      initState: JSON.stringify(initialState),
      src: contractSrc,
      wasmSrcCodeDir: path.join(__dirname, '../src'),
      wasmGlueCode: path.join(__dirname, '../pkg/rust-contract.js'),
    });

    console.log(contractTxId);

    foreignContractTxId = await warp.createContract.deploy({
      wallet,
      initState: JSON.stringify({
        ...initialState,
        ...{
          ticker: 'FOREIGN_PST',
          name: 'foreign contract',
        },
      }),
      src: contractSrc,
      wasmSrcCodeDir: path.join(__dirname, '../src'),
      wasmGlueCode: path.join(__dirname, '../pkg/rust-contract.js'),
    });

    console.log(foreignContractTxId);

    // connecting to the PST contract
    pst = warp.pst(contractTxId).setEvaluationOptions({
      internalWrites: true,
    }) as PstContract;

    pst2 = warp.pst(foreignContractTxId).setEvaluationOptions({
      internalWrites: true,
    }) as PstContract;

    // connecting wallet to the PST contract
    pst.connect(wallet);
    pst2.connect(wallet);

    await mineBlock(arweave);
  });

  afterAll(async () => {
    await arlocal.stop();
  });

  it('should properly deploy contract', async () => {
    const contractTx = await arweave.transactions.get(contractTxId);

    expect(contractTx).not.toBeNull();

    const contractSrcTx = await arweave.transactions.get(getTag(contractTx, SmartWeaveTags.CONTRACT_SRC_TX_ID));
    expect(getTag(contractSrcTx, SmartWeaveTags.CONTENT_TYPE)).toEqual('application/wasm');
    expect(getTag(contractSrcTx, SmartWeaveTags.WASM_LANG)).toEqual('rust');
  });

  it('should read pst state and balance data', async () => {
    expect(await pst.currentState()).toEqual(initialState);

    expect((await pst.currentBalance('uhE-QeYS8i4pmUtnxQyHD7dzXFNaJ9oMK-IM-QPNY6M')).balance).toEqual(10000000);
    expect((await pst.currentBalance('33F0QHcb22W7LwWR1iRC8Az1ntZG09XQ03YWuw2ABqA')).balance).toEqual(23111222);
    expect((await pst.currentBalance(walletAddress)).balance).toEqual(555669);
  });

  it('should properly transfer tokens', async () => {
    await pst.transfer({
      target: 'uhE-QeYS8i4pmUtnxQyHD7dzXFNaJ9oMK-IM-QPNY6M',
      qty: 555,
    });
    await mineBlock(arweave);

    expect((await pst.currentState()).balances[walletAddress]).toEqual(555669 - 555);
    expect((await pst.currentState()).balances['uhE-QeYS8i4pmUtnxQyHD7dzXFNaJ9oMK-IM-QPNY6M']).toEqual(10000000 + 555);
  });

  it('should properly view contract state', async () => {
    const result = await pst.currentBalance('uhE-QeYS8i4pmUtnxQyHD7dzXFNaJ9oMK-IM-QPNY6M');
    expect(result.balance).toEqual(10000000 + 555);
    expect(result.ticker).toEqual('EXAMPLE_PST_TOKEN');
    expect(result.target).toEqual('uhE-QeYS8i4pmUtnxQyHD7dzXFNaJ9oMK-IM-QPNY6M');
  });

  // note: the dummy logic on the test contract should add 1000 tokens
  // to each address, if the foreign contract state 'ticker' field = 'FOREIGN_PST'
  it('should properly read foreign contract state', async () => {
    await pst.writeInteraction({
      function: 'foreignRead',
      contractTxId: foreignContractTxId,
    });
    await mineBlock(arweave);
    expect((await pst.currentState()).balances[walletAddress]).toEqual(555669 - 555 + 1000);
    expect((await pst.currentState()).balances['uhE-QeYS8i4pmUtnxQyHD7dzXFNaJ9oMK-IM-QPNY6M']).toEqual(
      10000000 + 555 + 1000
    );
  });

  it('should properly perform internal write', async () => {
    expect((await pst2.currentBalance('uhE-QeYS8i4pmUtnxQyHD7dzXFNaJ9oMK-IM-QPNY6M')).balance).toEqual(10000000);

    await pst.writeInteraction({
      function: 'foreignWrite',
      contractTxId: foreignContractTxId,
      target: 'uhE-QeYS8i4pmUtnxQyHD7dzXFNaJ9oMK-IM-QPNY6M',
      qty: 555,
    });
    await mineBlock(arweave);

    expect((await pst2.currentBalance('uhE-QeYS8i4pmUtnxQyHD7dzXFNaJ9oMK-IM-QPNY6M')).balance).toEqual(10000555);
    expect((await pst2.currentBalance(walletAddress)).balance).toEqual(555669 - 555);
  });

  it("should properly evolve contract's source code", async () => {
    expect((await pst.currentState()).balances[walletAddress]).toEqual(556114);

    const newSource = fs.readFileSync(path.join(__dirname, './data/token-evolve.js'), 'utf8');

    const newSrcTxId = await pst.save({ src: newSource });
    await mineBlock(arweave);

    await pst.evolve(newSrcTxId);
    await mineBlock(arweave);

    // note: the evolved balance always adds 555 to the result
    expect((await pst.currentBalance(walletAddress)).balance).toEqual(556114 + 555);
  });

  it('should properly perform dry write with overwritten caller', async () => {
    const newWallet = await arweave.wallets.generate();
    const overwrittenCaller = await arweave.wallets.jwkToAddress(newWallet);
    await pst.transfer({
      target: overwrittenCaller,
      qty: 1000,
    });

    await mineBlock(arweave);

    // note: transfer should be done from the "overwrittenCaller" address, not the "walletAddress"
    const result: InteractionResult<PstState, unknown> = await pst.dryWrite(
      {
        function: 'transfer',
        target: 'uhE-QeYS8i4pmUtnxQyHD7dzXFNaJ9oMK-IM-QPNY6M',
        qty: 333,
      },
      overwrittenCaller
    );

    expect(result.state.balances[walletAddress]).toEqual(555114 - 1000 + 1000);
    expect(result.state.balances['uhE-QeYS8i4pmUtnxQyHD7dzXFNaJ9oMK-IM-QPNY6M']).toEqual(10000000 + 1000 + 555 + 333);
    expect(result.state.balances[overwrittenCaller]).toEqual(1000 - 333);
  });
});
