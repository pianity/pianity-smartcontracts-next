// NOTE: It is currently required to use a script file in order to run json2ts instead of using its
// CLI because the CLI interprets `--additionalProperties false` as `false` being a string.

import { parse, join } from "node:path";
import { readdirSync, writeFileSync, mkdirSync } from "node:fs";

import { compileFromFile } from "json-schema-to-typescript";

const BINDINGS_ROOT = "./definition/bindings";
const BINDINGS_JSON = join(BINDINGS_ROOT, "json");
const BINDINGS_TS = join(BINDINGS_ROOT, "ts");

mkdirSync(BINDINGS_TS, { recursive: true });

// NOTE:
for (const fileName of readdirSync(BINDINGS_JSON)) {
  const jsonPath = join(BINDINGS_JSON, fileName);
  const tsPath = join(BINDINGS_TS, parse(fileName).name + ".ts");

  compileFromFile(jsonPath, {
    additionalProperties: false,
  }).then((tsContent) => writeFileSync(tsPath, tsContent));
}
