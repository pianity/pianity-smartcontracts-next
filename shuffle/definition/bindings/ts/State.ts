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

export interface State {
  canEvolve?: boolean | null;
  evolve?: string | null;
  name?: string | null;
  settings: Settings;
  shuffles: {
    [k: string]: Shuffle;
  };
}
export interface Settings {
  /**
   * 0 <= boost_cap <= 1
   */
  boostCap: number;
  boostPriceModifier: number;
  /**
   * Id of the token used to boost shuffles luck
   */
  boostToken: string;
  /**
   * NOTE: Currently only Pianity is allowed to do mints and transfers which means that ownership always defaults to Pianity. This field represents the address to which ownership always defaults in the ERC1155 contract.
   *
   * It is required in order to, for example, determine whether a transfer represents a sell or a resell.
   */
  custodian: string;
  /**
   * Address of the attached ERC1155-compliant contract
   */
  erc1155: string;
  operators: string[];
  paused: boolean;
  superOperators: string[];
}
export interface Shuffle {
  id: string;
  nfts: ShuffleBaseIds;
}
