//! Path routing implementation using Bellman-Ford algorithm for the DEX-OS core engine
//!
//! This module implements the Priority 2 feature from DEX-OS-V1.csv:
//! "Core Trading,DEX Aggregator,DEX Aggregator,Bellman-Ford,Path Routing,Medium"
//!
//! It provides functionality for finding the best trading paths across multiple DEXes
//! using the Bellman-Ford algorithm to handle negative weight edges (which can represent
//! arbitrage opportunities or fees).

use crate::types::{TokenId, Quantity};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

/// Represents an edge in the trading graph (a trading path between tokens on a DEX)
#[derive(Debug, Clone, PartialEq)]
pub struct TradingEdge {
    /// Source token
    pub from_token: TokenId,
    /// Destination token
    pub to_token: TokenId,
    /// DEX where this trading pair is available
    pub dex_name: String,
    /// Exchange rate (how much to_token you get for 1 from_token)
    pub exchange_rate: f64,
    /// Fee percentage (0.0 to 1.0)
    pub fee: f64,
    /// Liquidity available for this trading pair
    pub liquidity: Quantity,
}

/// Represents a node in the trading graph (a token)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradingNode {
    /// Token identifier
    pub token_id: TokenId,
}

/// Result of a path routing calculation
#[derive(Debug, Clone)]
pub struct RoutingPath {
    /// Sequence of edges that form the path
    pub edges: Vec<TradingEdge>,
    /// Total exchange rate for the path (cumulative)
    pub total_exchange_rate: f64,
    /// Total fee for the path (cumulative)
    pub total_fee: f64,
    /// Minimum liquidity along the path
    pub min_liquidity: Quantity,
}

/// Manages path routing using the Bellman-Ford algorithm
#[derive(Debug, Clone)]
pub struct PathRouter {
    /// Graph representation: token -> list of outgoing edges
    graph: HashMap<TokenId, Vec<TradingEdge>>,
    /// All tokens in the graph
    tokens: Vec<TokenId>,
}

impl PathRouter {
    /// Create a new path router
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
            tokens: Vec::new(),
        }
    }

    /// Add a trading edge to the graph
    pub fn add_edge(&mut self, edge: TradingEdge) {
        // Add source token if not already present
        if !self.tokens.contains(&edge.from_token) {
            self.tokens.push(edge.from_token.clone());
        }
        
        // Add destination token if not already present
        if !self.tokens.contains(&edge.to_token) {
            self.tokens.push(edge.to_token.clone());
        }
        
        // Add edge to the graph
        self.graph.entry(edge.from_token.clone()).or_insert_with(Vec::new).push(edge);
    }

    /// Remove all edges for a specific DEX
    pub fn remove_dex_edges(&mut self, dex_name: &str) {
        for edges in self.graph.values_mut() {
            edges.retain(|edge| edge.dex_name != dex_name);
        }
        
        // Clean up empty edge lists
        self.graph.retain(|_, edges| !edges.is_empty());
    }

    /// Find the best path from source to destination token using Bellman-Ford algorithm
    /// 
    /// This implementation can handle negative weights (which might represent arbitrage
    /// opportunities) and will detect negative cycles.
    pub fn find_best_path(&self, source: &TokenId, destination: &TokenId, amount: f64) -> Result<Option<RoutingPath>, PathRoutingError> {
        if source == destination {
            return Ok(None); // No path needed
        }
        
        if self.tokens.is_empty() {
            return Ok(None); // No tokens in graph
        }
        
        // Initialize distances and predecessors
        let mut distances: HashMap<&TokenId, f64> = HashMap::new();
        let mut predecessors: HashMap<&TokenId, (&TokenId, usize)> = HashMap::new(); // (predecessor token, edge index)
        
        // Initialize all distances to infinity except source
        for token in &self.tokens {
            distances.insert(token, if token == source { 0.0 } else { f64::INFINITY });
        }
        
        // Relax edges repeatedly
        for _ in 0..self.tokens.len() - 1 {
            for (from_token, edges) in &self.graph {
                let from_distance = *distances.get(from_token).unwrap_or(&f64::INFINITY);
                
                if from_distance == f64::INFINITY {
                    continue; // Skip unreachable nodes
                }
                
                for (edge_index, edge) in edges.iter().enumerate() {
                    let to_distance = *distances.get(&edge.to_token).unwrap_or(&f64::INFINITY);
                    // Weight is negative log of exchange rate (to convert multiplication to addition)
                    // Lower exchange rate means higher "distance" (worse path)
                    let weight = -edge.exchange_rate.ln();
                    let new_distance = from_distance + weight;
                    
                    if new_distance < to_distance {
                        distances.insert(&edge.to_token, new_distance);
                        predecessors.insert(&edge.to_token, (from_token, edge_index));
                    }
                }
            }
        }
        
        // Check for negative cycles
        for (from_token, edges) in &self.graph {
            let from_distance = *distances.get(from_token).unwrap_or(&f64::INFINITY);
            
            if from_distance == f64::INFINITY {
                continue;
            }
            
            for edge in edges {
                let to_distance = *distances.get(&edge.to_token).unwrap_or(&f64::INFINITY);
                let weight = -edge.exchange_rate.ln();
                let new_distance = from_distance + weight;
                
                if new_distance < to_distance {
                    // Negative cycle detected - this could represent an arbitrage opportunity
                    return Err(PathRoutingError::NegativeCycleDetected);
                }
            }
        }
        
        // Reconstruct path if destination is reachable
        if distances.get(destination).unwrap_or(&f64::INFINITY) != &f64::INFINITY {
            let mut path_edges = Vec::new();
            let mut current_token = destination;
            let mut visited = std::collections::HashSet::new();
            
            // Backtrack from destination to source
            while current_token != source && !visited.contains(current_token) {
                visited.insert(current_token.clone());
                
                if let Some((prev_token, edge_index)) = predecessors.get(current_token) {
                    if let Some(edges) = self.graph.get(*prev_token) {
                        if let Some(edge) = edges.get(*edge_index) {
                            path_edges.push(edge.clone());
                            current_token = prev_token;
                            continue;
                        }
                    }
                }
                break;
            }
            
            // Check if we successfully reached the source
            if current_token == source {
                // Reverse the path to get it from source to destination
                path_edges.reverse();
                
                // Calculate path metrics
                let mut total_exchange_rate = 1.0;
                let mut total_fee = 0.0;
                let mut min_liquidity = u64::MAX;
                
                for edge in &path_edges {
                    total_exchange_rate *= edge.exchange_rate;
                    total_fee += edge.fee;
                    min_liquidity = min_liquidity.min(edge.liquidity);
                }
                
                // Adjust for the input amount
                total_exchange_rate *= amount;
                
                return Ok(Some(RoutingPath {
                    edges: path_edges,
                    total_exchange_rate,
                    total_fee,
                    min_liquidity,
                }));
            }
        }
        
        Ok(None) // No path found
    }

    /// Find all possible paths from source to destination (for exploration)
    pub fn find_all_paths(&self, source: &TokenId, destination: &TokenId, max_hops: usize) -> Vec<RoutingPath> {
        let mut all_paths = Vec::new();
        let mut queue = VecDeque::new();
        
        // Start with direct edges from source
        if let Some(edges) = self.graph.get(source) {
            for edge in edges {
                let path = vec![edge.clone()];
                queue.push_back((edge.to_token.clone(), path, 1));
            }
        }
        
        // BFS to find all paths
        while let Some((current_token, path, hops)) = queue.pop_front() {
            if &current_token == destination {
                // Found a path to destination
                let mut total_exchange_rate = 1.0;
                let mut total_fee = 0.0;
                let mut min_liquidity = u64::MAX;
                
                for edge in &path {
                    total_exchange_rate *= edge.exchange_rate;
                    total_fee += edge.fee;
                    min_liquidity = min_liquidity.min(edge.liquidity);
                }
                
                all_paths.push(RoutingPath {
                    edges: path,
                    total_exchange_rate,
                    total_fee,
                    min_liquidity,
                });
                
                continue;
            }
            
            // Continue searching if we haven't reached max hops
            if hops < max_hops {
                if let Some(edges) = self.graph.get(&current_token) {
                    for edge in edges {
                        // Avoid cycles by checking if we've already visited this token in this path
                        if !path.iter().any(|e| e.from_token == edge.to_token) {
                            let mut new_path = path.clone();
                            new_path.push(edge.clone());
                            queue.push_back((edge.to_token.clone(), new_path, hops + 1));
                        }
                    }
                }
            }
        }
        
        all_paths
    }

    /// Get all tokens in the graph
    pub fn get_tokens(&self) -> &[TokenId] {
        &self.tokens
    }

    /// Get all edges from a specific token
    pub fn get_edges_from_token(&self, token: &TokenId) -> Option<&Vec<TradingEdge>> {
        self.graph.get(token)
    }

    /// Get the number of tokens in the graph
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    /// Get the number of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.graph.values().map(|edges| edges.len()).sum()
    }
}

/// Errors that can occur during path routing operations
#[derive(Debug, Error)]
pub enum PathRoutingError {
    #[error("Negative cycle detected in trading graph")]
    NegativeCycleDetected,
    #[error("Source and destination tokens are the same")]
    SameSourceDestination,
    #[error("No path found between source and destination")]
    NoPathFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_router_creation() {
        let router = PathRouter::new();
        assert_eq!(router.token_count(), 0);
        assert_eq!(router.edge_count(), 0);
        assert!(router.get_tokens().is_empty());
    }

    #[test]
    fn test_add_edge() {
        let mut router = PathRouter::new();
        
        let edge = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };
        
        router.add_edge(edge.clone());
        
        assert_eq!(router.token_count(), 2);
        assert_eq!(router.edge_count(), 1);
        assert!(router.get_tokens().contains(&"BTC".to_string()));
        assert!(router.get_tokens().contains(&"ETH".to_string()));
        
        let edges = router.get_edges_from_token(&"BTC".to_string()).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0], edge);
    }

    #[test]
    fn test_find_best_path_simple() {
        let mut router = PathRouter::new();
        
        // Add a simple path: BTC -> ETH
        let edge = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };
        
        router.add_edge(edge.clone());
        
        // Find path from BTC to ETH
        let result = router.find_best_path(&"BTC".to_string(), &"ETH".to_string(), 1.0).unwrap();
        assert!(result.is_some());
        
        let path = result.unwrap();
        assert_eq!(path.edges.len(), 1);
        assert_eq!(path.edges[0], edge);
        assert_eq!(path.total_exchange_rate, 13.5);
        assert_eq!(path.total_fee, 0.003);
        assert_eq!(path.min_liquidity, 1000000);
    }

    #[test]
    fn test_find_best_path_multi_hop() {
        let mut router = PathRouter::new();
        
        // Add path: BTC -> ETH -> USDC
        let edge1 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };
        
        let edge2 = TradingEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            exchange_rate: 3200.0,
            fee: 0.003,
            liquidity: 50000000,
        };
        
        router.add_edge(edge1.clone());
        router.add_edge(edge2.clone());
        
        // Find path from BTC to USDC
        let result = router.find_best_path(&"BTC".to_string(), &"USDC".to_string(), 1.0).unwrap();
        assert!(result.is_some());
        
        let path = result.unwrap();
        assert_eq!(path.edges.len(), 2);
        assert_eq!(path.edges[0], edge1);
        assert_eq!(path.edges[1], edge2);
        assert_eq!(path.total_exchange_rate, 13.5 * 3200.0); // 43,200
        assert_eq!(path.total_fee, 0.006); // 0.003 + 0.003
        assert_eq!(path.min_liquidity, 1000000); // Limited by BTC->ETH liquidity
    }

    #[test]
    fn test_find_best_path_no_path() {
        let mut router = PathRouter::new();
        
        // Add path: BTC -> ETH
        let edge = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };
        
        router.add_edge(edge.clone());
        
        // Try to find path from BTC to USDC (no path exists)
        let result = router.find_best_path(&"BTC".to_string(), &"USDC".to_string(), 1.0).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_remove_dex_edges() {
        let mut router = PathRouter::new();
        
        // Add edges from different DEXes
        let edge1 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };
        
        let edge2 = TradingEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            exchange_rate: 3200.0,
            fee: 0.003,
            liquidity: 50000000,
        };
        
        router.add_edge(edge1.clone());
        router.add_edge(edge2.clone());
        
        assert_eq!(router.edge_count(), 2);
        
        // Remove Uniswap edges
        router.remove_dex_edges("Uniswap");
        
        assert_eq!(router.edge_count(), 1);
        assert!(router.get_edges_from_token(&"BTC".to_string()).map_or(true, |edges| edges.is_empty()));
        assert_eq!(router.get_edges_from_token(&"ETH".to_string()).unwrap().len(), 1);
    }

    #[test]
    fn test_find_all_paths() {
        let mut router = PathRouter::new();
        
        // Add multiple paths: BTC -> ETH -> USDC and BTC -> USDC
        let edge1 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            exchange_rate: 13.5,
            fee: 0.003,
            liquidity: 1000000,
        };
        
        let edge2 = TradingEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            exchange_rate: 3200.0,
            fee: 0.003,
            liquidity: 50000000,
        };
        
        let edge3 = TradingEdge {
            from_token: "BTC".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "Curve".to_string(),
            exchange_rate: 45000.0,
            fee: 0.004,
            liquidity: 20000000,
        };
        
        router.add_edge(edge1.clone());
        router.add_edge(edge2.clone());
        router.add_edge(edge3.clone());
        
        // Find all paths from BTC to USDC with max 2 hops
        let paths = router.find_all_paths(&"BTC".to_string(), &"USDC".to_string(), 2);
        assert_eq!(paths.len(), 2);
        
        // One path should be direct (BTC -> USDC)
        let direct_path = paths.iter().find(|p| p.edges.len() == 1).unwrap();
        assert_eq!(direct_path.edges[0], edge3);
        
        // One path should be multi-hop (BTC -> ETH -> USDC)
        let multi_hop_path = paths.iter().find(|p| p.edges.len() == 2).unwrap();
        assert_eq!(multi_hop_path.edges[0], edge1);
        assert_eq!(multi_hop_path.edges[1], edge2);
    }
}