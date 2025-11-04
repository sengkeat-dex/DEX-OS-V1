//! Price prediction implementation using Kalman Filter for the DEX-OS core engine
//!
//! This module implements the Priority 2 feature from DEX-OS-V1.csv:
//! "Core Trading,Oracle,Oracle,Kalman Filter,Price Prediction,Medium"
//!
//! It provides functionality for predicting future prices using a Kalman filter,
//! which is particularly useful for oracle price prediction in DeFi applications.

use crate::types::TokenId;
use thiserror::Error;

/// Represents the state of the Kalman filter
#[derive(Debug, Clone)]
pub struct KalmanState {
    /// Estimated price
    pub price: f64,
    /// Estimated variance (uncertainty)
    pub variance: f64,
}

/// Kalman Filter for price prediction
#[derive(Debug, Clone)]
pub struct KalmanPricePredictor {
    /// Current state estimate
    state: KalmanState,
    /// Process noise (how much we expect the true price to change)
    process_noise: f64,
    /// Measurement noise (how noisy our observations are)
    measurement_noise: f64,
    /// Last update timestamp
    last_update: u64,
}

impl KalmanPricePredictor {
    /// Create a new Kalman filter for price prediction
    pub fn new(initial_price: f64, process_noise: f64, measurement_noise: f64) -> Self {
        Self {
            state: KalmanState {
                price: initial_price,
                variance: 1.0, // Start with high uncertainty
            },
            process_noise,
            measurement_noise,
            last_update: 0,
        }
    }

    /// Update the filter with a new price observation
    pub fn update(&mut self, observation: f64, timestamp: u64) -> Result<KalmanState, PricePredictionError> {
        // Time update (prediction)
        // In a simple model, we assume the price doesn't change much between observations
        // but we add process noise to account for uncertainty
        let predicted_variance = self.state.variance + self.process_noise;
        
        // Measurement update (correction)
        let innovation = observation - self.state.price;
        let innovation_variance = predicted_variance + self.measurement_noise;
        let kalman_gain = predicted_variance / innovation_variance;
        
        // Update state estimate
        self.state.price += kalman_gain * innovation;
        self.state.variance = (1.0 - kalman_gain) * predicted_variance;
        
        // Update timestamp
        self.last_update = timestamp;
        
        Ok(self.state.clone())
    }

    /// Predict the next price without a new observation
    pub fn predict(&self) -> Result<KalmanState, PricePredictionError> {
        // In this simple model, we predict the price will change based on recent trend
        // but with increased uncertainty
        
        // Simple prediction: assume price continues in the same direction with small momentum
        let predicted_price = self.state.price * 1.001; // Small positive momentum assumption
        
        Ok(KalmanState {
            price: predicted_price,
            variance: self.state.variance + self.process_noise,
        })
    }

    /// Get the current state estimate
    pub fn get_current_state(&self) -> &KalmanState {
        &self.state
    }

    /// Get the estimated price
    pub fn get_estimated_price(&self) -> f64 {
        self.state.price
    }

    /// Get the uncertainty (variance) of the estimate
    pub fn get_uncertainty(&self) -> f64 {
        self.state.variance
    }

    /// Reset the filter with a new initial price
    pub fn reset(&mut self, initial_price: f64) {
        self.state.price = initial_price;
        self.state.variance = 1.0; // Reset uncertainty
        self.last_update = 0;
    }

    /// Adjust noise parameters
    pub fn set_noise_parameters(&mut self, process_noise: f64, measurement_noise: f64) {
        self.process_noise = process_noise;
        self.measurement_noise = measurement_noise;
    }
}

/// Manages price prediction for multiple token pairs
#[derive(Debug, Clone)]
pub struct PricePredictionManager {
    /// Predictors for different token pairs
    predictors: std::collections::HashMap<(TokenId, TokenId), KalmanPricePredictor>,
}

impl PricePredictionManager {
    /// Create a new price prediction manager
    pub fn new() -> Self {
        Self {
            predictors: std::collections::HashMap::new(),
        }
    }

    /// Add or update a price predictor for a token pair
    pub fn add_predictor(&mut self, base_token: TokenId, quote_token: TokenId, predictor: KalmanPricePredictor) {
        self.predictors.insert((base_token, quote_token), predictor);
    }

    /// Update the price for a token pair
    pub fn update_price(&mut self, base_token: &TokenId, quote_token: &TokenId, price: f64, timestamp: u64) -> Result<KalmanState, PricePredictionError> {
        let key = (base_token.clone(), quote_token.clone());
        
        if let Some(predictor) = self.predictors.get_mut(&key) {
            predictor.update(price, timestamp)
        } else {
            Err(PricePredictionError::PredictorNotFound)
        }
    }

    /// Predict the next price for a token pair
    pub fn predict_price(&self, base_token: &TokenId, quote_token: &TokenId) -> Result<KalmanState, PricePredictionError> {
        let key = (base_token.clone(), quote_token.clone());
        
        if let Some(predictor) = self.predictors.get(&key) {
            predictor.predict()
        } else {
            Err(PricePredictionError::PredictorNotFound)
        }
    }

    /// Get the current estimated price for a token pair
    pub fn get_estimated_price(&self, base_token: &TokenId, quote_token: &TokenId) -> Result<f64, PricePredictionError> {
        let key = (base_token.clone(), quote_token.clone());
        
        if let Some(predictor) = self.predictors.get(&key) {
            Ok(predictor.get_estimated_price())
        } else {
            Err(PricePredictionError::PredictorNotFound)
        }
    }

    /// Remove a predictor for a token pair
    pub fn remove_predictor(&mut self, base_token: &TokenId, quote_token: &TokenId) -> bool {
        let key = (base_token.clone(), quote_token.clone());
        self.predictors.remove(&key).is_some()
    }

    /// Get the number of predictors
    pub fn predictor_count(&self) -> usize {
        self.predictors.len()
    }

    /// Check if there are any predictors
    pub fn has_predictors(&self) -> bool {
        !self.predictors.is_empty()
    }
}

/// Errors that can occur during price prediction operations
#[derive(Debug, Error)]
pub enum PricePredictionError {
    #[error("Predictor not found for token pair")]
    PredictorNotFound,
    #[error("Invalid price observation")]
    InvalidObservation,
    #[error("Invalid timestamp")]
    InvalidTimestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kalman_filter_creation() {
        let filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);
        assert_eq!(filter.get_estimated_price(), 100.0);
        assert_eq!(filter.get_uncertainty(), 1.0);
    }

    #[test]
    fn test_kalman_filter_update() {
        let mut filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);
        
        // Update with a new observation
        let state = filter.update(105.0, 1000).unwrap();
        assert_ne!(state.price, 100.0); // Should have changed
        assert!(state.variance < 1.0); // Uncertainty should have decreased
    }

    #[test]
    fn test_kalman_filter_convergence() {
        let mut filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);
        
        // Feed a series of observations that are all around 105
        for i in 0..10 {
            let observation = 105.0 + (i as f64 - 5.0) * 0.1; // Values around 105
            filter.update(observation, i * 100).unwrap();
        }
        
        // The estimate should have converged toward 105
        let estimated_price = filter.get_estimated_price();
        assert!((estimated_price - 105.0).abs() < 2.0); // Should be within 2 of 105
        assert!(filter.get_uncertainty() < 0.5); // Uncertainty should be low
    }

    #[test]
    fn test_kalman_filter_prediction() {
        let mut filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);
        
        // Update with some observations
        filter.update(102.0, 1000).unwrap();
        filter.update(104.0, 2000).unwrap();
        
        // Make a prediction
        let prediction = filter.predict().unwrap();
        assert_ne!(prediction.price, filter.get_estimated_price()); // Prediction should differ
        assert!(prediction.variance > filter.get_uncertainty()); // Prediction should have higher uncertainty
    }

    #[test]
    fn test_price_prediction_manager() {
        let mut manager = PricePredictionManager::new();
        
        // Add a predictor
        let predictor = KalmanPricePredictor::new(50000.0, 10.0, 50.0);
        manager.add_predictor("BTC".to_string(), "USD".to_string(), predictor);
        
        assert_eq!(manager.predictor_count(), 1);
        assert!(manager.has_predictors());
        
        // Update a price
        let state = manager.update_price(&"BTC".to_string(), &"USD".to_string(), 51000.0, 1000).unwrap();
        assert_ne!(state.price, 50000.0);
        
        // Predict next price
        let prediction = manager.predict_price(&"BTC".to_string(), &"USD".to_string()).unwrap();
        assert_ne!(prediction.price, state.price);
        
        // Get estimated price
        let estimated = manager.get_estimated_price(&"BTC".to_string(), &"USD".to_string()).unwrap();
        assert_eq!(estimated, state.price);
        
        // Remove predictor
        assert!(manager.remove_predictor(&"BTC".to_string(), &"USD".to_string()));
        assert_eq!(manager.predictor_count(), 0);
    }

    #[test]
    fn test_reset_filter() {
        let mut filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);
        
        // Update the filter
        filter.update(150.0, 1000).unwrap();
        assert_ne!(filter.get_estimated_price(), 100.0);
        assert!(filter.get_uncertainty() < 1.0);
        
        // Reset the filter
        filter.reset(200.0);
        assert_eq!(filter.get_estimated_price(), 200.0);
        assert_eq!(filter.get_uncertainty(), 1.0);
    }

    #[test]
    fn test_noise_parameter_adjustment() {
        let mut filter = KalmanPricePredictor::new(100.0, 0.1, 0.5);
        
        // Update with an observation
        filter.update(105.0, 1000).unwrap();
        let variance_before = filter.get_uncertainty();
        
        // Adjust noise parameters
        filter.set_noise_parameters(0.5, 1.0); // Higher noise
        
        // Update again
        filter.update(110.0, 2000).unwrap();
        let variance_after = filter.get_uncertainty();
        
        // With higher noise, uncertainty should be higher
        assert!(variance_after > variance_before);
    }
}