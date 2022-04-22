#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

mod constants;
mod feeto;
mod helpers;
mod pair_list;
mod wcspr;

use alloc::string::String;

use casper_erc20::{
    constants::{
        ADDRESS_RUNTIME_ARG_NAME, AMOUNT_RUNTIME_ARG_NAME, DECIMALS_RUNTIME_ARG_NAME,
        NAME_RUNTIME_ARG_NAME, OWNER_RUNTIME_ARG_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        SPENDER_RUNTIME_ARG_NAME, SYMBOL_RUNTIME_ARG_NAME, TOTAL_SUPPLY_RUNTIME_ARG_NAME,
    },
    Address,
};

use constants::{
    GET_RESERVES_ENTRY_POINT_NAME, TOKEN0_RUNTIME_ARG_NAME, TOKEN1_RUNTIME_ARG_NAME,
    AMOUNT0_DESIRED_RUNTIME_ARG_NAME, AMOUNT1_DESIRED_RUNTIME_ARG_NAME,
    AMOUNT0_MIN_RUNTIME_ARG_NAME, AMOUNT1_MIN_RUNTIME_ARG_NAME,
    TO_RUNTIME_ARG_NAME, DEAD_LINE_RUNTIME_ARG_NAME,
};

use casper_types::{account::AccountHash, ContractHash, HashAddr, Key, URef, U256, runtime_args};

use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};

use once_cell::unsync::OnceCell;

const CONTRACT_KEY_NAME_ARG_NAME: &str = "contract_key_name";

#[derive(Default)]
pub struct SwapperyRouter {
    pair_list_uref: OnceCell<URef>,
    feeto_uref: OnceCell<URef>,
    feeto_setter_uref: OnceCell<URef>,
    wcspr_uref: OnceCell<URef>,
}

impl SwapperyRouter {
    fn new(pair_list_uref: URef, feeto_uref: URef, feeto_setter_uref: URef, wcspr_uref: URef) -> Self {
        Self {
            pair_list_uref: pair_list_uref.into(),
            feeto_uref: feeto_uref.into(),
            feeto_setter_uref: feeto_setter_uref.into(),
            wcspr_uref: wcspr_uref.into(),
        }
    }
    fn pair_list_uref(&self) -> URef {
        *self
            .pair_list_uref
            .get_or_init(pair_list::get_pair_list_uref)
    }
    fn get_pair_for(&self, token0: ContractHash, token1: ContractHash) -> Address {
        pair_list::get_pair_for(self.pair_list_uref(), token0, token1)
    }
    fn add_pair_for(&self, token0: ContractHash, token1: ContractHash, pair: Address) {
        pair_list::add_pair_for(self.pair_list_uref(), token0, token1, pair)
    }

    fn feeto_uref(&self) -> URef {
        *self.feeto_uref.get_or_init(feeto::feeto_uref)
    }

    fn read_feeto(&self) -> Address {
        feeto::read_feeto_from(self.feeto_uref())
    }

    fn write_feeto(&self, feeto: Address) {
        feeto::write_feeto_to(self.feeto_uref(), feeto)
    }

    fn feeto_setter_uref(&self) -> URef {
        *self.feeto_setter_uref.get_or_init(feeto::feeto_setter_uref)
    }

    fn read_feeto_setter(&self) -> Address {
        feeto::read_feeto_setter_from(self.feeto_setter_uref())
    }

    fn write_feeto_setter(&self, feeto_setter: Address) {
        feeto::write_feeto_setter_to(self.feeto_setter_uref(), feeto_setter)
    }

    fn wcspr_uref(&self) -> URef {
        *self.wcspr_uref.get_or_init(wcspr::wcspr_uref)
    }

    fn read_wcspr(&self) -> ContractHash {
        wcspr::read_wcspr_from(self.wcspr_uref())
    }

    fn write_wcspr(&self, wcspr: ContractHash) {
        wcspr::write_wcspr_to(self.wcspr_uref(), wcspr);
    }
    
    fn _add_liquidity(
        &self,
        token0: ContractHash,
        token1: ContractHash,
        amount0_desired: U256,
        amount1_desired: U256,
        amount0_min: U256,
        amount1_min: U256
    ) -> (U256, U256) {
        let mut amounts: (U256, U256);
        let pair: Address = self.get_pair_for(token0, token1);
        let reserves: (U256, U256) = runtime::call_versioned_contract(
            *pair.as_contract_package_hash().unwrap_or_revert(),
            None,
            GET_RESERVES_ENTRY_POINT_NAME,
            runtime_args! {},
        );
        if !(reserves.0 == U256::zero() && reserves.1 == U256::zero()) {
            amounts = (amount0_desired, amount1_desired);
        } else {
            let amount1_optimal: U256 = helpers::quote(amount0_desired, reserves.0, reserves.1);
            if amount1_optimal <= amount1_desired {
                if !(amount1_optimal >= amount1_min) {
                    // require(amountBOptimal >= amountBMin, 'PancakeRouter: INSUFFICIENT_B_AMOUNT);
                }
                amounts = (amount0_desired, amount1_optimal);
            } else {
                let amount0_optimal: U256 = helpers::quote(amount1_desired, reserves.1, reserves.0);
                if !(amount0_optimal >= amount0_min) {
                    // require(amountAOptimal >= amountAMin, 'PancakeRouter: INSUFFICIENT_A_AMOUNT');
                }
                amounts = (amount0_optimal, amount1_desired);
            }
        }
        amounts
    }
}

#[no_mangle]
pub extern "C" fn add_liquidity() {
    let token0: ContractHash = runtime::get_named_arg(TOKEN0_RUNTIME_ARG_NAME);
    let token1: ContractHash = runtime::get_named_arg(TOKEN1_RUNTIME_ARG_NAME);
    let amount0_desired: U256 = runtime::get_named_arg(AMOUNT0_DESIRED_RUNTIME_ARG_NAME);
    let amount1_desired: U256 = runtime::get_named_arg(AMOUNT1_DESIRED_RUNTIME_ARG_NAME);
    let amount0_min: U256 = runtime::get_named_arg(AMOUNT0_MIN_RUNTIME_ARG_NAME);
    let amount1_min: U256 = runtime::get_named_arg(AMOUNT1_MIN_RUNTIME_ARG_NAME);
    let to: Address = runtime::get_named_arg(TO_RUNTIME_ARG_NAME);
    let dead_line: U256 = runtime::get_named_arg(DEAD_LINE_RUNTIME_ARG_NAME);
}

#[no_mangle]
fn call() {
    let name: String = runtime::get_named_arg(NAME_RUNTIME_ARG_NAME);
    let symbol: String = runtime::get_named_arg(SYMBOL_RUNTIME_ARG_NAME);
    let decimals: u8 = runtime::get_named_arg(DECIMALS_RUNTIME_ARG_NAME);
    let initial_supply: U256 = runtime::get_named_arg(TOTAL_SUPPLY_RUNTIME_ARG_NAME);
    let contract_key_name: String = runtime::get_named_arg(CONTRACT_KEY_NAME_ARG_NAME);

    // let _ = SwapperyPair::create(
    //     name,
    //     symbol,
    //     decimals,
    //     initial_supply,
    //     contract_key_name.as_str(),
    //     Address::from(AccountHash::new([0u8; 32])),
    //     Address::from(AccountHash::new([0u8; 32]))
    // );

    let key: Key = runtime::get_key(contract_key_name.as_str()).unwrap_or_revert();
    let hash: HashAddr = key.into_hash().unwrap_or_revert();
    let contract_hash = ContractHash::new(hash);
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
