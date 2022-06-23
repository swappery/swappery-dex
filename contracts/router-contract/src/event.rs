use alloc::{string::String, vec::Vec};
use casper_types::{ContractHash, U256};

pub enum RouterEvent {
    CreatePair {
        token0: String,
        token1: String,
        pair: String,
    },
    AddLiquidity {
        token0: String,
        token1: String,
        amount0: U256,
        amount1: U256,
        recipient: String,
    },
    RemoveLiquidity {
        token0: String,
        token1: String,
        liquidity: U256,
        recipient: String,
    },
    SwapExactIn {
        amount_in: U256,
        amount_out: U256,
        path: Vec<ContractHash>,
        recipient: String,
    },
    SwapExactOut {
        amount_in: U256,
        amount_out: U256,
        path: Vec<ContractHash>,
        recipient: String,
    },
    Installed {
        contract_hash: ContractHash,
    },
}
