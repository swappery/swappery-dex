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
mod entry_points;
mod error;

use alloc::{string::String, vec::Vec};

use casper_erc20::{
    constants::{
        AMOUNT_RUNTIME_ARG_NAME, OWNER_RUNTIME_ARG_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        TRANSFER_FROM_ENTRY_POINT_NAME, 
    },
    Address,
};

use constants::{
    GET_RESERVES_ENTRY_POINT_NAME, TOKEN0_RUNTIME_ARG_NAME, TOKEN1_RUNTIME_ARG_NAME,
    AMOUNT0_DESIRED_RUNTIME_ARG_NAME, AMOUNT1_DESIRED_RUNTIME_ARG_NAME,
    AMOUNT0_MIN_RUNTIME_ARG_NAME, AMOUNT1_MIN_RUNTIME_ARG_NAME,
    TO_RUNTIME_ARG_NAME, DEAD_LINE_RUNTIME_ARG_NAME,  MINT_ENTRY_POINT_NAME, 
    LIQUIDITY_RUNTIME_ARG_NAME, BURN_ENTRY_POINT_NAME, AMOUNT0_RUNTIME_ARG_NAME,
    AMOUNT1_RUNTIME_ARG_NAME, SWAP_ENTRY_POINT_NAME, AMOUNT_IN_RUNTIME_ARG_NAME,
    AMOUNT_OUT_RUNTIME_ARG_NAME, AMOUNT_IN_MAX_RUNTIME_ARG_NAME, AMOUNT_OUT_MIN_RUNTIME_ARG_NAME,
    PATH_RUNTIME_ARG_NAME, WCSPR_CONTRACT_KEY_NAME, PAIR_LIST_KEY_NAME, FEETO_KEY_NAME,
    FEETO_SETTER_KEY_NAME, 
};

use casper_types::{
    ContractHash, Key, URef, U256, runtime_args, RuntimeArgs, contracts::NamedKeys,
    Error,
};

use casper_contract::{contract_api::{runtime, storage}, unwrap_or_revert::UnwrapOrRevert};

use once_cell::unsync::OnceCell;

const CONTRACT_KEY_NAME_ARG_NAME: &str = "contract_key_name";

#[derive(Default)]
pub struct SwapperyRouter {
    pair_list_uref: OnceCell<URef>,
    feeto_uref: OnceCell<URef>,
    feeto_setter_uref: OnceCell<URef>,
}

impl SwapperyRouter {
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

    pub fn wcspr_token(&self) -> ContractHash {
        helpers::read_from(WCSPR_CONTRACT_KEY_NAME)
    }

    pub fn make_pair_path_from_token_path(&self, token_path: Vec<ContractHash>) -> Vec<Address> {
        let mut pair_path: Vec<Address> = Vec::with_capacity(token_path.len() - 1);
        for i in 0..token_path.len() - 2 {
            pair_path.push(self.get_pair_for(*token_path.get(i).unwrap_or_revert(), *token_path.get(i + 1).unwrap_or_revert()));
        }
        pair_path
    }

    pub fn create(
        feeto: Address,
        feeto_setter: Address,
        wcspr_token: ContractHash,
        contract_key_name: String,
    ) -> Result<SwapperyRouter, Error> {
        let pair_list_uref: URef = storage::new_dictionary(PAIR_LIST_KEY_NAME).unwrap_or_revert();
        let feeto_uref: URef = storage::new_uref(feeto).into_read_write();
        let feeto_setter_uref: URef = storage::new_uref(feeto_setter).into_read_write();
        let wcspr_token_key: Key = {
            let wcspr_token_uref = storage::new_uref(wcspr_token).into_read();
            Key::from(wcspr_token_uref)
        };
        let pair_list_key = {
            runtime::remove_key(PAIR_LIST_KEY_NAME);
            Key::from(pair_list_uref)
        };
        let feeto_key = Key::from(feeto_uref);
        let feeto_setter_key = Key::from(feeto_setter_uref);

        let mut named_keys = NamedKeys::new();
        named_keys.insert(String::from(PAIR_LIST_KEY_NAME), pair_list_key);
        named_keys.insert(String::from(FEETO_KEY_NAME), feeto_key);
        named_keys.insert(String::from(FEETO_SETTER_KEY_NAME), feeto_setter_key);
        named_keys.insert(String::from(WCSPR_CONTRACT_KEY_NAME), wcspr_token_key);

        let (contract_hash, _version) = storage::new_contract(
            entry_points::default(),
            Some(named_keys),
            Some(contract_key_name),
            None,
        );
        Ok(SwapperyRouter::new(
            pair_list_uref,
            feeto_uref,
            feeto_setter_uref,
        ))
    }
    
    pub fn _add_liquidity(
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
                    runtime::revert(error::Error::InsufficientBAmount);
                }
                amounts = (amount0_desired, amount1_optimal);
            } else {
                let amount0_optimal: U256 = helpers::quote(amount1_desired, reserves.1, reserves.0);
                if !(amount0_optimal >= amount0_min) {
                    runtime::revert(error::Error::InsufficientAAmount);
                }
                amounts = (amount0_optimal, amount1_desired);
            }
        }
        amounts
    }

    pub fn _swap(
        &self,
        amounts: Vec<U256>,
        path: Vec<ContractHash>,
        _to: Address
    ) {
        for i in 0..path.len() - 1 {
            let (input, output): (&ContractHash, &ContractHash) = (path.get(i).unwrap_or_revert(), path.get(i + 1).unwrap_or_revert());
            let (token0, ..) = helpers::sort_tokens(*input, *output);
            let amount_out: &U256 = amounts.get(i + 1).unwrap_or_revert();
            let mut amounts_out: (U256, U256);
            if input.eq(&token0) {
                amounts_out = (U256::zero(), *amount_out);
            } else {
                amounts_out = (*amount_out, U256::zero());
            }
            let to: Address;
            if i < path.len() - 2 {
                to = self.get_pair_for(*output, *path.get(i + 2).unwrap_or_revert());
            } else {
                to = _to;
            }
            
            let pair: Address = self.get_pair_for(*input, *output);
            runtime::call_versioned_contract::<()>(
                *pair.as_contract_package_hash().unwrap_or_revert(),
                None,
                SWAP_ENTRY_POINT_NAME,
                runtime_args! {
                    AMOUNT0_RUNTIME_ARG_NAME => amounts_out.0,
                    AMOUNT1_RUNTIME_ARG_NAME => amounts_out.1,
                    TO_RUNTIME_ARG_NAME => to
                }
            );
        }
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
    
    let tokens: (ContractHash, ContractHash) = helpers::sort_tokens(token0, token1);

    let amounts: (U256, U256) = SwapperyRouter::default()._add_liquidity(tokens.0, tokens.1, amount0_desired, amount1_desired, amount0_min, amount1_min);    
    let pair: Address = SwapperyRouter::default().get_pair_for(tokens.0, tokens.1);
    let caller: Address = helpers::get_caller_address().unwrap_or_revert();
    runtime::call_contract::<()>(
        tokens.0,
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => caller,
            RECIPIENT_RUNTIME_ARG_NAME => pair,
            AMOUNT_RUNTIME_ARG_NAME => amounts.0
        },
    );
    runtime::call_contract::<()>(
        tokens.1,
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => caller,
            RECIPIENT_RUNTIME_ARG_NAME => pair,
            AMOUNT_RUNTIME_ARG_NAME => amounts.1
        },
    );
    runtime::call_versioned_contract::<()>(
        *pair.as_contract_package_hash().unwrap_or_revert(),
        None,
        MINT_ENTRY_POINT_NAME,
        runtime_args! {
            TO_RUNTIME_ARG_NAME => to
        },
    );
}

#[no_mangle]
pub extern "C" fn remove_liquidity() {
    let token0: ContractHash = runtime::get_named_arg(TOKEN0_RUNTIME_ARG_NAME);
    let token1: ContractHash = runtime::get_named_arg(TOKEN1_RUNTIME_ARG_NAME);
    let liquidity: U256 = runtime::get_named_arg(LIQUIDITY_RUNTIME_ARG_NAME);
    let amount0_min: U256 = runtime::get_named_arg(AMOUNT0_MIN_RUNTIME_ARG_NAME);
    let amount1_min: U256 = runtime::get_named_arg(AMOUNT1_MIN_RUNTIME_ARG_NAME);
    let to: Address = runtime::get_named_arg(TO_RUNTIME_ARG_NAME);
    let dead_line: U256 = runtime::get_named_arg(DEAD_LINE_RUNTIME_ARG_NAME);

    let tokens: (ContractHash, ContractHash) = helpers::sort_tokens(token0, token1);

    let pair: Address = SwapperyRouter::default().get_pair_for(tokens.0, tokens.1);
    let caller: Address = helpers::get_caller_address().unwrap_or_revert();

    runtime::call_versioned_contract::<()>(
        *pair.as_contract_package_hash().unwrap_or_revert(),
        None,
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => caller,
            RECIPIENT_RUNTIME_ARG_NAME => pair,
            AMOUNT_RUNTIME_ARG_NAME => liquidity
        },
    );
    let amounts: (U256, U256) = runtime::call_versioned_contract(
        *pair.as_contract_package_hash().unwrap_or_revert(),
        None,
        BURN_ENTRY_POINT_NAME,
        runtime_args! {
            TO_RUNTIME_ARG_NAME => to
        },
    );
    if amounts.0 < amount0_min {
        runtime::revert(error::Error::InsufficientAAmount);
    }
    if amounts.1 < amount1_min {
        runtime::revert(error::Error::InsufficientBAmount);
    }
}

#[no_mangle]
pub extern "C" fn swap_exact_tokens_for_tokens() {
    let amount_in: U256 = runtime::get_named_arg(AMOUNT_IN_RUNTIME_ARG_NAME);
    let amount_out_min: U256 = runtime::get_named_arg(AMOUNT_OUT_MIN_RUNTIME_ARG_NAME);
    let path: Vec<ContractHash> = runtime::get_named_arg(PATH_RUNTIME_ARG_NAME);
    let to: Address = runtime::get_named_arg(TO_RUNTIME_ARG_NAME);
    let dead_line: U256 = runtime::get_named_arg(DEAD_LINE_RUNTIME_ARG_NAME);

    let amounts: Vec<U256> = helpers::get_amounts_out(amount_in, SwapperyRouter::default().make_pair_path_from_token_path(path.clone()));
    if !(amounts.last().unwrap_or_revert() >= &amount_out_min) {
        runtime::revert(error::Error::InsufficientOutputAmount);
    }

    let caller: Address = helpers::get_caller_address().unwrap_or_revert();
    runtime::call_contract::<()>(
        *path.get(0).unwrap_or_revert(),
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => caller,
            RECIPIENT_RUNTIME_ARG_NAME => SwapperyRouter::default().get_pair_for(
                *path.get(0).unwrap_or_revert(),
                *path.get(1).unwrap_or_revert(),
            ),
            AMOUNT_RUNTIME_ARG_NAME => *amounts.get(0).unwrap_or_revert(),
        },
    );
    SwapperyRouter::default()._swap(amounts, path, to);
}

#[no_mangle]
pub extern "C" fn swap_tokens_for_exact_tokens() {
    let amount_out: U256 = runtime::get_named_arg(AMOUNT_OUT_RUNTIME_ARG_NAME);
    let amount_in_max: U256 = runtime::get_named_arg(AMOUNT_IN_MAX_RUNTIME_ARG_NAME);
    let path: Vec<ContractHash> = runtime::get_named_arg(PATH_RUNTIME_ARG_NAME);
    let to: Address = runtime::get_named_arg(TO_RUNTIME_ARG_NAME);
    let dead_line: U256 = runtime::get_named_arg(DEAD_LINE_RUNTIME_ARG_NAME);

    let amounts: Vec<U256> = helpers::get_amounts_in(amount_out, SwapperyRouter::default().make_pair_path_from_token_path(path.clone()));
    
    if !(amounts.get(0).unwrap_or_revert() >= &amount_in_max) {
        runtime::revert(error::Error::InsufficientInputAmount);
    }

    let caller: Address = helpers::get_caller_address().unwrap_or_revert();
    runtime::call_contract::<()>(
        *path.get(0).unwrap_or_revert(),
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => caller,
            RECIPIENT_RUNTIME_ARG_NAME => SwapperyRouter::default().get_pair_for(
                *path.get(0).unwrap_or_revert(),
                *path.get(1).unwrap_or_revert(),
            ),
            AMOUNT_RUNTIME_ARG_NAME => *amounts.get(0).unwrap_or_revert(),
        },
    );
    SwapperyRouter::default()._swap(amounts, path, to);
}

#[no_mangle]
fn call() {
    let feeto: Address = runtime::get_named_arg(FEETO_KEY_NAME);
    let feeto_setter: Address = runtime::get_named_arg(FEETO_SETTER_KEY_NAME);
    let wcspr_token: ContractHash = runtime::get_named_arg(WCSPR_CONTRACT_KEY_NAME);
    let contract_key_name: String = runtime::get_named_arg(CONTRACT_KEY_NAME_ARG_NAME);

    let _ = SwapperyRouter::create(
        feeto,
        feeto_setter,
        wcspr_token,
        contract_key_name,
    );
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}