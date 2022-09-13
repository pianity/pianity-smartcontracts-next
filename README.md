# Pianity Smartcontracts Next

*A collection of Warp smartcontracts written in Rust by Pianity*

---

## What's Inside?

### Erc1155

A generic implementation of the Erc1155 contract, featuring a way to proxy its Transfer action
through another contract.

### Fee

A proxy contract that plugs on the Erc1155, used for managing NFT sellings, enabling a royalty-like
system.

## How to Use

To build the contracts:

```
yarn build
```

To run the test suites:

```
yarn test
```

## Automatic Typescript Types Generation

All the Rust contracts in this repo include a way to automatically generate Typescript types from
them so they can be consumed easier in TS programs. This works by using two tools:

1. [schemars](https://github.com/GREsau/schemars): used to generate a standard JSON Schema from the
   types we want to export;
1. [json-schema-to-typescript](https://github.com/bcherny/json-schema-to-typescript): consumes
   these JSON Schema and produce valid TS types.

The *.json* & *.ts* files are outputted to `<contract>/definition/bindings/{json,ts}`.
