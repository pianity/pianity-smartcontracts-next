export type Action =
  | {
      duration: number;
      function: "transferLocked";
      qty: string;
      to: string;
      tokenId: string;
    }
  | {
      function: "unlock";
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