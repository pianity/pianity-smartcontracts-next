export interface State {
  allAttachedRoyalties: {
    [k: string]: AttachedRoyalties;
  };
  evolve?: string | null;
  name: string;
  settings: Settings;
}
export interface AttachedRoyalties {
  baseId: string;
  rate: number;
  royalties: {
    [k: string]: number;
  };
}
export interface Settings {
  canEvolve: boolean;
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
