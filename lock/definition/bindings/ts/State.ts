export interface State {
  evolve?: string | null;
  name: string;
  settings: Settings;
  vault: {
    [k: string]: LockedBalance[];
  };
}
export interface Settings {
  canEvolve: boolean;
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
