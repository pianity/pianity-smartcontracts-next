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
      kind: "CannotMixReadAndWrite";
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
      data: string;
      kind: "TransferResult";
    }
  | {
      data: ForeignWriteErrorFor_ContractError;
      kind: "Erc1155Error";
    }
  | {
      kind: "Erc1155ReadFailed";
    }
  | {
      data: string;
      kind: "ShuffleNotFound";
    }
  | {
      /**
       * @minItems 2
       * @maxItems 2
       */
      data: [string, string];
      kind: "NftAlreadyInAShuffle";
    }
  | {
      data: string;
      kind: "NoNftAvailable";
    }
  | {
      kind: "BoostCapExceeded";
    }
  | {
      kind: "ContractIsPaused";
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
    }
  | {
      kind: "ContractIsPaused";
    };
