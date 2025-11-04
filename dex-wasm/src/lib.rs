//! WebAssembly interface for the DEX-OS core engine
//!
//! This module provides the WASM bindings to allow the DEX-OS core engine
//! to be used in web browsers and other WASM environments.

use dex_core::{amm::ConstantProductAMM, orderbook::OrderBook, types::Order};
use wasm_bindgen::prelude::*;

/// WASM wrapper for the OrderBook
#[wasm_bindgen]
pub struct WasmOrderBook {
    inner: OrderBook,
}

#[wasm_bindgen]
impl WasmOrderBook {
    /// Create a new orderbook
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmOrderBook {
        WasmOrderBook {
            inner: OrderBook::new(),
        }
    }

    /// Add an order to the orderbook
    #[wasm_bindgen]
    pub fn add_order(&mut self, order: JsValue) -> Result<JsValue, JsValue> {
        let order: Order = serde_wasm_bindgen::from_value(order)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize order: {}", e)))?;

        match self.inner.add_order(order) {
            Ok(trades) => {
                // Convert trades to JsValue
                let js_trades = serde_wasm_bindgen::to_value(&trades).map_err(|e| {
                    JsValue::from_str(&format!("Failed to serialize trades: {}", e))
                })?;
                Ok(js_trades)
            }
            Err(e) => Err(JsValue::from_str(&format!("Failed to add order: {}", e))),
        }
    }

    /// Get the best bid price
    #[wasm_bindgen]
    pub fn best_bid(&self) -> Option<u64> {
        self.inner.best_bid()
    }

    /// Get the best ask price
    #[wasm_bindgen]
    pub fn best_ask(&self) -> Option<u64> {
        self.inner.best_ask()
    }

    /// Remove an order from the orderbook
    #[wasm_bindgen]
    pub fn remove_order(&mut self, order_id: u64) -> Result<JsValue, JsValue> {
        match self.inner.remove_order(order_id) {
            Ok(order) => {
                let js_order = serde_wasm_bindgen::to_value(&order)
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize order: {}", e)))?;
                Ok(js_order)
            }
            Err(e) => Err(JsValue::from_str(&format!("Failed to remove order: {}", e))),
        }
    }

    /// Lookup an order by its ID
    #[wasm_bindgen]
    pub fn get_order(&self, order_id: u64) -> Result<JsValue, JsValue> {
        match self.inner.get_order(order_id) {
            Some(order) => {
                let js_order = serde_wasm_bindgen::to_value(order)
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize order: {}", e)))?;
                Ok(js_order)
            }
            None => Err(JsValue::from_str("Order not found")),
        }
    }

    /// Generate a Merkle proof for a batch of orders
    #[wasm_bindgen]
    pub fn generate_batch_proof(&self, order_ids: JsValue) -> Result<JsValue, JsValue> {
        let order_ids: Vec<u64> = serde_wasm_bindgen::from_value(order_ids)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize order IDs: {}", e)))?;

        match self.inner.generate_batch_proof(&order_ids) {
            Some(proof) => {
                let js_proof = serde_wasm_bindgen::to_value(&proof)
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize proof: {}", e)))?;
                Ok(js_proof)
            }
            None => Err(JsValue::from_str("Failed to generate batch proof")),
        }
    }
}

/// WASM wrapper for the ConstantProductAMM
#[wasm_bindgen]
pub struct WasmAMM {
    inner: ConstantProductAMM,
}

#[wasm_bindgen]
impl WasmAMM {
    /// Create a new AMM with the specified fee (in basis points)
    #[wasm_bindgen(constructor)]
    pub fn new(fee: u32) -> WasmAMM {
        WasmAMM {
            inner: ConstantProductAMM::new(fee),
        }
    }

    /// Add liquidity to the pool
    #[wasm_bindgen]
    pub fn add_liquidity(
        &mut self,
        token_a: String,
        amount_a: u64,
        token_b: String,
        amount_b: u64,
    ) -> Result<u64, JsValue> {
        self.inner
            .add_liquidity(token_a, amount_a, token_b, amount_b)
            .map_err(|e| JsValue::from_str(&format!("Failed to add liquidity: {}", e)))
    }

    /// Remove liquidity from the pool
    #[wasm_bindgen]
    pub fn remove_liquidity(
        &mut self,
        token_a: String,
        token_b: String,
        liquidity_tokens: u64,
    ) -> Result<JsValue, JsValue> {
        match self
            .inner
            .remove_liquidity(token_a, token_b, liquidity_tokens)
        {
            Ok((amount_a, amount_b)) => {
                let result = serde_json::json!({
                    "amount_a": amount_a,
                    "amount_b": amount_b
                });
                let js_result = serde_wasm_bindgen::to_value(&result).map_err(|e| {
                    JsValue::from_str(&format!("Failed to serialize result: {}", e))
                })?;
                Ok(js_result)
            }
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to remove liquidity: {}",
                e
            ))),
        }
    }

    /// Swap tokens in the pool
    #[wasm_bindgen]
    pub fn swap(
        &mut self,
        from_token: String,
        to_token: String,
        amount_in: u64,
    ) -> Result<u64, JsValue> {
        self.inner
            .swap(from_token, to_token, amount_in)
            .map_err(|e| JsValue::from_str(&format!("Failed to swap: {}", e)))
    }

    /// Get the price of one token in terms of another
    #[wasm_bindgen]
    pub fn get_price(&self, from_token: String, to_token: String) -> Result<f64, JsValue> {
        self.inner
            .get_price(&from_token, &to_token)
            .map_err(|e| JsValue::from_str(&format!("Failed to get price: {}", e)))
    }

    /// Find the optimal price within a given range using binary search
    #[wasm_bindgen]
    pub fn find_price_in_range(
        &self,
        from_token: String,
        to_token: String,
        min_price: f64,
        max_price: f64,
        tolerance: f64,
    ) -> Result<f64, JsValue> {
        self.inner
            .find_price_in_range(&from_token, &to_token, min_price, max_price, tolerance)
            .map_err(|e| JsValue::from_str(&format!("Failed to find price in range: {}", e)))
    }

    /// Check if a given price is within acceptable slippage range
    #[wasm_bindgen]
    pub fn is_price_within_slippage(
        &self,
        from_token: String,
        to_token: String,
        proposed_price: f64,
        max_slippage: f64,
    ) -> Result<bool, JsValue> {
        self.inner
            .is_price_within_slippage(&from_token, &to_token, proposed_price, max_slippage)
            .map_err(|e| {
                JsValue::from_str(&format!("Failed to check price within slippage: {}", e))
            })
    }
}

// The default allocator is used for WASM builds to avoid unmaintained dependencies.
