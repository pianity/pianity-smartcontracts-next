/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type ContractError =
  | {
      data: string;
      kind: "RuntimeError";
    }
  | {
      kind: "TransferAmountMustBeHigherThanZero";
    }
  | {
      kind: "TransferFromAndToCannotBeEqual";
    }
  | {
      data: string;
      kind: "TokenNotFound";
    }
  | {
      kind: "IDontLikeThisContract";
    }
  | {
      data: number;
      kind: "CallerBalanceNotEnough";
    }
  | {
      kind: "OnlyOwnerCanEvolve";
    }
  | {
      kind: "EvolveNotAllowed";
    }
  | {
      kind: "ForbiddenNestedBatch";
    }
  | {
      kind: "CannotMixeReadAndWrite";
    }
  | {
      kind: "EmptyBatch";
    }
  | {
      kind: "UnauthorizedConfiguration";
    }
  | {
      data: string;
      kind: "UnauthorizedAddress";
    }
  | {
      data: string;
      kind: "UnauthorizedTransfer";
    }
  | {
      kind: "InvalidFee";
    }
  | {
      kind: "InvalidRate";
    }
  | {
      kind: "TokenOwnerNotFound";
    }
  | {
      data: string;
      kind: "TokenAlreadyExists";
    }
  | {
      data: string;
      kind: "TokenDoesNotExist";
    }
  | {
      data: string;
      kind: "TokenIsNotAnNFT";
    }
  | {
      kind: "InvalidTokenId";
    }
  | {
      data: string;
      kind: "TransferResult";
    }
  | {
      kind: "Erc1155ReadFailed";
    }
  | {
      data: ForeignWriteErrorFor_ContractError;
      kind: "Erc1155Error";
    }
  | {
      data: string;
      kind: "InvalidNftId";
    };
export type ForeignWriteErrorFor_ContractError =
  | {
      data: ContractError1;
      kind: "ContractError";
    }
  | {
      kind: "ParseError";
    };
export type ContractError1 =
  | {
      data: string;
      kind: "RuntimeError";
    }
  | {
      kind: "TransferAmountMustBeHigherThanZero";
    }
  | {
      kind: "TransferFromAndToCannotBeEqual";
    }
  | {
      data: string;
      kind: "TokenNotFound";
    }
  | {
      kind: "IDontLikeThisContract";
    }
  | {
      data: string;
      kind: "OwnerBalanceNotEnough";
    }
  | {
      kind: "OnlyOwnerCanEvolve";
    }
  | {
      kind: "EvolveNotAllowed";
    }
  | {
      kind: "ForbiddenNestedBatch";
    }
  | {
      kind: "CannotMixeReadAndWrite";
    }
  | {
      kind: "EmptyBatch";
    }
  | {
      kind: "UnauthorizedConfiguration";
    }
  | {
      data: string;
      kind: "UnauthorizedAddress";
    }
  | {
      data: string;
      kind: "UnauthorizedTransfer";
    }
  | {
      kind: "TokenAlreadyExists";
    };
