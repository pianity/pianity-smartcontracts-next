# Pianity Smartcontracts Next

*A collection of Warp smartcontracts written in Rust by Pianity.*

---

## What's Inside?

### Erc1155

A generic implementation of the Erc1155 contract, featuring a way to proxy its Transfer action
through another contract.

### Fee

A proxy contract that plugs on the Erc1155 used for managing NFT sellings, enabling a royalty-like
system.

## Structure of a Contract Directory

A contract is separated into two different crates: the *definition* and the *implementation*.

The *definition* crate contains all the types that the contract defines, mainly: its state, the
actions and their parameters, the errors it can produce, ... This crate only includes type
information, no actual implementation whatsoever. Its goal is to define and expose the shape of the
contract to provide valuable information as to how this contract should be consumed. For example, a
Rust program that wants to, say, evaluate the contract's state, could import this crate and get
accurate type checking on it. Note that this crate also contains tools that allows it to be
translated automatically into Typescript so that any TS program can also consume its types. See
[the dedicated section](#generate-typescript-bindings) to learn more about this.

The *implementation* crate is where the contract's logic resides. It imports the *definition* crate
and implements all the different actions defined in it. This crate will eventually be compiled to
WASM, this is what will get uploaded to the blockchain when deploying the contract; it is not meant
to be consumed in another way.

## How to Use this Repo

### Build the Contracts

```
yarn build
```

The resulting WASM binaries with their glue codes are located `<contract>/implementation/pkg`.

### Generate Typescript Bindings

```
yarn gen-bindings
```

All the Rust contracts in this repo include a way to automatically generate Typescript types from
them so they can be consumed easier in TS programs. This works by using two tools:

1. [schemars](https://github.com/GREsau/schemars): to generate standard JSON schemas from the types
   we want to export. The code that calls this library is located in
   `<contract>/definition/src/generate_types.rs`. It is written in a test, to facilitate its
   execution. See `gen-json` script of `<contract>/package.json`. The *.json* files are outputted
   to `<contract>/definition/bindings/json`.

1. [json-schema-to-typescript](https://github.com/bcherny/json-schema-to-typescript): to consume
   these JSON schemas and produce valid TS types. The code that runs this library is located in
   `<contract>/scripts/generate-ts.ts`. See `gen-ts` script of `<contract>/package.json`. The *.ts*
   files are outputted to `<contract>/definition/bindings/ts`.

The resulting *.ts* files are valid TS files that exports the types `State` and `Action` and all
other types referenced inside them. In the future, the script will probably generate a valid NPM
package to make it easy to publish/consume the generated types.

### Run the Test Suites

```
yarn test
```
