export type Action =
  | {
      baseId: string;
      function: "editAttachedRoyalties";
      rate: number;
      royalties: {
        [k: string]: number;
      };
    }
  | {
      baseId: string;
      function: "attachRoyalties";
      rate: number;
      royalties: {
        [k: string]: number;
      };
    }
  | {
      baseId?: string | null;
      function: "mintNft";
      rate: number;
      royalties: {
        [k: string]: number;
      };
      scarcity: Scarcity;
    }
  | {
      from: string;
      function: "transfer";
      price: string;
      to: string;
      tokenId: string;
    }
  | {
      function: "configure";
      operators?: string[] | null;
      paused?: boolean | null;
      superOperators?: string[] | null;
    }
  | {
      function: "evolve";
      value: string;
    }
  | {
      actions: Action[];
      function: "batch";
    };
export type Scarcity = "unique" | "legendary" | "epic" | "rare";

/**
 * This type allows to restrict the type of an interaction to a specific action.
 *
 * Example:
 * ```typescript
 * const specificAction: Actions["specificAction"] = { function: "specificAction", foo: "bar" };
 * ```
 */
type Actions = {
    [K in Action["function"]]: Action & { function: K };
};