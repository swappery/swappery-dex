#![no_std]
#![no_main]

extern crate alloc;

mod entry_points;
mod constants;
mod helpers;
mod variables;

use alloc::string::String;

use once_cell::unsync::OnceCell;

use core::ops::{Deref, DerefMut};

use casper_erc20::{
    constants::{
        ADDRESS_RUNTIME_ARG_NAME, AMOUNT_RUNTIME_ARG_NAME,
        OWNER_RUNTIME_ARG_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        SPENDER_RUNTIME_ARG_NAME,
    },
    Address, ERC20, Error
};

use casper_types::{CLValue, U256, URef, contracts::NamedKeys, Key};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use constants::{
    RESERVE0_KEY_NAME, RESERVE1_KEY_NAME, TOKEN0_KEY_NAME, TOKEN1_KEY_NAME,
    LOCKED_FLAG_KEY_NAME, TO_RUNTIME_ARG_NAME, AMOUNT0_RUNTIME_ARG_NAME, AMOUNT1_RUNTIME_ARG_NAME,
};

#[derive(Default)]
pub struct SwapperyPair {
    erc20: ERC20,
    reserve0_uref: OnceCell<URef>,
    reserve1_uref: OnceCell<URef>,
    locked_uref: OnceCell<URef>,
}

impl SwapperyPair {
    fn new(erc20: ERC20, reserve0_uref: URef, reserve1_uref: URef, locked_uref: URef) -> Self {
        Self {
            erc20: erc20,
            reserve0_uref: reserve0_uref.into(),
            reserve1_uref: reserve1_uref.into(),
            locked_uref: locked_uref.into(),
        }
    }
    fn reserve0_uref(&self) -> URef {
        *self
            .reserve0_uref
            .get_or_init(variables::reserve0_uref)
    }

    fn read_reserve0(&self) -> U256 {
        variables::read_reserve_from(self.reserve0_uref())
    }

    fn write_reserve0(&self, reserve0: U256) {
        variables::write_reserve_to(self.reserve0_uref(), reserve0)
    }

    fn reserve1_uref(&self) -> URef {
        *self
            .reserve1_uref
            .get_or_init(variables::reserve1_uref)
    }

    fn read_reserve1(&self) -> U256 {
        variables::read_reserve_from(self.reserve1_uref())
    }

    fn write_reserve1(&self, reserve1: U256) {
        variables::write_reserve_to(self.reserve1_uref(), reserve1)
    }

    fn locked_uref(&self) -> URef {
        *self
            .locked_uref
            .get_or_init(variables::locked_uref)
    }

    fn read_locked(&self) -> bool {
        variables::read_locked_from(self.locked_uref())
    }

    fn write_locked(&self, locked: bool) {
        variables::write_locked_to(self.locked_uref(), locked)
    }

    pub fn token0(&self) -> Address {
        helpers::read_from(TOKEN0_KEY_NAME)
    }

    pub fn token1(&self) -> Address {
        helpers::read_from(TOKEN1_KEY_NAME)
    }

    pub fn create(name: String, symbol: String, decimals: u8, initial_supply: U256, contract_key_name: &str, token0: Address, token1: Address, reserve0: U256, reserve1: U256, locked: bool)
     -> Result<SwapperyPair, Error> {
        let reserve0_uref = storage::new_uref(reserve0).into_read_write();
        let reserve1_uref = storage::new_uref(reserve1).into_read_write();
        let locked_uref = storage::new_uref(locked).into_read_write();

        let token0_key = {
            let token0_uref = storage::new_uref(token0).into_read();
            Key::from(token0_uref)
        };

        let token1_key = {
            let token1_uref = storage::new_uref(token1).into_read();
            Key::from(token1_uref)
        };

        let mut named_keys = NamedKeys::new();

        named_keys.insert(String::from(RESERVE0_KEY_NAME), Key::from(reserve0_uref));
        named_keys.insert(String::from(RESERVE1_KEY_NAME), Key::from(reserve1_uref));
        named_keys.insert(String::from(TOKEN0_KEY_NAME), token0_key);
        named_keys.insert(String::from(TOKEN1_KEY_NAME), token1_key);
        named_keys.insert(String::from(LOCKED_FLAG_KEY_NAME), Key::from(locked_uref));
        let erc20 = ERC20::install_custom(
            name,
            symbol,
            decimals,
            initial_supply,
            contract_key_name,
            named_keys,
            entry_points::default(),
        )?;
        Ok(SwapperyPair::new(erc20, reserve0_uref, reserve1_uref, locked_uref ))
    }
}

impl Deref for SwapperyPair {
    type Target = ERC20;

    fn deref(&self) -> &Self::Target {
        &self.erc20
    }
}

impl DerefMut for SwapperyPair {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.erc20
    }
}

#[no_mangle]
pub extern "C" fn name() {
    let name = ERC20::default().name();
    runtime::ret(CLValue::from_t(name).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn symbol() {
    let symbol = ERC20::default().symbol();
    runtime::ret(CLValue::from_t(symbol).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn decimals() {
    let decimals = ERC20::default().decimals();
    runtime::ret(CLValue::from_t(decimals).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn total_supply() {
    let total_supply = ERC20::default().total_supply();
    runtime::ret(CLValue::from_t(total_supply).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn balance_of() {
    let address: Address = runtime::get_named_arg(ADDRESS_RUNTIME_ARG_NAME);
    let balance = ERC20::default().balance_of(address);
    runtime::ret(CLValue::from_t(balance).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer() {
    let recipient: Address = runtime::get_named_arg(RECIPIENT_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);

    ERC20::default()
        .transfer(recipient, amount)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn approve() {
    let spender: Address = runtime::get_named_arg(SPENDER_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);

    ERC20::default().approve(spender, amount).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn allowance() {
    let owner: Address = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    let spender: Address = runtime::get_named_arg(SPENDER_RUNTIME_ARG_NAME);
    let val = ERC20::default().allowance(owner, spender);
    runtime::ret(CLValue::from_t(val).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer_from() {
    let owner: Address = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    let recipient: Address = runtime::get_named_arg(RECIPIENT_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);
    ERC20::default()
        .transfer_from(owner, recipient, amount)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn mint() {
    let to: Address = runtime::get_named_arg(TO_RUNTIME_ARG_NAME);
    //::default().mint(owner, amount).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn burn() {
    let to: Address = runtime::get_named_arg(TO_RUNTIME_ARG_NAME);
    //ERC20::default().burn(owner, amount).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn swap() {
    let amount0: U256 = runtime::get_named_arg(AMOUNT0_RUNTIME_ARG_NAME);
    let amount1: U256 = runtime::get_named_arg(AMOUNT1_RUNTIME_ARG_NAME);
    let to: Address = runtime::get_named_arg(TO_RUNTIME_ARG_NAME);
}