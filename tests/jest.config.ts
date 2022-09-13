import { pathsToModuleNameMapper } from "ts-jest";

import { compilerOptions } from "tsconfig";

export default {
    preset: "ts-jest",

    moduleNameMapper: pathsToModuleNameMapper(compilerOptions.paths, { prefix: "<rootDir>/" }),
};
