const fs = require('fs');
const path = require('path');
const { WarpNodeFactory } = require('warp-contracts');
const { mineBlock } = require('./utils/mine-block');
const { loadWallet, walletAddress } = require('./utils/load-wallet');
const { connectArweave } = require('./utils/connect-arweave');

module.exports.deploy = async function (host, port, protocol, target, walletJwk) {
  const arweave = connectArweave(host, port, protocol);
  const warp = WarpNodeFactory.memCached(arweave);
  const wallet = await loadWallet(arweave, walletJwk, target);
  const walletAddr = await walletAddress(arweave, wallet);
  const contractSrc = fs.readFileSync(path.join(__dirname, '../../pkg/rust-contract_bg.wasm'));
  const stateFromFile = JSON.parse(fs.readFileSync(path.join(__dirname, '../state/init-state.json'), 'utf-8'));

  const initialState = {
    ...stateFromFile,
    ...{
      owner: walletAddr,
      balances: {
        ...stateFromFile.balances,
        [walletAddr]: 10000000,
      },
    },
  };
  const contractTxId = await warp.createContract.deploy(
    {
      wallet,
      initState: JSON.stringify(initialState),
      src: contractSrc,
      wasmSrcCodeDir: path.join(__dirname, '../../src'),
      wasmGlueCode: path.join(__dirname, '../../pkg/rust-contract.js'),
    },
    target == 'mainnet'
  );
  fs.writeFileSync(path.join(__dirname, `../${target}/contract-tx-id.txt`), contractTxId);

  if (target == 'testnet' || target == 'local') {
    await mineBlock(arweave);
  }

  if (target == 'testnet') {
    console.log(`Check contract at https://sonar.warp.cc/#/app/contract/${contractTxId}?network=testnet`);
  } else {
    console.log('Contract tx id', contractTxId);
  }
};
