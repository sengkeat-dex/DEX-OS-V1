//! DEX-OS core engine library

pub mod orderbook;
pub mod amm;
pub mod types;
pub mod merkle_tree;
pub mod stableswap;
pub mod fee_management;
pub mod fee_distribution;
pub mod path_routing;
pub mod partial_fill;
pub mod trade_prevention;
pub mod price_prediction;
pub mod reward_distribution;
pub mod multisig_wallet;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}