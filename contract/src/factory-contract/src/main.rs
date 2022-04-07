#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::string::String;

use swappery_pair::SwapperyPair;

use casper_erc20::{
    constants::{
        ADDRESS_RUNTIME_ARG_NAME, AMOUNT_RUNTIME_ARG_NAME, DECIMALS_RUNTIME_ARG_NAME,
        NAME_RUNTIME_ARG_NAME, OWNER_RUNTIME_ARG_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        SPENDER_RUNTIME_ARG_NAME, SYMBOL_RUNTIME_ARG_NAME, TOTAL_SUPPLY_RUNTIME_ARG_NAME,
    },
    Address,
};

use casper_types::{ HashAddr, ContractHash, Key, account::AccountHash };

use casper_contract::{ 
    contract_api::{ runtime },
    unwrap_or_revert::UnwrapOrRevert
};

const CONTRACT_KEY_NAME_ARG_NAME: &str = "contract_key_name";

#[no_mangle]
fn call() {
    let name: String = runtime::get_named_arg(NAME_RUNTIME_ARG_NAME);
    let symbol: String = runtime::get_named_arg(SYMBOL_RUNTIME_ARG_NAME);
    let decimals = runtime::get_named_arg(DECIMALS_RUNTIME_ARG_NAME);
    let initial_supply = runtime::get_named_arg(TOTAL_SUPPLY_RUNTIME_ARG_NAME);
    let contract_key_name: String = runtime::get_named_arg(CONTRACT_KEY_NAME_ARG_NAME);

    let _ = SwapperyPair::create(
        name,
        symbol,
        decimals,
        initial_supply,
        contract_key_name.as_str(),
        Address::from(AccountHash::new([0u8; 32])),
        Address::from(AccountHash::new([0u8; 32]))
    );

    let key: Key = runtime::get_key(contract_key_name.as_str()).unwrap_or_revert();
    let hash: HashAddr = key.into_hash().unwrap_or_revert();
    let contract_hash = ContractHash::new(hash);
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}