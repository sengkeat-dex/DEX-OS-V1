//! Automated Market Maker implementation for the DEX-OS core engine

use crate::types::{TokenId, Quantity};
use std::collections::HashMap;

/// Errors that can occur when working with the AMM
#[derive(Debug, Clone, PartialEq)]
pub enum AMMError {
    InvalidToken,
    InsufficientLiquidity,
    PriceRangeNotFound,
}

impl std::fmt::Display for AMMError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AMMError::InvalidToken => write!(f, "Invalid token"),
            AMMError::InsufficientLiquidity => write!(f, "Insufficient liquidity"),
            AMMError::PriceRangeNotFound => write!(f, "Price range not found"),
        }
    }
}

impl std::error::Error for AMMError {}

/// Constant product AMM implementation (x * y = k)
#[derive(Debug, Clone)]
pub struct ConstantProductAMM {
    /// Reserves of tokens in the pool
    pub reserves: HashMap<TokenId, Quantity>,
    /// Total supply of liquidity tokens
    pub total_supply: Quantity,
    /// Fee percentage (in basis points, so 30 = 0.3%)
    pub fee: u32,
}

impl ConstantProductAMM {
    /// Create a new AMM pool
    pub fn new(fee: u32) -> Self {
        Self {
            reserves: HashMap::new(),
            total_supply: 0,
            fee,
        }
    }

    /// Add liquidity to the pool
    pub fn add_liquidity(
        &mut self,
        token_a: TokenId,
        amount_a: Quantity,
        token_b: TokenId,
        amount_b: Quantity,
    ) -> Result<Quantity, AMMError> {
        // Initialize reserves if they don't exist
        self.reserves.entry(token_a.clone()).or_insert(0);
        self.reserves.entry(token_b.clone()).or_insert(0);

        let reserve_a = self.reserves[&token_a];
        let reserve_b = self.reserves[&token_b];

        let liquidity_tokens = if self.total_supply == 0 {
            // First liquidity provider
            // Calculate initial liquidity tokens as geometric mean
            ((amount_a as f64 * amount_b as f64).sqrt() as Quantity).max(1)
        } else {
            // Subsequent liquidity providers
            // Calculate liquidity tokens based on proportional contribution
            let liquidity_a = (amount_a * self.total_supply) / reserve_a;
            let liquidity_b = (amount_b * self.total_supply) / reserve_b;
            liquidity_a.min(liquidity_b)
        };

        // Update reserves
        *self.reserves.get_mut(&token_a).unwrap() += amount_a;
        *self.reserves.get_mut(&token_b).unwrap() += amount_b;

        // Update total supply
        self.total_supply += liquidity_tokens;

        Ok(liquidity_tokens)
    }

    /// Remove liquidity from the pool
    pub fn remove_liquidity(
        &mut self,
        token_a: TokenId,
        token_b: TokenId,
        liquidity_tokens: Quantity,
    ) -> Result<(Quantity, Quantity), AMMError> {
        if liquidity_tokens > self.total_supply {
            return Err(AMMError::InsufficientLiquidity);
        }

        let reserve_a = *self.reserves.get(&token_a).unwrap_or(&0);
        let reserve_b = *self.reserves.get(&token_b).unwrap_or(&0);

        let amount_a = (liquidity_tokens * reserve_a) / self.total_supply;
        let amount_b = (liquidity_tokens * reserve_b) / self.total_supply;

        // Update reserves
        *self.reserves.get_mut(&token_a).unwrap() -= amount_a;
        *self.reserves.get_mut(&token_b).unwrap() -= amount_b;

        // Update total supply
        self.total_supply -= liquidity_tokens;

        Ok((amount_a, amount_b))
    }

    /// Swap tokens in the pool
    pub fn swap(
        &mut self,
        from_token: TokenId,
        to_token: TokenId,
        amount_in: Quantity,
    ) -> Result<Quantity, AMMError> {
        let reserve_in = *self.reserves.get(&from_token).ok_or(AMMError::InvalidToken)?;
        let reserve_out = *self.reserves.get(&to_token).ok_or(AMMError::InvalidToken)?;

        if reserve_in == 0 || reserve_out == 0 {
            return Err(AMMError::InsufficientLiquidity);
        }

        // Calculate amount out with fee
        let amount_in_with_fee = amount_in * (10000 - self.fee) as u64;
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = (reserve_in * 10000) + amount_in_with_fee;
        let amount_out = numerator / denominator;

        if amount_out >= reserve_out {
            return Err(AMMError::InsufficientLiquidity);
        }

        // Update reserves
        *self.reserves.get_mut(&from_token).unwrap() += amount_in;
        *self.reserves.get_mut(&to_token).unwrap() -= amount_out;

        Ok(amount_out)
    }

    /// Get the price of one token in terms of another
    pub fn get_price(&self, from_token: &TokenId, to_token: &TokenId) -> Result<f64, AMMError> {
        let reserve_in = *self.reserves.get(from_token).ok_or(AMMError::InvalidToken)?;
        let reserve_out = *self.reserves.get(to_token).ok_or(AMMError::InvalidToken)?;

        if reserve_in == 0 {
            return Err(AMMError::InsufficientLiquidity);
        }

        Ok(reserve_out as f64 / reserve_in as f64)
    }

    /// Find the optimal price within a given range using binary search
    /// This implements the Priority 2 feature from DEX-OS-V1.csv:
    /// "Core Trading,AMM,AMM,Binary Search,Price Range Checks,Medium"
    /// 
    /// Uses binary search to find a price within the specified range that meets
    /// the desired criteria for trading.
    /// 
    /// # Arguments
    /// * `from_token` - The token being sold
    /// * `to_token` - The token being bought
    /// * `min_price` - Minimum acceptable price (in to_token/from_token)
    /// * `max_price` - Maximum acceptable price (in to_token/from_token)
    /// * `tolerance` - Tolerance for price matching (as a ratio)
    /// 
    /// # Returns
    /// * `Ok(f64)` - The optimal price within the range
    /// * `Err(AMMError)` - If no suitable price is found or on error
    pub fn find_price_in_range(
        &self,
        from_token: &TokenId,
        to_token: &TokenId,
        min_price: f64,
        max_price: f64,
        tolerance: f64,
    ) -> Result<f64, AMMError> {
        // Validate inputs
        if min_price <= 0.0 || max_price <= 0.0 || tolerance <= 0.0 {
            return Err(AMMError::PriceRangeNotFound);
        }
        
        if min_price > max_price {
            return Err(AMMError::PriceRangeNotFound);
        }
        
        let reserve_in = *self.reserves.get(from_token).ok_or(AMMError::InvalidToken)?;
        let reserve_out = *self.reserves.get(to_token).ok_or(AMMError::InvalidToken)?;
        
        if reserve_in == 0 || reserve_out == 0 {
            return Err(AMMError::InsufficientLiquidity);
        }
        
        // Get current price
        let current_price = reserve_out as f64 / reserve_in as f64;
        
        // Check if current price is within range
        if current_price >= min_price && current_price <= max_price {
            return Ok(current_price);
        }
        
        // If current price is outside the range, use binary search to find
        // a trade amount that would result in a price within range
        let target_price = (min_price + max_price) / 2.0;
        
        // Binary search for the optimal trade amount
        let result = self.binary_search_for_price(
            reserve_in,
            reserve_out,
            target_price,
            tolerance,
            0.0,
            (reserve_in as f64).min(1_000_000.0), // Upper bound for search
        )?;
        
        Ok(result)
    }
    
    /// Helper function that performs binary search to find a trade amount
    /// that results in a desired price
    fn binary_search_for_price(
        &self,
        reserve_in: Quantity,
        reserve_out: Quantity,
        target_price: f64,
        tolerance: f64,
        low: f64,
        high: f64,
    ) -> Result<f64, AMMError> {
        const MAX_ITERATIONS: u32 = 100;
        let mut iterations = 0;
        let mut low = low;
        let mut high = high;
        
        while iterations < MAX_ITERATIONS && (high - low) > 1e-10 {
            let mid = (low + high) / 2.0;
            
            // Calculate resulting price after trading 'mid' amount of input token
            // For constant product formula: x * y = k
            // After trade: (x + mid) * (y - out) = k
            // So: out = y - k / (x + mid) = y - (x * y) / (x + mid)
            let k = (reserve_in as f64) * (reserve_out as f64);
            let new_reserve_in = (reserve_in as f64) + mid;
            
            if new_reserve_in <= 0.0 {
                low = mid;
                iterations += 1;
                continue;
            }
            
            let new_reserve_out = k / new_reserve_in;
            let resulting_price = new_reserve_out / new_reserve_in;
            
            let price_diff = (resulting_price - target_price).abs();
            
            if price_diff <= tolerance {
                return Ok(resulting_price);
            } else if resulting_price > target_price {
                low = mid;
            } else {
                high = mid;
            }
            
            iterations += 1;
        }
        
        // If we couldn't find an exact match, return the best approximation
        let mid = (low + high) / 2.0;
        let k = (reserve_in as f64) * (reserve_out as f64);
        let new_reserve_in = (reserve_in as f64) + mid;
        
        if new_reserve_in <= 0.0 {
            return Err(AMMError::PriceRangeNotFound);
        }
        
        let new_reserve_out = k / new_reserve_in;
        let resulting_price = new_reserve_out / new_reserve_in;
        
        // Check if this is within an acceptable range
        let price_diff = (resulting_price - target_price).abs();
        if price_diff <= tolerance * 10.0 {
            Ok(resulting_price)
        } else {
            Err(AMMError::PriceRangeNotFound)
        }
    }
    
    /// Check if a given price is within acceptable slippage range
    /// This is another implementation of price range checking using binary search concepts
    /// 
    /// # Arguments
    /// * `from_token` - The token being sold
    /// * `to_token` - The token being bought
    /// * `proposed_price` - The proposed trade price
    /// * `max_slippage` - Maximum allowed slippage (as a ratio, e.g., 0.005 for 0.5%)
    /// 
    /// # Returns
    /// * `Ok(bool)` - True if price is within slippage range, false otherwise
    /// * `Err(AMMError)` - On error
    pub fn is_price_within_slippage(
        &self,
        from_token: &TokenId,
        to_token: &TokenId,
        proposed_price: f64,
        max_slippage: f64,
    ) -> Result<bool, AMMError> {
        let current_price = self.get_price(from_token, to_token)?;
        
        // Calculate price impact
        let price_impact = ((current_price - proposed_price) / current_price).abs();
        
        Ok(price_impact <= max_slippage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amm_creation() {
        let amm = ConstantProductAMM::new(30); // 0.3% fee
        assert_eq!(amm.total_supply, 0);
        assert_eq!(amm.fee, 30);
        assert!(amm.reserves.is_empty());
    }

    #[test]
    fn test_add_liquidity() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "BTC".to_string();
        let token_b = "USD".to_string();

        let liquidity_tokens = amm.add_liquidity(
            token_a.clone(),
            1000,
            token_b.clone(),
            50000000, // 50,000,000 USD (assuming 1 BTC = 50,000 USD)
        ).unwrap();

        assert!(liquidity_tokens > 0);
        assert_eq!(amm.total_supply, liquidity_tokens);
        assert_eq!(*amm.reserves.get(&token_a).unwrap(), 1000);
        assert_eq!(*amm.reserves.get(&token_b).unwrap(), 50000000);
    }

    #[test]
    fn test_find_price_in_range() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "DAI".to_string();
        let token_b = "USDC".to_string();

        // Add initial liquidity
        amm.add_liquidity(
            token_a.clone(),
            1000000,  // 1,000,000 DAI
            token_b.clone(),
            1000000,  // 1,000,000 USDC
        ).unwrap();

        // Test finding price in range
        let price = amm.find_price_in_range(
            &token_a,
            &token_b,
            0.9,
            1.1,
            0.001,
        ).unwrap();

        // Price should be close to 1.0 (since we have equal reserves)
        assert!(price > 0.9 && price < 1.1);
    }

    #[test]
    fn test_find_price_in_range_not_found() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "DAI".to_string();
        let token_b = "USDC".to_string();

        // Add initial liquidity
        amm.add_liquidity(
            token_a.clone(),
            1000000,  // 1,000,000 DAI
            token_b.clone(),
            1000000,  // 1,000,000 USDC
        ).unwrap();

        // Test with impossible range
        let result = amm.find_price_in_range(
            &token_a,
            &token_b,
            2.0,
            3.0,
            0.001,
        );

        // Should return an error since we can't achieve such a high price
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AMMError::PriceRangeNotFound);
    }

    #[test]
    fn test_is_price_within_slippage() {
        let mut amm = ConstantProductAMM::new(30);
        let token_a = "DAI".to_string();
        let token_b = "USDC".to_string();

        // Add initial liquidity
        amm.add_liquidity(
            token_a.clone(),
            1000000,  // 1,000,000 DAI
            token_b.clone(),
            1000000,  // 1,000,000 USDC
        ).unwrap();

        // Test price within slippage
        let within_slippage = amm.is_price_within_slippage(
            &token_a,
            &token_b,
            1.0,  // Current price is 1.0
            0.01, // 1% slippage allowed
        ).unwrap();

        assert!(within_slippage);

        // Test price outside slippage
        let within_slippage = amm.is_price_within_slippage(
            &token_a,
            &token_b,
            1.5,  // Much higher than current price
            0.01, // 1% slippage allowed
        ).unwrap();

        assert!(!within_slippage);
    }
}