#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

extern crate alloc;

pub mod address;
mod allowances;
mod balances;
pub mod constants;
mod entry_points;
pub mod error;
mod helpers;
mod variables;

use alloc::string::String;

use once_cell::unsync::OnceCell;

use casper_types::{
    account::AccountHash, contracts::NamedKeys, runtime_args, CLValue, Key, RuntimeArgs, URef, U256,
    ContractHash,
};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use constants as consts;
pub use error::Error;
pub use address::Address;

#[derive(Default)]
pub struct SwapperyPair {
    balances_uref: OnceCell<URef>,
    allowances_uref: OnceCell<URef>,
    total_supply_uref: OnceCell<URef>,
    reserve0_uref: OnceCell<URef>,
    reserve1_uref: OnceCell<URef>,
    locked_uref: OnceCell<URef>,
    klast_uref: OnceCell<URef>,
}

impl SwapperyPair {
    fn new(
        balances_uref: URef,
        allowances_uref: URef,
        total_supply_uref: URef,
        reserve0_uref: URef,
        reserve1_uref: URef,
        locked_uref: URef,
        klast_uref: URef,
    ) -> Self {
        Self {
            balances_uref: balances_uref.into(),
            allowances_uref: allowances_uref.into(),
            total_supply_uref: total_supply_uref.into(),
            reserve0_uref: reserve0_uref.into(),
            reserve1_uref: reserve1_uref.into(),
            locked_uref: locked_uref.into(),
            klast_uref: klast_uref.into(),
        }
    }

    fn total_supply_uref(&self) -> URef {
        *self
            .total_supply_uref
            .get_or_init(variables::total_supply_uref)
    }

    fn read_total_supply(&self) -> U256 {
        variables::read_total_supply_from(self.total_supply_uref())
    }

    fn write_total_supply(&self, total_supply: U256) {
        variables::write_total_supply_to(self.total_supply_uref(), total_supply)
    }

    fn balances_uref(&self) -> URef {
        *self.balances_uref.get_or_init(balances::get_balances_uref)
    }

    fn read_balance(&self, owner: Address) -> U256 {
        balances::read_balance_from(self.balances_uref(), owner)
    }

    fn write_balance(&mut self, owner: Address, amount: U256) {
        balances::write_balance_to(self.balances_uref(), owner, amount)
    }

    fn allowances_uref(&self) -> URef {
        *self
            .allowances_uref
            .get_or_init(allowances::allowances_uref)
    }

    fn read_allowance(&self, owner: Address, spender: Address) -> U256 {
        allowances::read_allowance_from(self.allowances_uref(), owner, spender)
    }

    fn write_allowance(&mut self, owner: Address, spender: Address, amount: U256) {
        allowances::write_allowance_to(self.allowances_uref(), owner, spender, amount)
    }

    fn transfer_balance(
        &mut self,
        sender: Address,
        recipient: Address,
        amount: U256,
    ) -> Result<(), Error> {
        balances::transfer_balance(self.balances_uref(), sender, recipient, amount)
    }

    /// Returns the name of the token.
    pub fn name(&self) -> String {
        helpers::read_from(consts::NAME_KEY_NAME)
    }

    /// Returns the symbol of the token.
    pub fn symbol(&self) -> String {
        helpers::read_from(consts::SYMBOL_KEY_NAME)
    }

    /// Returns the decimals of the token.
    pub fn decimals(&self) -> u8 {
        helpers::read_from(consts::DECIMALS_KEY_NAME)
    }

    /// Returns the total supply of the token.
    pub fn total_supply(&self) -> U256 {
        self.read_total_supply()
    }

    /// Returns the balance of `owner`.
    pub fn balance_of(&self, owner: Address) -> U256 {
        self.read_balance(owner)
    }

    /// Transfers `amount` of tokens from the direct caller to `recipient`.
    pub fn transfer(&mut self, recipient: Address, amount: U256) -> Result<(), Error> {
        let sender = helpers::get_immediate_caller_address()?;
        self.transfer_balance(sender, recipient, amount)
    }

    /// Transfers `amount` of tokens from `owner` to `recipient` if the direct caller has been
    /// previously approved to spend the specified amount on behalf of the owner.
    pub fn transfer_from(
        &mut self,
        owner: Address,
        recipient: Address,
        amount: U256,
    ) -> Result<(), Error> {
        let spender = helpers::get_immediate_caller_address()?;
        if amount.is_zero() {
            return Ok(());
        }
        let spender_allowance = self.read_allowance(owner, spender);
        let new_spender_allowance = spender_allowance
            .checked_sub(amount)
            .ok_or(Error::InsufficientAllowance)?;
        self.transfer_balance(owner, recipient, amount)?;
        self.write_allowance(owner, spender, new_spender_allowance);
        Ok(())
    }

    /// Allows `spender` to transfer up to `amount` of the direct caller's tokens.
    pub fn approve(&mut self, spender: Address, amount: U256) -> Result<(), Error> {
        let owner = helpers::get_immediate_caller_address()?;
        self.write_allowance(owner, spender, amount);
        Ok(())
    }

    /// Returns the amount of `owner`'s tokens allowed to be spent by `spender`.
    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.read_allowance(owner, spender)
    }

    /// Mints `amount` new tokens and adds them to `owner`'s balance and to the token total supply.
    ///
    /// # Security
    ///
    /// This offers no security whatsoever, hence it is advised to NOT expose this method through a
    /// public entry point.
    pub fn mint(&mut self, owner: Address, amount: U256) -> Result<(), Error> {
        let new_balance = {
            let balance = self.read_balance(owner);
            balance.checked_add(amount).ok_or(Error::OverFlow)?
        };
        let new_total_supply = {
            let total_supply: U256 = self.read_total_supply();
            total_supply.checked_add(amount).ok_or(Error::OverFlow)?
        };
        self.write_balance(owner, new_balance);
        self.write_total_supply(new_total_supply);
        Ok(())
    }

    /// Burns (i.e. subtracts) `amount` of tokens from `owner`'s balance and from the token total
    /// supply.
    ///
    /// # Security
    ///
    /// This offers no security whatsoever, hence it is advised to NOT expose this method through a
    /// public entry point.
    pub fn burn(&mut self, owner: Address, amount: U256) -> Result<(), Error> {
        let new_balance = {
            let balance = self.read_balance(owner);
            balance
                .checked_sub(amount)
                .ok_or(Error::InsufficientBalance)?
        };
        let new_total_supply = {
            let total_supply = self.read_total_supply();
            total_supply.checked_sub(amount).ok_or(Error::OverFlow)?
        };
        self.write_balance(owner, new_balance);
        self.write_total_supply(new_total_supply);
        Ok(())
    }

    fn reserve0_uref(&self) -> URef {
        *self.reserve0_uref.get_or_init(variables::reserve0_uref)
    }

    fn read_reserve0(&self) -> U256 {
        variables::read_reserve_from(self.reserve0_uref())
    }

    fn write_reserve0(&self, reserve0: U256) {
        variables::write_reserve_to(self.reserve0_uref(), reserve0)
    }

    fn reserve1_uref(&self) -> URef {
        *self.reserve1_uref.get_or_init(variables::reserve1_uref)
    }

    fn read_reserve1(&self) -> U256 {
        variables::read_reserve_from(self.reserve1_uref())
    }

    fn write_reserve1(&self, reserve1: U256) {
        variables::write_reserve_to(self.reserve1_uref(), reserve1)
    }

    fn locked_uref(&self) -> URef {
        *self.locked_uref.get_or_init(variables::locked_uref)
    }

    fn read_locked(&self) -> bool {
        variables::read_locked_from(self.locked_uref())
    }

    fn write_locked(&self, locked: bool) {
        variables::write_locked_to(self.locked_uref(), locked)
    }

    fn klast_uref(&self) -> URef {
        *self.klast_uref.get_or_init(variables::klast_uref)
    }

    fn read_klast(&self) -> U256 {
        variables::read_klast_from(self.klast_uref())
    }

    fn write_klast(&self, klast: U256) {
        variables::write_klast_to(self.klast_uref(), klast)
    }

    pub fn reserve0(&self) -> U256 {
        self.read_reserve0()
    }

    pub fn reserve1(&self) -> U256 {
        self.read_reserve1()
    }

    pub fn locked(&self) -> bool {
        self.read_locked()
    }

    pub fn token0(&self) -> ContractHash {
        helpers::read_from(consts::TOKEN0_KEY_NAME)
    }

    pub fn token1(&self) -> ContractHash {
        helpers::read_from(consts::TOKEN1_KEY_NAME)
    }

    pub fn factory(&self) -> Address {
        helpers::read_from(consts::FACTORY_KEY_NAME)
    }

    pub fn klast(&self) -> U256 {
        self.read_klast()
    }

    pub fn _update(&mut self, balance0: U256, balance1: U256) {
        if !(balance0 <= U256::MAX || balance1 <= U256::MAX) {
            runtime::revert(Error::OverFlow);
        }
        self.write_reserve0(balance0);
        self.write_reserve1(balance1);
    }

    pub fn _mint_fee(&mut self, _reserve0: U256, _reserve1: U256) -> bool {
        // address feeTo = IPancakeFactory(factory).feeTo();
        // fee_on = feeTo != address(0);
        let _klast: U256 = self.klast(); // gas savings
        if true {
            if !(_klast.is_zero()) {
                let mut rootk: U256 = _reserve0 * _reserve1;
                rootk = rootk.integer_sqrt();
                let rootklast = _klast.integer_sqrt();
                if rootk > rootklast {
                    let numerator: U256 =
                        U256::from(self.read_total_supply()) * (rootk - rootklast);
                    let denominator: U256 = rootk * U256::from(3u64) + rootklast;
                    let liquidity: U256 = numerator / denominator;
                    if liquidity > U256::zero() {
                        //ERC20::default().mint(feeTo, liquidity).unwrap_or_revert();
                    }
                }
            }
        } else if !(_klast.is_zero()) {
            self.write_klast(U256::zero());
        }
        return true;
    }

    pub fn create(
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: U256,
        contract_key_name: &str,
        token0: ContractHash,
        token1: ContractHash,
    ) -> Result<SwapperyPair, Error> {
        let balances_uref = storage::new_dictionary(consts::BALANCES_KEY_NAME).unwrap_or_revert();
        let allowances_uref = storage::new_dictionary(consts::ALLOWANCES_KEY_NAME).unwrap_or_revert();
        let total_supply_uref = storage::new_uref(initial_supply).into_read_write();
        let reserve0_uref = storage::new_uref(U256::zero()).into_read_write();
        let reserve1_uref = storage::new_uref(U256::zero()).into_read_write();
        let locked_uref = storage::new_uref(false).into_read_write();
        let klast_uref = storage::new_uref(U256::zero()).into_read_write();

        let name_key = {
            let name_uref = storage::new_uref(name).into_read();
            Key::from(name_uref)
        };

        let symbol_key = {
            let symbol_uref = storage::new_uref(symbol).into_read();
            Key::from(symbol_uref)
        };

        let decimals_key = {
            let decimals_uref = storage::new_uref(decimals).into_read();
            Key::from(decimals_uref)
        };

        let total_supply_key = Key::from(total_supply_uref);

        let balances_dictionary_key = {
            // Sets up initial balance for the caller - either an account, or a contract.
            let caller = helpers::get_caller_address()?;
            balances::write_balance_to(balances_uref, caller, initial_supply);

            runtime::remove_key(consts::BALANCES_KEY_NAME);

            Key::from(balances_uref)
        };

        let allowances_dictionary_key = {
            runtime::remove_key(consts::ALLOWANCES_KEY_NAME);

            Key::from(allowances_uref)
        };

        let token0_key = {
            let token0_uref = storage::new_uref(token0).into_read();
            Key::from(token0_uref)
        };

        let token1_key = {
            let token1_uref = storage::new_uref(token1).into_read();
            Key::from(token1_uref)
        };

        let factory_key = {
            let factory_uref =
                storage::new_uref(helpers::get_caller_address().unwrap_or_revert()).into_read();
            Key::from(factory_uref)
        };

        let mut named_keys = NamedKeys::new();

        named_keys.insert(String::from(consts::NAME_KEY_NAME), name_key);
        named_keys.insert(String::from(consts::SYMBOL_KEY_NAME), symbol_key);
        named_keys.insert(String::from(consts::DECIMALS_KEY_NAME), decimals_key);
        named_keys.insert(String::from(consts::BALANCES_KEY_NAME), balances_dictionary_key);
        named_keys.insert(String::from(consts::ALLOWANCES_KEY_NAME), allowances_dictionary_key);
        named_keys.insert(String::from(consts::TOTAL_SUPPLY_KEY_NAME), total_supply_key);
        named_keys.insert(String::from(consts::RESERVE0_KEY_NAME), Key::from(reserve0_uref));
        named_keys.insert(String::from(consts::RESERVE1_KEY_NAME), Key::from(reserve1_uref));
        named_keys.insert(String::from(consts::TOKEN0_KEY_NAME), token0_key);
        named_keys.insert(String::from(consts::TOKEN1_KEY_NAME), token1_key);
        named_keys.insert(String::from(consts::LOCKED_FLAG_KEY_NAME), Key::from(locked_uref));
        named_keys.insert(String::from(consts::KLAST_KEY_NAME), Key::from(klast_uref));
        named_keys.insert(String::from(consts::FACTORY_KEY_NAME), factory_key);

        let (contract_hash, _version) = storage::new_contract(
            entry_points::default(),
            Some(named_keys),
            Some(String::from(contract_key_name)),
            None,
        );

        let mut contract_hash_key_name: String = String::from(contract_key_name);
        contract_hash_key_name.push_str("_contract_hash");
        // Hash of the installed contract will be reachable through named keys.
        runtime::put_key(contract_hash_key_name.as_str(), Key::from(contract_hash));
        Ok(SwapperyPair::new(
            balances_uref,
            allowances_uref,
            total_supply_uref,
            reserve0_uref,
            reserve1_uref,
            locked_uref,
            klast_uref,
        ))
    }
}

#[no_mangle]
pub extern "C" fn name() {
    let name = SwapperyPair::default().name();
    runtime::ret(CLValue::from_t(name).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn symbol() {
    let symbol = SwapperyPair::default().symbol();
    runtime::ret(CLValue::from_t(symbol).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn decimals() {
    let decimals = SwapperyPair::default().decimals();
    runtime::ret(CLValue::from_t(decimals).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn total_supply() {
    let total_supply = SwapperyPair::default().total_supply();
    runtime::ret(CLValue::from_t(total_supply).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn balance_of() {
    let address: Address = runtime::get_named_arg(consts::ADDRESS_RUNTIME_ARG_NAME);
    let balance = SwapperyPair::default().balance_of(address);
    runtime::ret(CLValue::from_t(balance).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer() {
    let recipient: Address = runtime::get_named_arg(consts::RECIPIENT_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(consts::AMOUNT_RUNTIME_ARG_NAME);

    SwapperyPair::default()
        .transfer(recipient, amount)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn approve() {
    let spender: Address = runtime::get_named_arg(consts::SPENDER_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(consts::AMOUNT_RUNTIME_ARG_NAME);

    SwapperyPair::default()
        .approve(spender, amount)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn allowance() {
    let owner: Address = runtime::get_named_arg(consts::OWNER_RUNTIME_ARG_NAME);
    let spender: Address = runtime::get_named_arg(consts::SPENDER_RUNTIME_ARG_NAME);
    let val = SwapperyPair::default().allowance(owner, spender);
    runtime::ret(CLValue::from_t(val).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer_from() {
    let owner: Address = runtime::get_named_arg(consts::OWNER_RUNTIME_ARG_NAME);
    let recipient: Address = runtime::get_named_arg(consts::RECIPIENT_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(consts::AMOUNT_RUNTIME_ARG_NAME);
    SwapperyPair::default()
        .transfer_from(owner, recipient, amount)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn get_reserves() {
    let reserves: (U256, U256) = (
        SwapperyPair::default().reserve0(),
        SwapperyPair::default().reserve1(),
    );
    runtime::ret(CLValue::from_t(reserves).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn mint() {
    let locked = SwapperyPair::default().locked();
    if locked {
        runtime::revert(Error::Locked);
    }
    SwapperyPair::default().write_locked(true);

    let to: Address = runtime::get_named_arg(consts::TO_RUNTIME_ARG_NAME);

    let _reserve0: U256 = SwapperyPair::default().reserve0();
    let _reserve1: U256 = SwapperyPair::default().reserve1();

    let token0: ContractHash = SwapperyPair::default().token0();
    let token1: ContractHash = SwapperyPair::default().token1();

    let self_addr = helpers::get_self_address().unwrap_or_revert();

    let balance0: U256 = runtime::call_contract(
        token0,
        consts::BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            consts::ADDRESS_RUNTIME_ARG_NAME => self_addr
        },
    );
    let balance1: U256 = runtime::call_contract(
        token1,
        consts::BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            consts::ADDRESS_RUNTIME_ARG_NAME => self_addr
        },
    );
    let amount0: U256 = balance0 - _reserve0;
    let amount1: U256 = balance1 - _reserve1;

    let fee_on: bool = SwapperyPair::default()._mint_fee(_reserve0, _reserve1);
    let _total_supply: U256 = SwapperyPair::default().total_supply();
    let liquidity: U256;
    if _total_supply.is_zero() {
        liquidity = (U256::from(amount0 * amount1).integer_sqrt()) - consts::MINIMUM_LIQUIDITY;
        SwapperyPair::default()
            .mint(
                Address::from(AccountHash::new([0u8; 32])),
                U256::from(consts::MINIMUM_LIQUIDITY),
            )
            .unwrap_or_revert();
    } else {
        liquidity = U256::min(
            amount0 * _total_supply / _reserve0,
            amount1 * _total_supply / _reserve1,
        );
    }
    if !(liquidity > U256::zero()) {
        runtime::revert(Error::InsufficientLiquidityMinted);
    }
    SwapperyPair::default()
        .mint(to, liquidity)
        .unwrap_or_revert();

    SwapperyPair::default()._update(balance0, balance1);
    if fee_on {
        SwapperyPair::default()
            .write_klast(SwapperyPair::default().reserve0() * SwapperyPair::default().reserve1());
    }

    SwapperyPair::default().write_locked(false);

    runtime::ret(CLValue::from_t(liquidity).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn burn() {
    let locked = SwapperyPair::default().locked();
    if locked {
        runtime::revert(Error::Locked);
    }
    SwapperyPair::default().write_locked(true);

    let to: Address = runtime::get_named_arg(consts::TO_RUNTIME_ARG_NAME);

    let _reserve0: U256 = SwapperyPair::default().reserve0();
    let _reserve1: U256 = SwapperyPair::default().reserve1();

    let token0: ContractHash = SwapperyPair::default().token0();
    let token1: ContractHash = SwapperyPair::default().token1();

    let self_addr = helpers::get_self_address().unwrap_or_revert();

    let mut balance0: U256 = runtime::call_contract(
        token0,
        consts::BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            consts::ADDRESS_RUNTIME_ARG_NAME => self_addr
        },
    );
    let mut balance1: U256 = runtime::call_contract(
        token1,
        consts::BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            consts::ADDRESS_RUNTIME_ARG_NAME => self_addr
        },
    );
    let liquidity: U256 = SwapperyPair::default().balance_of(self_addr);

    let fee_on: bool = SwapperyPair::default()._mint_fee(_reserve0, _reserve1);
    let _total_supply: U256 = SwapperyPair::default().total_supply();
    let amount0: U256 = liquidity * balance0 / _total_supply;
    let amount1: U256 = liquidity * balance1 / _total_supply;
    if !(amount0 > U256::zero() && amount1 > U256::zero()) {
        runtime::revert(Error::InsufficientLiquidityBurned);
    }

    SwapperyPair::default()
        .burn(self_addr, liquidity)
        .unwrap_or_revert();
    runtime::call_contract::<()>(
        token0,
        consts::TRANSFER_ENTRY_POINT_NAME,
        runtime_args! {
            consts::RECIPIENT_RUNTIME_ARG_NAME => to,
            consts::AMOUNT_RUNTIME_ARG_NAME => amount0
        },
    );
    runtime::call_contract::<()>(
        token1,
        consts::TRANSFER_ENTRY_POINT_NAME,
        runtime_args! {
            consts::RECIPIENT_RUNTIME_ARG_NAME => to,
            consts::AMOUNT_RUNTIME_ARG_NAME => amount1
        },
    );

    balance0 = runtime::call_contract(
        token0,
        consts::BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            consts::ADDRESS_RUNTIME_ARG_NAME => self_addr
        },
    );
    balance1 = runtime::call_contract(
        token1,
        consts::BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            consts::ADDRESS_RUNTIME_ARG_NAME => self_addr
        },
    );

    SwapperyPair::default()._update(balance0, balance1);
    if fee_on {
        SwapperyPair::default()
            .write_klast(SwapperyPair::default().reserve0() * SwapperyPair::default().reserve1());
    }

    SwapperyPair::default().write_locked(false);

    runtime::ret(CLValue::from_t((amount0, amount1)).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn swap() {
    let locked = SwapperyPair::default().locked();
    if locked {
        runtime::revert(Error::Locked);
    }
    SwapperyPair::default().write_locked(true);

    let amount0_out: U256 = runtime::get_named_arg(consts::AMOUNT0_RUNTIME_ARG_NAME);
    let amount1_out: U256 = runtime::get_named_arg(consts::AMOUNT1_RUNTIME_ARG_NAME);
    let to: Address = runtime::get_named_arg(consts::TO_RUNTIME_ARG_NAME);

    if !(amount0_out > U256::zero() || amount1_out > U256::zero()) {
        runtime::revert(Error::InsufficientOutputAmount);
    }

    let _reserve0: U256 = SwapperyPair::default().reserve0();
    let _reserve1: U256 = SwapperyPair::default().reserve1();

    if !(amount0_out < _reserve0 && amount1_out < _reserve1) {
        runtime::revert(Error::InsufficientLiquidity);
    }

    let balance0: U256;
    let balance1: U256;

    let self_addr = helpers::get_self_address().unwrap_or_revert();

    let token0: ContractHash = SwapperyPair::default().token0();
    let token1: ContractHash = SwapperyPair::default().token1();

    // if !(to != token0 && to != token1) {
    //     runtime::revert(Error::InvalidTo);
    // }

    if amount0_out > U256::zero() {
        runtime::call_contract::<()>(
            token0,
            consts::TRANSFER_ENTRY_POINT_NAME,
            runtime_args! {
                consts::RECIPIENT_RUNTIME_ARG_NAME => to,
                consts::AMOUNT_RUNTIME_ARG_NAME => amount0_out
            },
        );
    }
    if amount1_out > U256::zero() {
        runtime::call_contract::<()>(
            token1,
            consts::TRANSFER_ENTRY_POINT_NAME,
            runtime_args! {
                consts::RECIPIENT_RUNTIME_ARG_NAME => to,
                consts::AMOUNT_RUNTIME_ARG_NAME => amount1_out
            },
        );
    }

    //     IPancakeCallee(to).pancakeCall(

    balance0 = runtime::call_contract(
        token0,
        consts::BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            consts::ADDRESS_RUNTIME_ARG_NAME => self_addr
        },
    );
    balance1 = runtime::call_contract(
        token1,
        consts::BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            consts::ADDRESS_RUNTIME_ARG_NAME => self_addr
        },
    );

    let mut amount0_in: U256 = U256::zero();
    if balance0 > (_reserve0 - amount0_out) {
        amount0_in = balance0 - (_reserve0 - amount0_out);
    }

    let mut amount1_in: U256 = U256::zero();
    if balance1 > (_reserve1 - amount1_out) {
        amount1_in = balance1 - (_reserve1 - amount1_out);
    }

    if !(amount0_in > U256::zero() || amount1_in > U256::zero()) {
        runtime::revert(Error::InsufficientInputAmount);
    }

    let balance0_adjusted: U256 = balance0 * U256::from(1_000u64) - amount0_in * U256::from(2u64);
    let balance1_adjusted: U256 = balance1 * U256::from(1_000u64) - amount1_in * U256::from(2u64);

    if !((balance0_adjusted * balance1_adjusted)
        >= (_reserve0 * _reserve1 * U256::from(1_000_000u64)))
    {
        runtime::revert(Error::K);
    }

    SwapperyPair::default()._update(balance0, balance1);

    SwapperyPair::default().write_locked(false);
}

#[no_mangle]
fn call() {
    const CONTRACT_KEY_NAME_ARG_NAME: &str = "contract_key_name";

    let name: String = runtime::get_named_arg(consts::NAME_RUNTIME_ARG_NAME);
    let symbol: String = runtime::get_named_arg(consts::SYMBOL_RUNTIME_ARG_NAME);
    let decimals: u8 = runtime::get_named_arg(consts::DECIMALS_RUNTIME_ARG_NAME);
    let initial_supply: U256 = runtime::get_named_arg(consts::TOTAL_SUPPLY_RUNTIME_ARG_NAME);
    let contract_key_name: String = runtime::get_named_arg(CONTRACT_KEY_NAME_ARG_NAME);
    let token0: ContractHash = runtime::get_named_arg(consts::TOKEN0_KEY_NAME);
    let token1: ContractHash = runtime::get_named_arg(consts::TOKEN1_KEY_NAME);

    let _ = SwapperyPair::create(
        name,
        symbol,
        decimals,
        initial_supply,
        contract_key_name.as_str(),
        token0,
        token1,
    )
    .unwrap_or_revert();
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}