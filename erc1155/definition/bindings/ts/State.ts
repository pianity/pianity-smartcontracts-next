export interface State {
  approvals: {
    [k: string]: {
      [k: string]: boolean;
    };
  };
  canEvolve?: boolean | null;
  defaultToken: string;
  evolve?: string | null;
  name: string;
  settings: Settings;
  tickerNonce: number;
  tokens: {
    [k: string]: Token;
  };
}
export interface Settings {
  allowFreeTransfer: boolean;
  operators: string[];
  paused: boolean;
  proxies: string[];
  superOperators: string[];
}
export interface Token {
  balances: {
    [k: string]: string;
  };
  ticker: string;
  txId?: string | null;
}
