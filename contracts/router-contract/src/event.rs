use alloc::vec::Vec;
use casper_types::{Key, U256};

pub enum RouterEvent {
    CreatePair {
        token0: Key,
        token1: Key,
        pair: Key,
    },
    AddLiquidity {
        token0: Key,
        token1: Key,
        amount0: U256,
        amount1: U256,
        recipient: Key,
    },
    RemoveLiquidity {
        token0: Key,
        token1: Key,
        liquidity: U256,
        recipient: Key,
    },
    SwapExactIn {
        amount_in: U256,
        amount_out: U256,
        path: Vec<Key>,
        recipient: Key,
    },
    SwapExactOut {
        amount_in: U256,
        amount_out: U256,
        path: Vec<Key>,
        recipient: Key,
    },
}
