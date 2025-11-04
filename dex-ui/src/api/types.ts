export type OrderSide = "buy" | "sell";
export type OrderType = "limit" | "market";

export interface ApiPriceResponse {
  best_bid: number | null;
  best_ask: number | null;
}

export interface ApiCreateOrderRequest {
  trader_id: string;
  base_token: string;
  quote_token: string;
  side: OrderSide;
  order_type: OrderType;
  price?: number;
  quantity: number;
}

export interface ApiCreateOrderResponse {
  order_id: number;
  success: boolean;
  message?: string | null;
}

export interface ApiTrade {
  id: number;
  maker_order_id: number;
  taker_order_id: number;
  base_token: string;
  quote_token: string;
  price: number;
  quantity: number;
  timestamp: number;
}

export interface ApiGetTradesResponse {
  trades: ApiTrade[];
  success: boolean;
  message?: string | null;
}

export interface ApiDepthLevel {
  price: number;
  quantity: number;
}

export interface ApiDepthSnapshot {
  bids: ApiDepthLevel[];
  asks: ApiDepthLevel[];
  best_bid: number | null;
  best_ask: number | null;
  timestamp: number;
}

export interface ApiTokenResponse {
  token: string;
  expires_at: number;
}

export interface ApiSharedTokenRequest {
  trader_id: string;
  secret: string;
  ttl_seconds?: number;
  audience?: string;
}

export interface ApiWalletChallengeResponse {
  challenge: string;
  expires_at: number;
}

export interface ApiWalletTokenRequest {
  address: string;
  signature: string;
  ttl_seconds?: number;
  audience?: string;
}
