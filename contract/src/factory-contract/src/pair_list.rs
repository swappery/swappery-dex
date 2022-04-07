use alloc::string::String;

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{bytesrepr::ToBytes, URef, U256};

use casper_erc20::{Error, Address};

use crate::helpers::{get_uref};

const PAIR_LIST_KEY_NAME: &str = "pair_list";

pub(crate) fn get_pair_list_uref() -> URef {
    get_uref(PAIR_LIST_KEY_NAME)
}

fn make_dictionary_item_key(token0: Address, token1: Address) -> String {
    let mut preimage = Vec::new();
    preimage.append(&mut token0.to_bytes().unwrap_or_revert());
    preimage.append(&mut token1.to_bytes().unwrap_or_revert());

    let key_bytes = runtime::blake2b(&preimage);
    hex::encode(&key_bytes)
}

pub(crate) fn add_pair_for(
    pair_list_uref: URef,
    token0: Address,
    token1: Address,
    pair_address: Address,
) {
    let dictionary_item_key = make_dictionary_item_key(token0, token1);
    storage::dictionary_put(pair_list_uref, &dictionary_item_key, pair_address)
}

pub(crate) fn get_pair_for(pair_list_uref: URef, token0: Address, token1: Address) -> Address {
    let dictionary_item_key = make_dictionary_item_key(token0, token1);
    storage::dictionary_get(pair_list_uref, &dictionary_item_key)
        .unwrap_or_revert()
        .unwrap_or_default()
}
