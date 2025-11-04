//! Common types used throughout the DEX-OS core engine

use serde::{Deserialize, Serialize};

/// Unique identifier for orders
pub type OrderId = u64;

/// Unique identifier for traders
pub type TraderId = String;

/// Price representation
pub type Price = u64;

/// Quantity representation
pub type Quantity = u64;

/// Token identifier
pub type TokenId = String;

/// Unique identifier for trades
pub type TradeId = u64;

/// Order side (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order type (limit or market)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

/// Represents a trading pair
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradingPair {
    pub base: TokenId,
    pub quote: TokenId,
}

/// Represents an order in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub trader_id: TraderId,
    pub pair: TradingPair,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<Price>,
    pub quantity: Quantity,
    pub timestamp: u64,
}

/// Represents a trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: TradeId,
    pub maker_order_id: OrderId,
    pub taker_order_id: OrderId,
    pub base_token: TokenId,
    pub quote_token: TokenId,
    pub price: Price,
    pub quantity: Quantity,
    pub timestamp: u64,
}