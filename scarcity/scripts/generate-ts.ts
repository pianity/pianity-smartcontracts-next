import { parse, join } from "node:path";
import { readdirSync, writeFileSync, mkdirSync } from "node:fs";

import { SyntaxKind, createSourceFile, forEachChild, ScriptTarget } from "typescript";
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
export type Actions = {
    [K in Action["function"]]: Action & { function: K };
};`
    );
}

function fixRecursiveType(source: string, type: string): string {
    const sourceFile = createSourceFile(`${type}.ts`, source, ScriptTarget.Latest);

    let recurTypeContent: string | undefined;
    forEachChild(sourceFile, (node) => {
        if (!recurTypeContent && node.kind === SyntaxKind.TypeAliasDeclaration) {
            const identifierText = node
                .getChildren(sourceFile)
                .find(({ kind }) => kind === SyntaxKind.Identifier)
                ?.getText(sourceFile);

            if (identifierText === `${type}1`) {
                recurTypeContent = node.getText(sourceFile);
            }
        }
    });

    return source.replace(recurTypeContent, "").replace(`${type}1`, type);
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
            tsContent = addActionsType(fixRecursiveType(tsContent, nameWoExt));
        } else if (nameWoExt === "ReadResponse") {
            tsContent = fixRecursiveType(tsContent, nameWoExt);
        }

        writeFileSync(tsPath, tsContent);
    }
})();
