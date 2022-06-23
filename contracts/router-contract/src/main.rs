#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

mod constants;
mod entry_points;
mod error;
pub mod event;
mod feeto;
mod helpers;
mod pair_list;

use alloc::{string::String, vec::Vec};

use casper_erc20::{
    constants::{
        AMOUNT_RUNTIME_ARG_NAME, OWNER_RUNTIME_ARG_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        TRANSFER_FROM_ENTRY_POINT_NAME,
    },
    Address,
};

use constants as consts;

use casper_types::{
    account::AccountHash, contracts::NamedKeys, runtime_args, CLValue, ContractHash,
    ContractPackageHash, Error, HashAddr, Key, RuntimeArgs, URef, U256,
};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use once_cell::unsync::OnceCell;

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
        helpers::read_from(consts::WCSPR_CONTRACT_KEY_NAME)
    }

    pub fn create(
        feeto: Address,
        feeto_setter: Address,
        wcspr_token: ContractHash,
        contract_key_name: String,
    ) -> Result<SwapperyRouter, Error> {
        let pair_list_uref: URef =
            storage::new_dictionary(consts::PAIR_LIST_KEY_NAME).unwrap_or_revert();
        let feeto_uref: URef = storage::new_uref(feeto).into_read_write();
        let feeto_setter_uref: URef = storage::new_uref(feeto_setter).into_read_write();
        let wcspr_token_key: Key = {
            let wcspr_token_uref = storage::new_uref(wcspr_token).into_read();
            Key::from(wcspr_token_uref)
        };
        let pair_list_key = {
            runtime::remove_key(consts::PAIR_LIST_KEY_NAME);
            Key::from(pair_list_uref)
        };
        let feeto_key = Key::from(feeto_uref);
        let feeto_setter_key = Key::from(feeto_setter_uref);

        let mut named_keys = NamedKeys::new();
        named_keys.insert(String::from(consts::PAIR_LIST_KEY_NAME), pair_list_key);
        named_keys.insert(String::from(consts::FEETO_KEY_NAME), feeto_key);
        named_keys.insert(
            String::from(consts::FEETO_SETTER_KEY_NAME),
            feeto_setter_key,
        );
        named_keys.insert(
            String::from(consts::WCSPR_CONTRACT_KEY_NAME),
            wcspr_token_key,
        );

        let (contract_hash, version) = storage::new_contract(
            entry_points::default(),
            Some(named_keys),
            Some(contract_key_name),
            None,
        );
        let event = event::RouterEvent::Installed { contract_hash };
        helpers::emit(&event);
        Ok(SwapperyRouter::new(
            pair_list_uref,
            feeto_uref,
            feeto_setter_uref,
        ))
    }

    pub fn get_amounts_out(&self, amount_in: U256, path: Vec<ContractHash>) -> Vec<U256> {
        if !(path.len() >= 2) {
            runtime::revert(error::Error::InvalidPath);
        }

        let mut amounts: Vec<U256> = Vec::with_capacity(path.len());
        amounts.push(amount_in);
        for i in 0..path.len() - 1 {
            let pair: Address = self.get_pair_for(*path.get(i).unwrap(), *path.get(i + 1).unwrap());
            let reserves: (U256, U256) =
                helpers::get_reserves(*path.get(i).unwrap(), *path.get(i + 1).unwrap(), pair);
            amounts.push(helpers::get_amount_out(
                *amounts.get(i).unwrap_or_revert(),
                reserves.0,
                reserves.1,
            ));
        }
        amounts
    }

    pub fn get_amounts_in(&self, amount_out: U256, path: Vec<ContractHash>) -> Vec<U256> {
        if !(path.len() >= 1) {
            runtime::revert(error::Error::InvalidPath);
        }

        let mut amounts: Vec<U256> = Vec::with_capacity(path.len());
        amounts.push(amount_out);
        for i in 1..path.len() {
            let pair: Address = self.get_pair_for(
                *path.get(path.len() - i - 1).unwrap(),
                *path.get(path.len() - i).unwrap(),
            );
            let reserves: (U256, U256) = helpers::get_reserves(
                *path.get(path.len() - i - 1).unwrap(),
                *path.get(path.len() - i).unwrap(),
                pair,
            );
            amounts.push(helpers::get_amount_in(
                *amounts.get(i - 1).unwrap_or_revert(),
                reserves.0,
                reserves.1,
            ));
        }
        amounts.reverse();
        amounts
    }

    pub fn _add_liquidity(
        &self,
        token0: ContractHash,
        token1: ContractHash,
        amount0_desired: U256,
        amount1_desired: U256,
        amount0_min: U256,
        amount1_min: U256,
    ) -> (U256, U256) {
        let amounts: (U256, U256);
        let pair: Address = self.get_pair_for(token0, token1);
        let reserves = helpers::get_reserves(token0, token1, pair);

        if reserves.0 == U256::zero() && reserves.1 == U256::zero() {
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

    pub fn _swap(&self, amounts: Vec<U256>, path: Vec<ContractHash>, _to: Address) {
        for i in 0..path.len() - 1 {
            let (input, output): (&ContractHash, &ContractHash) = (
                path.get(i).unwrap_or_revert(),
                path.get(i + 1).unwrap_or_revert(),
            );
            let (token0, ..) = helpers::sort_tokens(*input, *output);
            let amount_out: &U256 = amounts.get(i + 1).unwrap_or_revert();
            let amounts_out: (U256, U256);
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
                consts::SWAP_ENTRY_POINT_NAME,
                runtime_args! {
                    consts::AMOUNT0_RUNTIME_ARG_NAME => amounts_out.0,
                    consts::AMOUNT1_RUNTIME_ARG_NAME => amounts_out.1,
                    consts::TO_RUNTIME_ARG_NAME => to
                },
            );
        }
        if path.last().unwrap_or_revert().eq(&self.wcspr_token()) {
            runtime::call_contract::<()>(
                self.wcspr_token(),
                consts::WITHDRAW_ENTRY_POINT_NAME,
                runtime_args! {
                    consts::AMOUNT_RUNTIME_ARG_NAME => *amounts.get(amounts.len() - 1).unwrap_or_revert(),
                    consts::TO_RUNTIME_ARG_NAME => _to
                },
            );
        }
    }

    pub fn _swap_supporting_fee(&self, path: Vec<ContractHash>, _to: Address) {
        for i in 0..path.len() - 1 {
            let (input, output) = (
                path.get(i).unwrap_or_revert(),
                path.get(i + 1).unwrap_or_revert(),
            );
            let (token0, ..) = helpers::sort_tokens(*input, *output);
            let pair = self.get_pair_for(*input, *output);
            let reserves = helpers::get_reserves(*input, *output, pair);

            let mut amount_in = runtime::call_contract(
                *input,
                consts::BALANCE_OF_ENTRY_POINT_NAME,
                runtime_args! {
                    consts::ADDRESS_RUNTIME_ARG_NAME => pair
                },
            );
            amount_in = amount_in - reserves.0;
            let amount_out = helpers::get_amount_out(amount_in, reserves.0, reserves.1);

            let amounts_out: (U256, U256);
            if input.eq(&token0) {
                amounts_out = (U256::zero(), amount_out);
            } else {
                amounts_out = (amount_out, U256::zero());
            }

            let to: Address;
            if i < path.len() - 2 {
                to = self.get_pair_for(*output, *path.get(i + 2).unwrap_or_revert());
            } else {
                to = _to;
            }
            runtime::call_versioned_contract::<()>(
                *pair.as_contract_package_hash().unwrap_or_revert(),
                None,
                consts::SWAP_ENTRY_POINT_NAME,
                runtime_args! {
                    consts::AMOUNT0_RUNTIME_ARG_NAME => amounts_out.0,
                    consts::AMOUNT1_RUNTIME_ARG_NAME => amounts_out.1,
                    consts::TO_RUNTIME_ARG_NAME => to
                },
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn create_pair() {
    let token0_key: Key = runtime::get_named_arg(consts::TOKEN0_RUNTIME_ARG_NAME);
    let token1_key: Key = runtime::get_named_arg(consts::TOKEN1_RUNTIME_ARG_NAME);
    let _token0_hash: HashAddr = token0_key.into_hash().unwrap_or_revert();
    let token0: ContractHash = ContractHash::new(_token0_hash);
    let _token1_hash: HashAddr = token1_key.into_hash().unwrap_or_revert();
    let token1: ContractHash = ContractHash::new(_token1_hash);
    let pair_key: Key = runtime::get_named_arg(consts::PAIR_RUNTIME_ARG_NAME);
    let _pair_hash: HashAddr = pair_key.into_hash().unwrap_or_revert();
    let pair: Address = Address::from(ContractPackageHash::from(_pair_hash));

    SwapperyRouter::default().add_pair_for(token0, token1, pair);
    let event = event::RouterEvent::CreatePair {
        token0: token0.to_formatted_string(),
        token1: token1.to_formatted_string(),
        pair: pair
            .as_contract_package_hash()
            .unwrap()
            .to_formatted_string(),
    };
    helpers::emit(&event);
}

#[no_mangle]
pub extern "C" fn get_pair() {
    let token0_key: Key = runtime::get_named_arg(consts::TOKEN0_RUNTIME_ARG_NAME);
    let token1_key: Key = runtime::get_named_arg(consts::TOKEN1_RUNTIME_ARG_NAME);
    let _token0_hash: HashAddr = token0_key.into_hash().unwrap_or_revert();
    let token0: ContractHash = ContractHash::new(_token0_hash);
    let _token1_hash: HashAddr = token1_key.into_hash().unwrap_or_revert();
    let token1: ContractHash = ContractHash::new(_token1_hash);

    let pair: Address = SwapperyRouter::default().get_pair_for(token0, token1);
    runtime::ret(CLValue::from_t(pair).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn set_feeto() {
    let feeto_key: Key = runtime::get_named_arg(consts::FEETO_KEY_NAME);
    let feeto = Address::from(AccountHash::new(feeto_key.into_hash().unwrap_or_revert()));
    let caller: Address = helpers::get_immediate_caller_address().unwrap_or_revert();
    let feeto_setter = SwapperyRouter::default().read_feeto_setter();
    if caller != feeto_setter {
        runtime::revert(error::Error::Permission);
    }
    SwapperyRouter::default().write_feeto(feeto);
}

#[no_mangle]
pub extern "C" fn set_feeto_setter() {
    let feeto_key: Key = runtime::get_named_arg(consts::FEETO_SETTER_KEY_NAME);
    let feeto = Address::from(AccountHash::new(feeto_key.into_hash().unwrap_or_revert()));
    let caller: Address = helpers::get_immediate_caller_address().unwrap_or_revert();
    let feeto_setter = SwapperyRouter::default().read_feeto_setter();
    if caller != feeto_setter {
        runtime::revert(error::Error::Permission);
    }
    SwapperyRouter::default().write_feeto_setter(feeto);
}

#[no_mangle]
pub extern "C" fn add_liquidity() {
    let token0_key: Key = runtime::get_named_arg(consts::TOKEN0_RUNTIME_ARG_NAME);
    let token1_key: Key = runtime::get_named_arg(consts::TOKEN1_RUNTIME_ARG_NAME);
    let _token0_hash: HashAddr = token0_key.into_hash().unwrap_or_revert();
    let token0: ContractHash = ContractHash::new(_token0_hash);
    let _token1_hash: HashAddr = token1_key.into_hash().unwrap_or_revert();
    let token1: ContractHash = ContractHash::new(_token1_hash);
    let amount0_desired: U256 = runtime::get_named_arg(consts::AMOUNT0_DESIRED_RUNTIME_ARG_NAME);
    let amount1_desired: U256 = runtime::get_named_arg(consts::AMOUNT1_DESIRED_RUNTIME_ARG_NAME);
    let amount0_min: U256 = runtime::get_named_arg(consts::AMOUNT0_MIN_RUNTIME_ARG_NAME);
    let amount1_min: U256 = runtime::get_named_arg(consts::AMOUNT1_MIN_RUNTIME_ARG_NAME);
    let to_key: Key = runtime::get_named_arg(consts::TO_RUNTIME_ARG_NAME);
    let to = Address::from(AccountHash::new(to_key.into_hash().unwrap_or_revert()));
    // let dead_line: U256 = runtime::get_named_arg(consts::DEAD_LINE_RUNTIME_ARG_NAME);

    // if dead_line < U256::from(u64::from(runtime::get_blocktime())) {
    //     runtime::revert(error::Error::Expired);
    // }

    let amounts: (U256, U256) = SwapperyRouter::default()._add_liquidity(
        token0,
        token1,
        amount0_desired,
        amount1_desired,
        amount0_min,
        amount1_min,
    );
    let pair: Address = SwapperyRouter::default().get_pair_for(token0, token1);
    let caller: Address = helpers::get_immediate_caller_address().unwrap_or_revert();
    runtime::call_contract::<()>(
        token0,
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => caller,
            RECIPIENT_RUNTIME_ARG_NAME => pair,
            AMOUNT_RUNTIME_ARG_NAME => amounts.0
        },
    );
    runtime::call_contract::<()>(
        token1,
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => caller,
            RECIPIENT_RUNTIME_ARG_NAME => pair,
            AMOUNT_RUNTIME_ARG_NAME => amounts.1
        },
    );
    let liquidity: U256 = runtime::call_versioned_contract(
        *pair.as_contract_package_hash().unwrap_or_revert(),
        None,
        consts::MINT_ENTRY_POINT_NAME,
        runtime_args! {
            consts::TO_RUNTIME_ARG_NAME => to,
            consts::FEETO_KEY_NAME => SwapperyRouter::default().read_feeto(),
        },
    );
    let event = event::RouterEvent::AddLiquidity {
        token0: token0.to_formatted_string(),
        token1: token1.to_formatted_string(),
        amount0: amounts.0,
        amount1: amounts.1,
        recipient: to.as_account_hash().unwrap().to_formatted_string(),
    };
    helpers::emit(&event);
    runtime::ret(CLValue::from_t(liquidity).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn remove_liquidity() {
    let token0_key: Key = runtime::get_named_arg(consts::TOKEN0_RUNTIME_ARG_NAME);
    let token1_key: Key = runtime::get_named_arg(consts::TOKEN1_RUNTIME_ARG_NAME);
    let _token0_hash: HashAddr = token0_key.into_hash().unwrap_or_revert();
    let token0: ContractHash = ContractHash::new(_token0_hash);
    let _token1_hash: HashAddr = token1_key.into_hash().unwrap_or_revert();
    let token1: ContractHash = ContractHash::new(_token1_hash);
    let liquidity: U256 = runtime::get_named_arg(consts::LIQUIDITY_RUNTIME_ARG_NAME);
    let amount0_min: U256 = runtime::get_named_arg(consts::AMOUNT0_MIN_RUNTIME_ARG_NAME);
    let amount1_min: U256 = runtime::get_named_arg(consts::AMOUNT1_MIN_RUNTIME_ARG_NAME);
    let to_key: Key = runtime::get_named_arg(consts::TO_RUNTIME_ARG_NAME);
    let to = Address::from(AccountHash::new(to_key.into_hash().unwrap_or_revert()));
    // let dead_line: U256 = runtime::get_named_arg(consts::DEAD_LINE_RUNTIME_ARG_NAME);

    // if dead_line < U256::from(u64::from(runtime::get_blocktime())) {
    //     runtime::revert(error::Error::Expired);
    // }

    let pair: Address = SwapperyRouter::default().get_pair_for(token0, token1);
    let caller: Address = helpers::get_immediate_caller_address().unwrap_or_revert();

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
        consts::BURN_ENTRY_POINT_NAME,
        runtime_args! {
            consts::TO_RUNTIME_ARG_NAME => to,
            consts::FEETO_KEY_NAME => SwapperyRouter::default().read_feeto(),
        },
    );
    if amounts.0 < amount0_min {
        runtime::revert(error::Error::InsufficientAAmount);
    }
    if token0.eq(&SwapperyRouter::default().wcspr_token()) {
        runtime::call_contract::<()>(
            SwapperyRouter::default().wcspr_token(),
            consts::WITHDRAW_ENTRY_POINT_NAME,
            runtime_args! {
                consts::AMOUNT_RUNTIME_ARG_NAME => amounts.0,
                consts::TO_RUNTIME_ARG_NAME => to
            },
        );
    }
    if amounts.1 < amount1_min {
        runtime::revert(error::Error::InsufficientBAmount);
    }
    if token1.eq(&SwapperyRouter::default().wcspr_token()) {
        runtime::call_contract::<()>(
            SwapperyRouter::default().wcspr_token(),
            consts::WITHDRAW_ENTRY_POINT_NAME,
            runtime_args! {
                consts::AMOUNT_RUNTIME_ARG_NAME => amounts.1,
                consts::TO_RUNTIME_ARG_NAME => to
            },
        );
    }
    let event = event::RouterEvent::RemoveLiquidity {
        token0: token0.to_formatted_string(),
        token1: token1.to_formatted_string(),
        liquidity: liquidity,
        recipient: to.as_account_hash().unwrap().to_formatted_string(),
    };
    helpers::emit(&event);
}

#[no_mangle]
pub extern "C" fn swap_exact_tokens_for_tokens() {
    let amount_in: U256 = runtime::get_named_arg(consts::AMOUNT_IN_RUNTIME_ARG_NAME);
    let amount_out_min: U256 = runtime::get_named_arg(consts::AMOUNT_OUT_MIN_RUNTIME_ARG_NAME);
    let path_key: Vec<Key> = runtime::get_named_arg(consts::PATH_RUNTIME_ARG_NAME);
    let to_key: Key = runtime::get_named_arg(consts::TO_RUNTIME_ARG_NAME);
    let to = Address::from(AccountHash::new(to_key.into_hash().unwrap_or_revert()));
    // let dead_line: U256 = runtime::get_named_arg(consts::DEAD_LINE_RUNTIME_ARG_NAME);

    // if dead_line < U256::from(u64::from(runtime::get_blocktime())) {
    //     runtime::revert(error::Error::Expired);
    // }

    let mut path: Vec<ContractHash> = Vec::new();
    for i in 0..path_key.len() {
        path.push({
            let _hash = path_key.get(i).unwrap().into_hash().unwrap_or_revert();
            ContractHash::new(_hash)
        });
    }

    let amounts: Vec<U256> = SwapperyRouter::default().get_amounts_out(amount_in, path.clone());

    if !(amounts.last().unwrap_or_revert() >= &amount_out_min) {
        runtime::revert(error::Error::InsufficientOutputAmount);
    }

    let caller: Address = helpers::get_immediate_caller_address().unwrap_or_revert();
    runtime::call_contract::<()>(
        *path.get(0).unwrap_or_revert(),
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => caller,
            RECIPIENT_RUNTIME_ARG_NAME => SwapperyRouter::default().get_pair_for(
                *path.get(0).unwrap_or_revert(),
                *path.get(1).unwrap_or_revert(),
            ),
            AMOUNT_RUNTIME_ARG_NAME => *amounts.clone().get(0).unwrap_or_revert(),
        },
    );
    SwapperyRouter::default()._swap(amounts.clone(), path.clone(), to);
    let event = event::RouterEvent::SwapExactIn {
        amount_in: (*amounts.get(0).unwrap_or_revert()),
        amount_out: { *amounts.last().unwrap_or_revert() },
        path,
        recipient: to.as_account_hash().unwrap().to_formatted_string(),
    };
    helpers::emit(&event);
}

#[no_mangle]
pub extern "C" fn swap_tokens_for_exact_tokens() {
    let amount_out: U256 = runtime::get_named_arg(consts::AMOUNT_OUT_RUNTIME_ARG_NAME);
    let amount_in_max: U256 = runtime::get_named_arg(consts::AMOUNT_IN_MAX_RUNTIME_ARG_NAME);
    let path_key: Vec<Key> = runtime::get_named_arg(consts::PATH_RUNTIME_ARG_NAME);
    let to_key: Key = runtime::get_named_arg(consts::TO_RUNTIME_ARG_NAME);
    let to = Address::from(AccountHash::new(to_key.into_hash().unwrap_or_revert()));
    // let dead_line: U256 = runtime::get_named_arg(consts::DEAD_LINE_RUNTIME_ARG_NAME);

    // if dead_line < U256::from(u64::from(runtime::get_blocktime())) {
    //     runtime::revert(error::Error::Expired);
    // }

    let mut path: Vec<ContractHash> = Vec::new();
    for i in 0..path_key.len() {
        path.push({
            let _hash = path_key.get(i).unwrap().into_hash().unwrap_or_revert();
            ContractHash::new(_hash)
        });
    }

    let amounts: Vec<U256> = SwapperyRouter::default().get_amounts_in(amount_out, path.clone());

    if !(amounts.get(0).unwrap_or_revert() <= &amount_in_max) {
        runtime::revert(error::Error::InsufficientInputAmount);
    }

    let caller: Address = helpers::get_immediate_caller_address().unwrap_or_revert();
    runtime::call_contract::<()>(
        *path.get(0).unwrap_or_revert(),
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => caller,
            RECIPIENT_RUNTIME_ARG_NAME => SwapperyRouter::default().get_pair_for(
                *path.get(0).unwrap_or_revert(),
                *path.get(1).unwrap_or_revert(),
            ),
            AMOUNT_RUNTIME_ARG_NAME => *amounts.clone().get(0).unwrap_or_revert(),
        },
    );
    SwapperyRouter::default()._swap(amounts.clone(), path.clone(), to);
    let event = event::RouterEvent::SwapExactOut {
        amount_in: (*amounts.get(0).unwrap_or_revert()),
        amount_out: { *amounts.last().unwrap_or_revert() },
        path,
        recipient: to.as_account_hash().unwrap().to_formatted_string(),
    };
    helpers::emit(&event);
}

#[no_mangle]
pub extern "C" fn swap_exact_tokens_for_tokens_supporting_fee() {
    let amount_in: U256 = runtime::get_named_arg(consts::AMOUNT_IN_RUNTIME_ARG_NAME);
    let amount_out_min: U256 = runtime::get_named_arg(consts::AMOUNT_OUT_MIN_RUNTIME_ARG_NAME);
    let path_key: Vec<Key> = runtime::get_named_arg(consts::PATH_RUNTIME_ARG_NAME);
    let to_key: Key = runtime::get_named_arg(consts::TO_RUNTIME_ARG_NAME);
    let to = Address::from(AccountHash::new(to_key.into_hash().unwrap_or_revert()));
    // let dead_line: U256 = runtime::get_named_arg(consts::DEAD_LINE_RUNTIME_ARG_NAME);

    // if dead_line < U256::from(u64::from(runtime::get_blocktime())) {
    //     runtime::revert(error::Error::Expired);
    // }

    let mut path: Vec<ContractHash> = Vec::new();
    for i in 0..path_key.len() {
        path.push({
            let _hash = path_key.get(i).unwrap().into_hash().unwrap_or_revert();
            ContractHash::new(_hash)
        });
    }

    let caller: Address = helpers::get_immediate_caller_address().unwrap_or_revert();
    runtime::call_contract::<()>(
        *path.get(0).unwrap_or_revert(),
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => caller,
            RECIPIENT_RUNTIME_ARG_NAME => SwapperyRouter::default().get_pair_for(
                *path.get(0).unwrap_or_revert(),
                *path.get(1).unwrap_or_revert(),
            ),
            AMOUNT_RUNTIME_ARG_NAME => amount_in,
        },
    );

    let balance_before: U256 = runtime::call_contract(
        *path.last().unwrap_or_revert(),
        consts::BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            consts::ADDRESS_RUNTIME_ARG_NAME => to,
        },
    );
    SwapperyRouter::default()._swap_supporting_fee(path.clone(), to);

    let balance_after: U256 = runtime::call_contract(
        *path.last().unwrap_or_revert(),
        consts::BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            consts::ADDRESS_RUNTIME_ARG_NAME => to,
        },
    );
    if (balance_after - balance_before) < amount_out_min {
        runtime::revert(error::Error::InsufficientOutputAmount);
    }
}

#[no_mangle]
fn call() {
    // let feeto = Address::Account(runtime::get_named_arg(consts::FEETO_KEY_NAME));
    // let feeto_setter = Address::Account(runtime::get_named_arg(consts::FEETO_SETTER_KEY_NAME));
    let wcspr_token_key: Key = runtime::get_named_arg(consts::WCSPR_CONTRACT_KEY_NAME);
    let wcspr_token = ContractHash::new(wcspr_token_key.into_hash().unwrap_or_revert());
    let feeto_key: Key = runtime::get_named_arg(consts::FEETO_KEY_NAME);
    let feeto = Address::from(AccountHash::new(feeto_key.into_hash().unwrap_or_revert()));
    let feeto_setter_key: Key = runtime::get_named_arg(consts::FEETO_SETTER_KEY_NAME);
    let feeto_setter = Address::from(AccountHash::new(
        feeto_setter_key.into_hash().unwrap_or_revert(),
    ));
    let contract_key_name: String = runtime::get_named_arg(consts::CONTRACT_KEY_NAME_ARG_NAME);

    let _ = SwapperyRouter::create(feeto, feeto_setter, wcspr_token, contract_key_name);
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
