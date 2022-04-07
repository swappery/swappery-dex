#![no_std]
#![no_main]

extern crate alloc;

mod entry_points;
mod constants;
mod helpers;
mod variables;
mod error;

use alloc::string::String;

use once_cell::unsync::OnceCell;

use core::ops::{Deref, DerefMut};

use casper_erc20::{
    constants::{
        ADDRESS_RUNTIME_ARG_NAME, AMOUNT_RUNTIME_ARG_NAME,
        OWNER_RUNTIME_ARG_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        SPENDER_RUNTIME_ARG_NAME, BALANCE_OF_ENTRY_POINT_NAME,
        TRANSFER_ENTRY_POINT_NAME,
    },
    Address, ERC20
};

use casper_types::{CLValue, U256, URef, contracts::NamedKeys, Key, runtime_args, account::AccountHash};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use constants::{
    RESERVE0_KEY_NAME, RESERVE1_KEY_NAME, TOKEN0_KEY_NAME, TOKEN1_KEY_NAME,
    LOCKED_FLAG_KEY_NAME, TO_RUNTIME_ARG_NAME, AMOUNT0_RUNTIME_ARG_NAME, AMOUNT1_RUNTIME_ARG_NAME,
    KLAST_KEY_NAME, FACTORY_KEY_NAME, MINIMUM_LIQUIDITY,
};
use error::Error;

#[derive(Default)]
pub struct SwapperyPair {
    erc20: ERC20,
    reserve0_uref: OnceCell<URef>,
    reserve1_uref: OnceCell<URef>,
    locked_uref: OnceCell<URef>,
    klast_uref: OnceCell<URef>,
}

impl SwapperyPair {
    fn new(erc20: ERC20, reserve0_uref: URef, reserve1_uref: URef, locked_uref: URef, klast_uref: URef) -> Self {
        Self {
            erc20: erc20,
            reserve0_uref: reserve0_uref.into(),
            reserve1_uref: reserve1_uref.into(),
            locked_uref: locked_uref.into(),
            klast_uref: klast_uref.into(),
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

    fn klast_uref(&self) -> URef {
        *self
            .klast_uref
            .get_or_init(variables::klast_uref)
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

    pub fn token0(&self) -> Address {
        helpers::read_from(TOKEN0_KEY_NAME)
    }

    pub fn token1(&self) -> Address {
        helpers::read_from(TOKEN1_KEY_NAME)
    }

    pub fn factory(&self) -> Address {
        helpers::read_from(FACTORY_KEY_NAME)
    }

    pub fn klast(&self) -> U256 {
        helpers::read_from(KLAST_KEY_NAME)
    }

    pub fn _update(
        &mut self,
        balance0: U256,
        balance1: U256,
    ) {
        if !(balance0 <= U256::MAX || balance1 <= U256::MAX) {
            runtime::revert(Error::OverFlow);
        }
        self.write_reserve0(balance0);
        self.write_reserve1(balance1);
    }

    pub fn _mintFee(
        &mut self,
        _reserve0: U256,
        _reserve1: U256
    ) -> bool {
        // address feeTo = IPancakeFactory(factory).feeTo();
        // feeOn = feeTo != address(0);
        let _kLast: U256 = self.klast(); // gas savings
        if true {
            if !(_kLast.is_zero()) {
                let mut rootK: U256 = _reserve0 * _reserve1;
                rootK = rootK.integer_sqrt();
                let rootKLast = _kLast.integer_sqrt();
                if rootK > rootKLast {
                    let numerator: U256 = U256::from(SwapperyPair::default().total_supply()) * (rootK - rootKLast);
                    let denominator: U256 = rootK * U256::from("3") + rootKLast;
                    let liquidity: U256 = numerator / denominator;
                    if liquidity > U256::zero() {
                        //ERC20::default().mint(feeTo, liquidity).unwrap_or_revert();
                    }
                }
            }
        } else if !(_kLast.is_zero()) {
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
        token0: Address, 
        token1: Address, 
        factory: Address)
     -> Result<SwapperyPair, Error> {
        let reserve0_uref = storage::new_uref(U256::zero()).into_read_write();
        let reserve1_uref = storage::new_uref(U256::zero()).into_read_write();
        let locked_uref = storage::new_uref(false).into_read_write();
        let klast_uref = storage::new_uref(U256::zero()).into_read_write();

        let token0_key = {
            let token0_uref = storage::new_uref(token0).into_read();
            Key::from(token0_uref)
        };

        let token1_key = {
            let token1_uref = storage::new_uref(token1).into_read();
            Key::from(token1_uref)
        };

        let factory_key = {
            let factory_uref = storage::new_uref(factory).into_read();
            Key::from(factory_uref)
        };

        let mut named_keys = NamedKeys::new();

        named_keys.insert(String::from(RESERVE0_KEY_NAME), Key::from(reserve0_uref));
        named_keys.insert(String::from(RESERVE1_KEY_NAME), Key::from(reserve1_uref));
        named_keys.insert(String::from(TOKEN0_KEY_NAME), token0_key);
        named_keys.insert(String::from(TOKEN1_KEY_NAME), token1_key);
        named_keys.insert(String::from(LOCKED_FLAG_KEY_NAME), Key::from(locked_uref));
        named_keys.insert(String::from(KLAST_KEY_NAME), Key::from(klast_uref));
        named_keys.insert(String::from(FACTORY_KEY_NAME), factory_key);
        let erc20 = ERC20::install_custom(
            name,
            symbol,
            decimals,
            initial_supply,
            contract_key_name,
            named_keys,
            entry_points::default(),
        ).unwrap_or_revert();
        Ok(SwapperyPair::new(erc20, reserve0_uref, reserve1_uref, locked_uref, klast_uref ))
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
    let address: Address = runtime::get_named_arg(ADDRESS_RUNTIME_ARG_NAME);
    let balance = SwapperyPair::default().balance_of(address);
    runtime::ret(CLValue::from_t(balance).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer() {
    let recipient: Address = runtime::get_named_arg(RECIPIENT_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);

    SwapperyPair::default()
        .transfer(recipient, amount)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn approve() {
    let spender: Address = runtime::get_named_arg(SPENDER_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);

    SwapperyPair::default().approve(spender, amount).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn allowance() {
    let owner: Address = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    let spender: Address = runtime::get_named_arg(SPENDER_RUNTIME_ARG_NAME);
    let val = SwapperyPair::default().allowance(owner, spender);
    runtime::ret(CLValue::from_t(val).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer_from() {
    let owner: Address = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    let recipient: Address = runtime::get_named_arg(RECIPIENT_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);
    SwapperyPair::default()
        .transfer_from(owner, recipient, amount)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn get_reserves() {
    let reserves: (U256, U256) = (SwapperyPair::default().reserve0(), SwapperyPair::default().reserve1());
    runtime::ret(CLValue::from_t(reserves).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn mint() {
    let locked = SwapperyPair::default().locked();
    if locked {
        runtime::revert(Error::Locked);
    }
    SwapperyPair::default().write_locked(true);

    let to: Address = runtime::get_named_arg(TO_RUNTIME_ARG_NAME);

    let _reserve0: U256 = SwapperyPair::default().reserve0();
    let _reserve1: U256 = SwapperyPair::default().reserve1();

    let token0: Address = SwapperyPair::default().token0();
    let token1: Address = SwapperyPair::default().token1();

    let self_addr = helpers::get_self_address().unwrap_or_revert();

    let balance0: U256 = runtime::call_versioned_contract(
        *token0.as_contract_package_hash().unwrap_or_revert(),
        None,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args!{
            ADDRESS_RUNTIME_ARG_NAME => self_addr
        }
    );
    let balance1: U256 = runtime::call_versioned_contract(
        *token1.as_contract_package_hash().unwrap_or_revert(),
        None,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args!{
            ADDRESS_RUNTIME_ARG_NAME => self_addr
        }
    );
    let amount0: U256 = balance0 - _reserve0;
    let amount1: U256 = balance1 - _reserve1;

    let feeOn: bool = SwapperyPair::default()._mintFee(_reserve0, _reserve1);
    let _totalSupply: U256 = SwapperyPair::default().total_supply();
    let mut liquidity: U256;
    if _totalSupply.is_zero() {
        liquidity = (U256::from(amount0 * amount1).integer_sqrt()) - MINIMUM_LIQUIDITY;
        SwapperyPair::default().mint(Address::from(AccountHash::new([0u8; 32])), MINIMUM_LIQUIDITY);
    } else {
        liquidity = U256::min(
            amount0 * _totalSupply / _reserve0,
            amount1 * _totalSupply / _reserve1
        );
    }
    if !(liquidity > U256::zero()) {
        runtime::revert(Error::InsufficientLiquidityMinted);
    }
    SwapperyPair::default().mint(to, liquidity);

    SwapperyPair::default()._update(balance0, balance1);
    if feeOn {
        SwapperyPair::default().write_klast( 
            SwapperyPair::default().reserve0() * 
            SwapperyPair::default().reserve1() );
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

    let to: Address = runtime::get_named_arg(TO_RUNTIME_ARG_NAME);

    let _reserve0: U256 = SwapperyPair::default().reserve0();
    let _reserve1: U256 = SwapperyPair::default().reserve1();

    let token0: Address = SwapperyPair::default().token0();
    let token1: Address = SwapperyPair::default().token1();

    let self_addr = helpers::get_self_address().unwrap_or_revert();

    let mut balance0: U256 = runtime::call_versioned_contract(
        *token0.as_contract_package_hash().unwrap_or_revert(),
        None,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args!{
            ADDRESS_RUNTIME_ARG_NAME => self_addr
        }
    );
    let mut balance1: U256 = runtime::call_versioned_contract(
        *token1.as_contract_package_hash().unwrap_or_revert(),
        None,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args!{
            ADDRESS_RUNTIME_ARG_NAME => self_addr
        }
    );
    let liquidity: U256 = SwapperyPair::default().balance_of(self_addr);

    let feeOn: bool = SwapperyPair::default()._mintFee(_reserve0, _reserve1);
    let _totalSupply: U256 = SwapperyPair::default().total_supply(); 
    let amount0: U256 = liquidity * balance0 / _totalSupply;
    let amount1: U256 = liquidity * balance1 / _totalSupply;
    if !(amount0 > U256::zero() && amount1 > U256::zero()) {
        runtime::revert(Error::InsufficientLiquidityBurned);
    }

    SwapperyPair::default().burn(self_addr, liquidity);
    runtime::call_versioned_contract(
        *token0.as_contract_package_hash().unwrap_or_revert(),
        None,
        TRANSFER_ENTRY_POINT_NAME,
        runtime_args!{
            RECIPIENT_RUNTIME_ARG_NAME => to,
            AMOUNT_RUNTIME_ARG_NAME => amount0
        }
    );
    runtime::call_versioned_contract(
        *token1.as_contract_package_hash().unwrap_or_revert(),
        None,
        TRANSFER_ENTRY_POINT_NAME,
        runtime_args!{
            RECIPIENT_RUNTIME_ARG_NAME => to,
            AMOUNT_RUNTIME_ARG_NAME => amount1
        }
    );

    balance0 = runtime::call_versioned_contract(
        *token0.as_contract_package_hash().unwrap_or_revert(),
        None,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args!{
            ADDRESS_RUNTIME_ARG_NAME => self_addr
        }
    );
    balance1 = runtime::call_versioned_contract(
        *token1.as_contract_package_hash().unwrap_or_revert(),
        None,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args!{
            ADDRESS_RUNTIME_ARG_NAME => self_addr
        }
    );

    SwapperyPair::default()._update(balance0, balance1);
    if feeOn {
        SwapperyPair::default().write_klast( 
            SwapperyPair::default().reserve0() * 
            SwapperyPair::default().reserve1() );
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

    let amount0Out: U256 = runtime::get_named_arg(AMOUNT0_RUNTIME_ARG_NAME);
    let amount1Out: U256 = runtime::get_named_arg(AMOUNT1_RUNTIME_ARG_NAME);
    let to: Address = runtime::get_named_arg(TO_RUNTIME_ARG_NAME);

    if !(amount0Out > U256::zero() || amount1Out > U256::zero()) {
        runtime::revert(Error::InsufficientOutputAmount);
    }

    let _reserve0: U256 = SwapperyPair::default().reserve0();
    let _reserve1: U256 = SwapperyPair::default().reserve1();

    if !(amount0Out < _reserve0 && amount1Out < _reserve1) {
        runtime::revert(Error::InsufficientLiquidity);
    }

    let mut balance0: U256;
    let mut balance1: U256;

    let self_addr = helpers::get_self_address().unwrap_or_revert();

    let token0: Address = SwapperyPair::default().token0();
    let token1: Address = SwapperyPair::default().token1();
    
    if !(to != token0 && to != token1) {
        runtime::revert(Error::InvalidTo);
    }

    if amount0Out > U256::zero() {
        runtime::call_versioned_contract(
            *token0.as_contract_package_hash().unwrap_or_revert(),
            None,
            TRANSFER_ENTRY_POINT_NAME,
            runtime_args!{
                RECIPIENT_RUNTIME_ARG_NAME => to,
                AMOUNT_RUNTIME_ARG_NAME => amount0Out
            }
        );
    }
    if amount1Out > U256::zero() {
        runtime::call_versioned_contract(
            *token1.as_contract_package_hash().unwrap_or_revert(),
            None,
            TRANSFER_ENTRY_POINT_NAME,
            runtime_args!{
                RECIPIENT_RUNTIME_ARG_NAME => to,
                AMOUNT_RUNTIME_ARG_NAME => amount1Out
            }
        );
    }

    //     IPancakeCallee(to).pancakeCall(
    
    balance0 = runtime::call_versioned_contract(
        *token0.as_contract_package_hash().unwrap_or_revert(),
        None,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args!{
            ADDRESS_RUNTIME_ARG_NAME => self_addr
        }
    );
    balance1 = runtime::call_versioned_contract(
        *token1.as_contract_package_hash().unwrap_or_revert(),
        None,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args!{
            ADDRESS_RUNTIME_ARG_NAME => self_addr
        }
    );

    let mut amount0In: U256 = U256::zero();
    if balance0 > ( _reserve0 - amount0Out ){
        amount0In = balance0 - (_reserve0 - amount0Out);
    }

    let mut amount1In: U256 = U256::zero();
    if balance1 > ( _reserve1 - amount1Out ){
        amount1In = balance1 - (_reserve1 - amount1Out);
    }

    if !(amount0In > U256::zero() || amount1In > U256::zero()) {
        runtime::revert(Error::InsufficientInputAmount);
    }

    let balance0Adjusted: U256 = balance0 * U256::from("1000") - amount0In * U256::from("2");
    let balance1Adjusted: U256 = balance1 * U256::from("1000") - amount1In * U256::from("2");
    
    if !((balance0Adjusted * balance1Adjusted) >= (_reserve0 * _reserve1 * U256::from("1000000"))) {
        runtime::revert(Error::K);
    }

    SwapperyPair::default()._update(balance0, balance1);

    SwapperyPair::default().write_locked(false);
}