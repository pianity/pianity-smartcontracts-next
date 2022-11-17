export type Action =
  | {
      baseId?: string | null;
      function: "mintShuffle";
      nfts: ShuffleBaseIds;
    }
  | {
      boost?: BoostOpenShuffle | null;
      function: "openShuffle";
      owner?: string | null;
      shuffleId: string;
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
export type ShuffleBaseIds =
  | {
      /**
       * @minItems 2
       * @maxItems 2
       */
      legendary: [string, string];
    }
  | {
      /**
       * @minItems 3
       * @maxItems 3
       */
      epic: [string, string, string];
    }
  | {
      /**
       * @minItems 4
       * @maxItems 4
       */
      rare: [string, string, string, string];
    };


export interface BoostOpenShuffle {
  /**
   * 0 <= boost <= 1
   */
  boost: number;
  shufflePrice: string;
}
/**
 * This type allows to restrict the type of an interaction to a specific action.
 *
 * Example:
 * ```typescript
 * const specificAction: Actions["specificAction"] = { function: "specificAction", foo: "bar" };
 * ```
 */
export type Actions = {
    [K in Action["function"]]: Action & { function: K };
};