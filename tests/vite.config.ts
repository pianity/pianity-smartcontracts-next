import { ParsedStack } from "vitest";
import { defineConfig } from "vitest/config";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig({
    test: {
        onStackTrace(error: Error, { file }: ParsedStack): boolean | void {
            return true;
            // // If we've encountered a ReferenceError, show the whole stack.
            // if (error.name === "ReferenceError") return;
            //
            // // Reject all frames from third party libraries.
            // if (file.includes("node_modules")) return false;
        },
    },
    base: "",
    plugins: [tsconfigPaths()],
});
