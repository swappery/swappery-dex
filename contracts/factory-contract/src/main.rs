#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
extern crate swappery_pair;

mod constants;
mod feeto;
mod helpers;
mod pair_list;

use alloc::string::String;

// use swappery_pair::SwapperyPair;

use swappery_pair::{
    constants::{
        ADDRESS_RUNTIME_ARG_NAME, AMOUNT_RUNTIME_ARG_NAME, DECIMALS_RUNTIME_ARG_NAME,
        NAME_RUNTIME_ARG_NAME, OWNER_RUNTIME_ARG_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        SPENDER_RUNTIME_ARG_NAME, SYMBOL_RUNTIME_ARG_NAME, TOTAL_SUPPLY_RUNTIME_ARG_NAME,
    },
    Address,
};

use casper_types::{account::AccountHash, ContractHash, HashAddr, Key, URef, U256};

use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};

use once_cell::unsync::OnceCell;

const CONTRACT_KEY_NAME_ARG_NAME: &str = "contract_key_name";

#[derive(Default)]
pub struct SwapperyFactory {
    pair_list_uref: OnceCell<URef>,
    feeto_uref: OnceCell<URef>,
    feeto_setter_uref: OnceCell<URef>,
}

impl SwapperyFactory {
    fn new(pair_list_uref: URef, feeto_uref: URef, feeto_setter_uref: URef) -> Self {
        Self {
            pair_list_uref: pair_list_uref.into(),
            feeto_uref: feeto_uref.into(),
            feeto_setter_uref: feeto_setter_uref.into(),
        }
    }
    fn pair_list_uref(&self) -> URef {
        *self
            .pair_list_uref
            .get_or_init(pair_list::get_pair_list_uref)
    }
    fn get_pair_for(&self, token0: Address, token1: Address) -> Address {
        pair_list::get_pair_for(self.pair_list_uref(), token0, token1)
    }
    fn add_pair_for(&self, token0: Address, token1: Address, pair: Address) {
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
