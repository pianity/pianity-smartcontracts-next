/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export interface State {
  canEvolve?: boolean | null;
  evolve?: string | null;
  name?: string | null;
  settings: Settings;
  vault: {
    [k: string]: LockedBalance[];
  };
}
export interface Settings {
  /**
   * Address of the attached ERC1155-compliant contract
   */
  erc1155: string;
  operators: string[];
  superOperators: string[];
}
export interface LockedBalance {
  at: number;
  duration: number;
  from: string;
  qty: string;
  token_id: string;
}
