//! Database layer for the DEX-OS core engine
//!
//! This module provides database functionality for persisting orders,
//! trades, and other DEX-related data.

use dex_core::types::{Order, OrderId, Trade, TradeId, TraderId, TradingPair};
use sqlx_core::{query::query, row::Row};
use sqlx_postgres::{PgPool, PgPoolOptions};
use thiserror::Error;

pub mod migrations;

/// Database manager for the DEX
#[derive(Clone)]
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    /// Establish a new connection pool using the provided database URL.
    pub async fn connect(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    /// Create a new database manager with the provided connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize the database schema
    pub async fn initialize(&self) -> Result<(), DatabaseError> {
        // Run migrations
        migrations::run_migrations(&self.pool).await?;
        Ok(())
    }

    /// Save an order to the database
    pub async fn save_order(&self, order: &Order) -> Result<(), DatabaseError> {
        query(
            r#"
            INSERT INTO orders (
                id, trader_id, base_token, quote_token, side, order_type, price, quantity, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                trader_id = $2,
                base_token = $3,
                quote_token = $4,
                side = $5,
                order_type = $6,
                price = $7,
                quantity = $8,
                timestamp = $9
            "#,
        )
        .bind(order.id as i64)
        .bind(&order.trader_id)
        .bind(&order.pair.base)
        .bind(&order.pair.quote)
        .bind(match order.side {
            dex_core::types::OrderSide::Buy => "buy",
            dex_core::types::OrderSide::Sell => "sell",
        })
        .bind(match order.order_type {
            dex_core::types::OrderType::Limit => "limit",
            dex_core::types::OrderType::Market => "market",
        })
        .bind(order.price.map(|p| p as i64))
        .bind(order.quantity as i64)
        .bind(order.timestamp as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load an order from the database by ID
    pub async fn load_order(&self, order_id: OrderId) -> Result<Option<Order>, DatabaseError> {
        let row = query(
            r#"
            SELECT 
                id, trader_id, base_token, quote_token, side, order_type, price, quantity, timestamp
            FROM orders
            WHERE id = $1
            "#,
        )
        .bind(order_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let price: Option<i64> = row.get("price");
            let order = Order {
                id: row.get::<i64, _>("id") as u64,
                trader_id: row.get("trader_id"),
                pair: TradingPair {
                    base: row.get("base_token"),
                    quote: row.get("quote_token"),
                },
                side: match row.get::<&str, _>("side") {
                    "buy" => dex_core::types::OrderSide::Buy,
                    "sell" => dex_core::types::OrderSide::Sell,
                    _ => return Err(DatabaseError::DataIntegrityError),
                },
                order_type: match row.get::<&str, _>("order_type") {
                    "limit" => dex_core::types::OrderType::Limit,
                    "market" => dex_core::types::OrderType::Market,
                    _ => return Err(DatabaseError::DataIntegrityError),
                },
                price: price.map(|p| p as u64),
                quantity: row.get::<i64, _>("quantity") as u64,
                timestamp: row.get::<i64, _>("timestamp") as u64,
            };
            Ok(Some(order))
        } else {
            Ok(None)
        }
    }

    /// Delete an order from the database
    pub async fn delete_order(&self, order_id: OrderId) -> Result<bool, DatabaseError> {
        let result = query("DELETE FROM orders WHERE id = $1")
            .bind(order_id as i64)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Save a trade to the database
    pub async fn save_trade(&self, trade: &Trade) -> Result<(), DatabaseError> {
        query(
            r#"
            INSERT INTO trades (
                id, maker_order_id, taker_order_id, base_token, quote_token, price, quantity, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(trade.id as i64)
        .bind(trade.maker_order_id as i64)
        .bind(trade.taker_order_id as i64)
        .bind(&trade.base_token)
        .bind(&trade.quote_token)
        .bind(trade.price as i64)
        .bind(trade.quantity as i64)
        .bind(trade.timestamp as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load a trade from the database by ID
    pub async fn load_trade(&self, trade_id: TradeId) -> Result<Option<Trade>, DatabaseError> {
        let row = query(
            r#"
            SELECT 
                id, maker_order_id, taker_order_id, base_token, quote_token, price, quantity, timestamp
            FROM trades
            WHERE id = $1
            "#,
        )
        .bind(trade_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let trade = Trade {
                id: row.get::<i64, _>("id") as u64,
                maker_order_id: row.get::<i64, _>("maker_order_id") as u64,
                taker_order_id: row.get::<i64, _>("taker_order_id") as u64,
                base_token: row.get("base_token"),
                quote_token: row.get("quote_token"),
                price: row.get::<i64, _>("price") as u64,
                quantity: row.get::<i64, _>("quantity") as u64,
                timestamp: row.get::<i64, _>("timestamp") as u64,
            };
            Ok(Some(trade))
        } else {
            Ok(None)
        }
    }

    /// Get all trades for a specific order
    pub async fn get_trades_for_order(
        &self,
        order_id: OrderId,
    ) -> Result<Vec<Trade>, DatabaseError> {
        let rows = query(
            r#"
            SELECT 
                id, maker_order_id, taker_order_id, base_token, quote_token, price, quantity, timestamp
            FROM trades
            WHERE maker_order_id = $1 OR taker_order_id = $1
            ORDER BY timestamp ASC
            "#,
        )
        .bind(order_id as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut trades = Vec::new();
        for row in rows {
            let trade = Trade {
                id: row.get::<i64, _>("id") as u64,
                maker_order_id: row.get::<i64, _>("maker_order_id") as u64,
                taker_order_id: row.get::<i64, _>("taker_order_id") as u64,
                base_token: row.get("base_token"),
                quote_token: row.get("quote_token"),
                price: row.get::<i64, _>("price") as u64,
                quantity: row.get::<i64, _>("quantity") as u64,
                timestamp: row.get::<i64, _>("timestamp") as u64,
            };
            trades.push(trade);
        }

        Ok(trades)
    }

    /// Get all trades for a specific trader
    pub async fn get_trades_for_trader(
        &self,
        trader_id: &TraderId,
    ) -> Result<Vec<Trade>, DatabaseError> {
        let rows = query(
            r#"
            SELECT 
                t.id, t.maker_order_id, t.taker_order_id, t.base_token, t.quote_token, t.price, t.quantity, t.timestamp
            FROM trades t
            JOIN orders o1 ON t.maker_order_id = o1.id
            JOIN orders o2 ON t.taker_order_id = o2.id
            WHERE o1.trader_id = $1 OR o2.trader_id = $1
            ORDER BY t.timestamp ASC
            "#,
        )
        .bind(trader_id)
        .fetch_all(&self.pool)
        .await?;

        let mut trades = Vec::new();
        for row in rows {
            let trade = Trade {
                id: row.get::<i64, _>("id") as u64,
                maker_order_id: row.get::<i64, _>("maker_order_id") as u64,
                taker_order_id: row.get::<i64, _>("taker_order_id") as u64,
                base_token: row.get("base_token"),
                quote_token: row.get("quote_token"),
                price: row.get::<i64, _>("price") as u64,
                quantity: row.get::<i64, _>("quantity") as u64,
                timestamp: row.get::<i64, _>("timestamp") as u64,
            };
            trades.push(trade);
        }

        Ok(trades)
    }
}

/// Errors that can occur when working with the database
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx_core::error::Error),
    #[error("Data integrity error")]
    DataIntegrityError,
}

#[cfg(test)]
impl DatabaseManager {
    /// Create a database manager backed by a lazily connected pool. Useful in tests that do
    /// not exercise the database but need a handle for wiring filters.
    pub fn connect_lazy(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy(database_url)?;
        Ok(Self { pool })
    }
}
