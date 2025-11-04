//! Orderbook implementation for the DEX-OS core engine

use crate::types::{OrderId, Order, OrderSide, Price, Quantity, Trade};
use crate::merkle_tree::MerkleTree;
use std::collections::{BTreeMap, HashMap};

/// Represents a level in the orderbook with a specific price
#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: Price,
    pub orders: Vec<OrderId>,
    pub total_quantity: Quantity,
}

/// Main orderbook structure
#[derive(Debug, Clone)]
pub struct OrderBook {
    /// Buy orders sorted by price (highest first)
    pub bids: BTreeMap<Price, PriceLevel>,
    /// Sell orders sorted by price (lowest first)
    pub asks: BTreeMap<Price, PriceLevel>,
    /// All orders indexed by ID for quick lookup
    pub orders: HashMap<OrderId, Order>,
}

impl OrderBook {
    /// Create a new empty orderbook
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
        }
    }

    /// Add an order to the orderbook and match it against existing orders
    pub fn add_order(&mut self, order: Order) -> Result<Vec<Trade>, OrderBookError> {
        // Store the order
        let order_id = order.id;
        self.orders.insert(order_id, order.clone());

        // Try to match the order
        let trades = self.match_order(&order);

        // Add remaining order to the book if it wasn't fully filled
        if order.quantity > 0 {
            // Add to the appropriate side of the book
            match order.side {
                OrderSide::Buy => self.add_bid(order),
                OrderSide::Sell => self.add_ask(order),
            }
        }

        Ok(trades)
    }

    /// Match an order against existing orders in the book
    fn match_order(&mut self, order: &Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        let mut remaining_quantity = order.quantity;

        match order.side {
            OrderSide::Buy => {
                // Match against asks (sell orders)
                let mut asks_to_remove = Vec::new();
                
                // Iterate through asks in ascending price order
                for (&ask_price, ask_level) in self.asks.iter_mut() {
                    // If this is a limit order and the ask price is higher than our limit, stop matching
                    if let Some(limit_price) = order.price {
                        if ask_price > limit_price {
                            break;
                        }
                    }
                    
                    // Match against orders at this price level
                    let mut orders_to_remove = Vec::new();
                    for &ask_order_id in &ask_level.orders {
                        if let Some(ask_order) = self.orders.get(&ask_order_id) {
                            let trade_quantity = std::cmp::min(remaining_quantity, ask_order.quantity);
                            
                            // Create a trade
                            let trade = Trade {
                                id: 0, // This will be set by the trade ID counter in the API
                                maker_order_id: ask_order_id,
                                taker_order_id: order.id,
                                base_token: ask_order.pair.base.clone(),
                                quote_token: ask_order.pair.quote.clone(),
                                price: ask_price,
                                quantity: trade_quantity,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            };
                            
                            trades.push(trade);
                            remaining_quantity -= trade_quantity;
                            
                            // If the ask order is fully filled, remove it
                            if trade_quantity >= ask_order.quantity {
                                orders_to_remove.push(ask_order_id);
                            } else {
                                // Update the remaining quantity of the ask order
                                if let Some(ask_order_mut) = self.orders.get_mut(&ask_order_id) {
                                    ask_order_mut.quantity -= trade_quantity;
                                }
                            }
                            
                            // If the taker order is fully filled, stop matching
                            if remaining_quantity == 0 {
                                break;
                            }
                        }
                    }
                    
                    // Remove fully filled ask orders
                    for order_id in orders_to_remove {
                        ask_level.orders.retain(|&id| id != order_id);
                        ask_level.total_quantity -= self.orders.get(&order_id).map(|o| o.quantity).unwrap_or(0);
                        self.orders.remove(&order_id);
                    }
                    
                    // If the price level is empty, mark it for removal
                    if ask_level.orders.is_empty() {
                        asks_to_remove.push(ask_price);
                    }
                    
                    // If the taker order is fully filled, stop matching
                    if remaining_quantity == 0 {
                        break;
                    }
                }
                
                // Remove empty ask price levels
                for price in asks_to_remove {
                    self.asks.remove(&price);
                }
            }
            OrderSide::Sell => {
                // Match against bids (buy orders)
                let mut bids_to_remove = Vec::new();
                
                // Iterate through bids in descending price order
                for (&bid_price, bid_level) in self.bids.iter_mut().rev() {
                    // If this is a limit order and the bid price is lower than our limit, stop matching
                    if let Some(limit_price) = order.price {
                        if bid_price < limit_price {
                            break;
                        }
                    }
                    
                    // Match against orders at this price level
                    let mut orders_to_remove = Vec::new();
                    for &bid_order_id in &bid_level.orders {
                        if let Some(bid_order) = self.orders.get(&bid_order_id) {
                            let trade_quantity = std::cmp::min(remaining_quantity, bid_order.quantity);
                            
                            // Create a trade
                            let trade = Trade {
                                id: 0, // This will be set by the trade ID counter in the API
                                maker_order_id: bid_order_id,
                                taker_order_id: order.id,
                                base_token: bid_order.pair.base.clone(),
                                quote_token: bid_order.pair.quote.clone(),
                                price: bid_price,
                                quantity: trade_quantity,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            };
                            
                            trades.push(trade);
                            remaining_quantity -= trade_quantity;
                            
                            // If the bid order is fully filled, remove it
                            if trade_quantity >= bid_order.quantity {
                                orders_to_remove.push(bid_order_id);
                            } else {
                                // Update the remaining quantity of the bid order
                                if let Some(bid_order_mut) = self.orders.get_mut(&bid_order_id) {
                                    bid_order_mut.quantity -= trade_quantity;
                                }
                            }
                            
                            // If the taker order is fully filled, stop matching
                            if remaining_quantity == 0 {
                                break;
                            }
                        }
                    }
                    
                    // Remove fully filled bid orders
                    for order_id in orders_to_remove {
                        bid_level.orders.retain(|&id| id != order_id);
                        bid_level.total_quantity -= self.orders.get(&order_id).map(|o| o.quantity).unwrap_or(0);
                        self.orders.remove(&order_id);
                    }
                    
                    // If the price level is empty, mark it for removal
                    if bid_level.orders.is_empty() {
                        bids_to_remove.push(bid_price);
                    }
                    
                    // If the taker order is fully filled, stop matching
                    if remaining_quantity == 0 {
                        break;
                    }
                }
                
                // Remove empty bid price levels
                for price in bids_to_remove {
                    self.bids.remove(&price);
                }
            }
        }

        trades
    }

    /// Add a bid order to the orderbook
    fn add_bid(&mut self, order: Order) {
        if let Some(price) = order.price {
            self.bids.entry(price).and_modify(|level| {
                level.orders.push(order.id);
                level.total_quantity += order.quantity;
            }).or_insert(PriceLevel {
                price,
                orders: vec![order.id],
                total_quantity: order.quantity,
            });
        }
    }

    /// Add an ask order to the orderbook
    fn add_ask(&mut self, order: Order) {
        if let Some(price) = order.price {
            self.asks.entry(price).and_modify(|level| {
                level.orders.push(order.id);
                level.total_quantity += order.quantity;
            }).or_insert(PriceLevel {
                price,
                orders: vec![order.id],
                total_quantity: order.quantity,
            });
        }
    }

    /// Remove an order from the orderbook
    pub fn remove_order(&mut self, order_id: OrderId) -> Result<Order, OrderBookError> {
        let order = self.orders.remove(&order_id)
            .ok_or(OrderBookError::OrderNotFound)?;

        match order.side {
            OrderSide::Buy => self.remove_bid(order_id, order.price.unwrap_or(0)),
            OrderSide::Sell => self.remove_ask(order_id, order.price.unwrap_or(0)),
        }

        Ok(order)
    }

    /// Remove a bid order from the orderbook
    fn remove_bid(&mut self, order_id: OrderId, price: Price) {
        if let Some(level) = self.bids.get_mut(&price) {
            level.orders.retain(|&id| id != order_id);
            // Note: In a production implementation, we would recalculate total_quantity
            // based on remaining orders to avoid floating point errors
        }
    }

    /// Remove an ask order from the orderbook
    fn remove_ask(&mut self, order_id: OrderId, price: Price) {
        if let Some(level) = self.asks.get_mut(&price) {
            level.orders.retain(|&id| id != order_id);
            // Note: In a production implementation, we would recalculate total_quantity
            // based on remaining orders to avoid floating point errors
        }
    }

    /// Get the best bid price (highest buy order)
    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    /// Get the best ask price (lowest sell order)
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    /// Lookup an order by its ID
    /// This implements the Priority 2 feature from DEX-OS-V1.csv:
    /// "Core Trading,Orderbook,Orderbook,Hash Map,Order ID Lookup,Medium"
    pub fn get_order(&self, order_id: OrderId) -> Option<&Order> {
        self.orders.get(&order_id)
    }

    /// Generate a Merkle proof for a batch of orders
    /// This implements the Priority 2 feature from DEX-OS-V1.csv for Batch Order Proofs
    pub fn generate_batch_proof(&self, order_ids: &[OrderId]) -> Option<Vec<u8>> {
        // Collect the orders for the batch
        let mut orders: Vec<Order> = Vec::new();
        for &order_id in order_ids {
            if let Some(order) = self.orders.get(&order_id) {
                orders.push(order.clone());
            } else {
                // If any order is not found, we can't generate a proof for this batch
                return None;
            }
        }

        // Convert orders to serialized data for the Merkle tree
        let mut order_data: Vec<Vec<u8>> = Vec::new();
        for order in orders {
            // In a real implementation, you would serialize the order properly
            // For this example, we'll use a simple representation
            let data = format!("{}:{}:{}:{}", order.id, order.trader_id, order.quantity, order.timestamp);
            order_data.push(data.into_bytes());
        }

        // Create the Merkle tree
        let tree = MerkleTree::from_data(&order_data);
        
        // Return the root hash as the batch proof
        tree.root_hash()
    }
}

/// Errors that can occur when working with the orderbook
#[derive(Debug, thiserror::Error)]
pub enum OrderBookError {
    #[error("Order not found")]
    OrderNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TradingPair, OrderType};

    #[test]
    fn test_orderbook_creation() {
        let orderbook = OrderBook::new();
        assert!(orderbook.bids.is_empty());
        assert!(orderbook.asks.is_empty());
        assert!(orderbook.orders.is_empty());
    }

    #[test]
    fn test_add_order() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        let order = Order {
            id: 1,
            trader_id: "trader1".to_string(),
            pair,
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 100,
            timestamp: 1234567890,
        };

        assert!(orderbook.add_order(order).is_ok());
        assert_eq!(orderbook.orders.len(), 1);
        assert_eq!(orderbook.bids.len(), 1);
        assert_eq!(orderbook.asks.len(), 0);
    }

    #[test]
    fn test_batch_proof_generation() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        // Add multiple orders
        let order1 = Order {
            id: 1,
            trader_id: "trader1".to_string(),
            pair: pair.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 100,
            timestamp: 1234567890,
        };

        let order2 = Order {
            id: 2,
            trader_id: "trader2".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Some(51000),
            quantity: 200,
            timestamp: 1234567891,
        };

        assert!(orderbook.add_order(order1).is_ok());
        assert!(orderbook.add_order(order2).is_ok());

        // Generate a batch proof
        let order_ids = vec![1, 2];
        let proof = orderbook.generate_batch_proof(&order_ids);
        assert!(proof.is_some());

        // Try to generate a proof with a non-existent order
        let invalid_order_ids = vec![1, 3];
        let invalid_proof = orderbook.generate_batch_proof(&invalid_order_ids);
        assert!(invalid_proof.is_none());
    }

    #[test]
    fn test_order_lookup() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        let order = Order {
            id: 1,
            trader_id: "trader1".to_string(),
            pair,
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 100,
            timestamp: 1234567890,
        };

        // Add order to orderbook
        assert!(orderbook.add_order(order.clone()).is_ok());

        // Lookup the order by ID
        let found_order = orderbook.get_order(1);
        assert!(found_order.is_some());
        assert_eq!(found_order.unwrap().id, 1);
        assert_eq!(found_order.unwrap().trader_id, "trader1");
        assert_eq!(found_order.unwrap().quantity, 100);

        // Try to lookup a non-existent order
        let not_found = orderbook.get_order(999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_order_matching() {
        let mut orderbook = OrderBook::new();
        let pair = TradingPair {
            base: "BTC".to_string(),
            quote: "USD".to_string(),
        };

        // Add a sell order
        let sell_order = Order {
            id: 1,
            trader_id: "seller".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 100,
            timestamp: 1234567890,
        };

        assert!(orderbook.add_order(sell_order).is_ok());

        // Add a buy order that matches
        let buy_order = Order {
            id: 2,
            trader_id: "buyer".to_string(),
            pair: pair.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: Some(50000),
            quantity: 50,
            timestamp: 1234567891,
        };

        let trades = orderbook.add_order(buy_order).unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 50000);
        assert_eq!(trades[0].quantity, 50);
        assert_eq!(trades[0].maker_order_id, 1);
        assert_eq!(trades[0].taker_order_id, 2);

        // Check that the sell order was partially filled
        let remaining_sell_order = orderbook.get_order(1);
        assert!(remaining_sell_order.is_some());
        assert_eq!(remaining_sell_order.unwrap().quantity, 50);
    }
}