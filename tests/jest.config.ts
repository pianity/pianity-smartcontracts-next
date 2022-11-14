import { pathsToModuleNameMapper, InitialOptionsTsJest } from "ts-jest";

import { compilerOptions } from "tsconfig";

const config: InitialOptionsTsJest = {
    preset: "ts-jest",

    moduleNameMapper: pathsToModuleNameMapper(compilerOptions.paths, { prefix: "<rootDir>/" }),
    setupFilesAfterEnv: ["<rootDir>/jest.setup.ts"],
};

export default config;
