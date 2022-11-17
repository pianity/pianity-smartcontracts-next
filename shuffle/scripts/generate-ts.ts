// NOTE: It is currently required to use a script file in order to run json2ts instead of using its
// CLI because the CLI interprets `--additionalProperties false` as `false` being a string.

import { parse, join } from "node:path";
import { readdirSync, writeFileSync, mkdirSync } from "node:fs";

import { compileFromFile } from "json-schema-to-typescript";

const DEFINITION_DIR = "./definition";
const CARGO_TOML_PATH = join(DEFINITION_DIR, "Cargo.toml");
const BINDINGS_ROOT = join(DEFINITION_DIR, "bindings");
const BINDINGS_JSON = join(BINDINGS_ROOT, "json");
const BINDINGS_TS = join(BINDINGS_ROOT, "ts");

function addActionsType(content: string): string {
    return (
        content +
        `/**
 * This type allows to restrict the type of an interaction to a specific action.
 *
 * Example:
 * \`\`\`typescript
 * const specificAction: Actions["specificAction"] = { function: "specificAction", foo: "bar" };
 * \`\`\`
 */
type Actions = {
    [K in Action["function"]]: Action & { function: K };
};`
    );
}

function fixRecursiveActionType(content: string): string {
    return content
        .replace(/export type Action1 =.*?((\r*\n){2}|$)/gs, "")
        .replace("Action1", "Action");
}

(async () => {
    mkdirSync(BINDINGS_TS, { recursive: true });

    for (const jsonFilename of readdirSync(BINDINGS_JSON)) {
        const nameWoExt = parse(jsonFilename).name;
        const jsonPath = join(BINDINGS_JSON, jsonFilename);
        const tsPath = join(BINDINGS_TS, nameWoExt + ".ts");

        let tsContent = await compileFromFile(jsonPath, {
            additionalProperties: false,
            bannerComment: "",
        });

        if (nameWoExt === "Action") {
            tsContent = addActionsType(fixRecursiveActionType(tsContent));
        }

        writeFileSync(tsPath, tsContent);
    }
})();
