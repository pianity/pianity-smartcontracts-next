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
  paused: boolean;
  superOperators: string[];
}
export interface LockedBalance {
  at: number;
  duration: number;
  from: string;
  qty: string;
  token_id: string;
}
