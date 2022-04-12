use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use casper_engine_test_support::{Code, SessionBuilder, TestContext, TestContextBuilder};
use casper_erc20::constants as consts;
use casper_erc20::Address;
use casper_types::{
    account::AccountHash,
    contracts::ContractPackageHash,
    bytesrepr::{FromBytes, ToBytes},
    runtime_args, AsymmetricType, CLTyped, ContractHash, Key, PublicKey, RuntimeArgs, U256, U512,
};

const CONTRACT_SWAPPERY_PAIR: &str = "swappery_pair.wasm";
const CONTRACT_ERC20_TOKEN: &str = "erc20_token.wasm";
const PAIR_CONTRACT_KEY_NAME: &str = "test_swappery_pair";
const PAIR_CONTRACT_HASH_KEY_NAME: &str = "test_swappery_pair_contract_hash";

const ERC20_CONTRACT_KEY_NAME: &str = "erc20_token_contract";
const ERC20_CONTRACT_HASH_KEY_NAME: &str = "erc20_token_contract_contract_hash";

pub const RESERVE0_KEY_NAME: &str = "reserve0";
pub const RESERVE1_KEY_NAME: &str = "reserve1";
pub const TOKEN0_KEY_NAME: &str = "token0";
pub const TOKEN1_KEY_NAME: &str = "token1";
pub const KLAST_KEY_NAME: &str = "klast";
pub const FACTORY_KEY_NAME: &str = "factory";
pub const LOCKED_FLAG_KEY_NAME: &str = "locked";
pub const MINT_ENTRY_POINT_NAME: &str = "mint";
pub const BURN_ENTRY_POINT_NAME: &str = "burn";
pub const SWAP_ENTRY_POINT_NAME: &str = "swap";
pub const GET_RESERVES_ENTRY_POINT_NAME: &str = "get_reserves";

fn blake2b256(item_key_string: &[u8]) -> Box<[u8]> {
    let mut hasher = VarBlake2b::new(32).unwrap();
    hasher.update(item_key_string);
    hasher.finalize_boxed()
}

#[derive(Clone, Copy)]
pub struct Sender(pub AccountHash);

pub struct TestFixture {
    context: TestContext,
    pub ali: AccountHash,
    pub bob: AccountHash,
    pub joe: AccountHash,
}

impl TestFixture {
    pub const TOKEN_NAME: &'static str = "test_pair_token";
    pub const TOKEN_SYMBOL: &'static str = "TSP";
    pub const TOKEN_DECIMALS: u8 = 8;
    pub const PAIR_TOKEN0_NAME: &'static str = "token0_for_pair";
    pub const PAIR_TOKEN0_SYMBOL: &'static str = "T0P";
    pub const PAIR_TOKEN1_NAME: &'static str = "token1_for_pair";
    pub const PAIR_TOKEN1_SYMBOL: &'static str = "T1P";
    const TOKEN_TOTAL_SUPPLY_AS_U64: u64 = 10_000;

    pub fn token_total_supply() -> U256 {
        Self::TOKEN_TOTAL_SUPPLY_AS_U64.into()
    }

    pub fn install_contract() -> TestFixture {
        let ali = PublicKey::ed25519_from_bytes([3u8; 32]).unwrap();
        let bob = PublicKey::ed25519_from_bytes([6u8; 32]).unwrap();
        let joe = PublicKey::ed25519_from_bytes([9u8; 32]).unwrap();

        let mut context = TestContextBuilder::new()
            .with_public_key(ali.clone(), U512::from(500_000_000_000_000_000u64))
            .with_public_key(bob.clone(), U512::from(500_000_000_000_000_000u64))
            .build();

        let session_code = Code::from(CONTRACT_ERC20_TOKEN);
        let session_args = runtime_args! {
            consts::NAME_RUNTIME_ARG_NAME => TestFixture::PAIR_TOKEN0_NAME,
            consts::SYMBOL_RUNTIME_ARG_NAME => TestFixture::PAIR_TOKEN0_SYMBOL,
            consts::DECIMALS_RUNTIME_ARG_NAME => TestFixture::TOKEN_DECIMALS,
            consts::TOTAL_SUPPLY_RUNTIME_ARG_NAME => TestFixture::token_total_supply(),
        };

        let session = SessionBuilder::new(session_code, session_args)
            .with_address(ali.to_account_hash())
            .with_authorization_keys(&[ali.to_account_hash()])
            .build();

        context.run(session);

        let session_code = Code::from(CONTRACT_ERC20_TOKEN);
        let session_args = runtime_args! {
            consts::NAME_RUNTIME_ARG_NAME => TestFixture::PAIR_TOKEN1_NAME,
            consts::SYMBOL_RUNTIME_ARG_NAME => TestFixture::PAIR_TOKEN1_SYMBOL,
            consts::DECIMALS_RUNTIME_ARG_NAME => TestFixture::TOKEN_DECIMALS,
            consts::TOTAL_SUPPLY_RUNTIME_ARG_NAME => TestFixture::token_total_supply(),
        };

        let session = SessionBuilder::new(session_code, session_args)
            .with_address(bob.to_account_hash())
            .with_authorization_keys(&[bob.to_account_hash()])
            .build();

        context.run(session);

        let token0: ContractPackageHash = context.get_account(ali.to_account_hash()).unwrap().named_keys()
            .get(ERC20_CONTRACT_KEY_NAME).unwrap().normalize().into_hash().unwrap().into();
        let token1: ContractPackageHash = context.get_account(bob.to_account_hash()).unwrap().named_keys()
            .get(ERC20_CONTRACT_KEY_NAME).unwrap().normalize().into_hash().unwrap().into();

        let session_code = Code::from(CONTRACT_SWAPPERY_PAIR);
        let session_args = runtime_args! {
            consts::NAME_RUNTIME_ARG_NAME => TestFixture::TOKEN_NAME,
            consts::SYMBOL_RUNTIME_ARG_NAME => TestFixture::TOKEN_SYMBOL,
            consts::DECIMALS_RUNTIME_ARG_NAME => TestFixture::TOKEN_DECIMALS,
            consts::TOTAL_SUPPLY_RUNTIME_ARG_NAME => U256::zero(),
            "contract_key_name" => PAIR_CONTRACT_KEY_NAME,
            TOKEN0_KEY_NAME => Address::from(token0),
            TOKEN1_KEY_NAME => Address::from(token1)
        };

        let session = SessionBuilder::new(session_code, session_args)
            .with_address(ali.to_account_hash())
            .with_authorization_keys(&[ali.to_account_hash()])
            .build();

        context.run(session);

        TestFixture {
            context,
            ali: ali.to_account_hash(),
            bob: bob.to_account_hash(),
            joe: joe.to_account_hash(),
        }
    }

    fn pair_contract_hash(&self) -> ContractHash {
        self.context
            .get_account(self.ali)
            .unwrap()
            .named_keys()
            .get(PAIR_CONTRACT_HASH_KEY_NAME)
            .unwrap()
            .normalize()
            .into_hash()
            .unwrap()
            .into()
    }

    pub fn pair_contract_package_hash(&self) -> ContractPackageHash {
        self.context
            .get_account(self.ali)
            .unwrap()
            .named_keys()
            .get(PAIR_CONTRACT_KEY_NAME)
            .unwrap()
            .normalize()
            .into_hash()
            .unwrap()
            .into()
    }

    fn query_pair_contract<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        match self
            .context
            .query(self.ali, &[PAIR_CONTRACT_HASH_KEY_NAME.to_string(), name.to_string()])
        {
            Err(_) => None,
            Ok(maybe_value) => {
                let value = maybe_value
                    .into_t()
                    .unwrap_or_else(|_| panic!("{} is not expected type.", name));
                Some(value)
            }
        }
    }

    fn token0_contract_hash(&self) -> ContractHash {
        self.context
            .get_account(self.ali)
            .unwrap()
            .named_keys()
            .get(ERC20_CONTRACT_HASH_KEY_NAME)
            .unwrap()
            .normalize()
            .into_hash()
            .unwrap()
            .into()
    }

    pub fn token0_contract_package_hash(&self) -> ContractPackageHash {
        self.context
            .get_account(self.ali)
            .unwrap()
            .named_keys()
            .get(ERC20_CONTRACT_KEY_NAME)
            .unwrap()
            .normalize()
            .into_hash()
            .unwrap()
            .into()
    }

    fn query_token0_contract<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        match self
            .context
            .query(self.ali, &[ERC20_CONTRACT_HASH_KEY_NAME.to_string(), name.to_string()])
        {
            Err(_) => None,
            Ok(maybe_value) => {
                let value = maybe_value
                    .into_t()
                    .unwrap_or_else(|_| panic!("{} is not expected type.", name));
                Some(value)
            }
        }
    }

    fn token1_contract_hash(&self) -> ContractHash {
        self.context
            .get_account(self.bob)
            .unwrap()
            .named_keys()
            .get(ERC20_CONTRACT_HASH_KEY_NAME)
            .unwrap()
            .normalize()
            .into_hash()
            .unwrap()
            .into()
    }

    pub fn token1_contract_package_hash(&self) -> ContractPackageHash {
        self.context
            .get_account(self.bob)
            .unwrap()
            .named_keys()
            .get(ERC20_CONTRACT_KEY_NAME)
            .unwrap()
            .normalize()
            .into_hash()
            .unwrap()
            .into()
    }

    fn query_token1_contract<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        match self
            .context
            .query(self.bob, &[ERC20_CONTRACT_HASH_KEY_NAME.to_string(), name.to_string()])
        {
            Err(_) => None,
            Ok(maybe_value) => {
                let value = maybe_value
                    .into_t()
                    .unwrap_or_else(|_| panic!("{} is not expected type.", name));
                Some(value)
            }
        }
    }

    fn pair_call(&mut  self, sender: Sender, method: &str, args: RuntimeArgs) {
        let Sender(address) = sender;
        let code = Code::Hash(self.pair_contract_hash().value(), method.to_string());
        let session = SessionBuilder::new(code, args)
            .with_address(address)
            .with_authorization_keys(&[address])
            .build();
        self.context.run(session);
    }

    pub fn pair_token_name(&self) -> String {
        self.query_pair_contract(consts::NAME_RUNTIME_ARG_NAME)
            .unwrap()
    }

    pub fn pair_token_symbol(&self) -> String {
        self.query_pair_contract(consts::SYMBOL_RUNTIME_ARG_NAME)
            .unwrap()
    }

    pub fn pair_token_decimals(&self) -> u8 {
        self.query_pair_contract(consts::DECIMALS_RUNTIME_ARG_NAME)
            .unwrap()
    }

    pub fn pair_token0(&self) -> Address {
        self.query_pair_contract(TOKEN0_KEY_NAME).unwrap()
    }

    pub fn pair_token1(&self) -> Address {
        self.query_pair_contract(TOKEN1_KEY_NAME).unwrap()
    }

    pub fn pair_factory(&self) -> Address {
        self.query_pair_contract(FACTORY_KEY_NAME).unwrap()
    }

    pub fn pair_reserve0(&self) -> U256 {
        self.query_pair_contract(RESERVE0_KEY_NAME).unwrap()
    }

    pub fn pair_reserve1(&self) -> U256 {
        self.query_pair_contract(RESERVE1_KEY_NAME).unwrap()
    }

    pub fn pair_klast(&self) -> U256 {
        self.query_pair_contract(KLAST_KEY_NAME).unwrap()
    }

    pub fn pair_balance_of(&self, account: Key) -> Option<U256> {
        let item_key = base64::encode(&account.to_bytes().unwrap());

        let key = Key::Hash(self.pair_contract_hash().value());
        let value = self
            .context
            .query_dictionary_item(key, Some(consts::BALANCES_KEY_NAME.to_string()), item_key)
            .ok()?;

        Some(value.into_t::<U256>().unwrap())
    }

    pub fn pair_allowance(&self, owner: Key, spender: Key) -> Option<U256> {
        let mut preimage = Vec::new();
        preimage.append(&mut owner.to_bytes().unwrap());
        preimage.append(&mut spender.to_bytes().unwrap());
        let key_bytes = blake2b256(&preimage);
        let allowance_item_key = hex::encode(&key_bytes);

        let key = Key::Hash(self.pair_contract_hash().value());

        let value = self
            .context
            .query_dictionary_item(
                key,
                Some(consts::ALLOWANCES_KEY_NAME.to_string()),
                allowance_item_key,
            )
            .ok()?;

        Some(value.into_t::<U256>().unwrap())
    }

    pub fn pair_transfer(&mut self, recipient: Key, amount: U256, sender: Sender) {
        self.pair_call(
            sender,
            consts::TRANSFER_ENTRY_POINT_NAME,
            runtime_args! {
                consts::RECIPIENT_RUNTIME_ARG_NAME => recipient,
                consts::AMOUNT_RUNTIME_ARG_NAME => amount
            },
        );
    }

    pub fn pair_approve(&mut self, spender: Key, amount: U256, sender: Sender) {
        self.pair_call(
            sender,
            consts::APPROVE_ENTRY_POINT_NAME,
            runtime_args! {
                consts::SPENDER_RUNTIME_ARG_NAME => spender,
                consts::AMOUNT_RUNTIME_ARG_NAME => amount
            },
        );
    }

    pub fn pair_transfer_from(&mut self, owner: Key, recipient: Key, amount: U256, sender: Sender) {
        self.pair_call(
            sender,
            consts::TRANSFER_FROM_ENTRY_POINT_NAME,
            runtime_args! {
                consts::OWNER_RUNTIME_ARG_NAME => owner,
                consts::RECIPIENT_RUNTIME_ARG_NAME => recipient,
                consts::AMOUNT_RUNTIME_ARG_NAME => amount
            },
        );
    }

    pub fn pair_mint(&mut self, to: Key, sender: Sender) {
        self.pair_call(
            sender,
            MINT_ENTRY_POINT_NAME,
            runtime_args! {
                "to" => to
            },
        );
    }

    pub fn pair_burn(&mut self, to: Key, sender: Sender) {
        self.pair_call(
            sender,
            BURN_ENTRY_POINT_NAME,
            runtime_args! {
                "to" => to
            },
        );
    }

    pub fn pair_swap(&mut self, amount0: U256, amount1: U256, to: Key, sender: Sender) {
        self.pair_call(
            sender,
            SWAP_ENTRY_POINT_NAME,
            runtime_args! {
                "amount0" => amount0,
                "amount1" => amount1,
                "to" => to
            },
        );
    }

    //token0

    fn token0_call(&mut self, sender: Sender, method: &str, args: RuntimeArgs) {
        let Sender(address) = sender;
        let code = Code::Hash(self.token0_contract_hash().value(), method.to_string());
        let session = SessionBuilder::new(code, args)
            .with_address(address)
            .with_authorization_keys(&[address])
            .build();
        self.context.run(session);
    }

    pub fn token0_token_name(&self) -> String {
        self.query_token0_contract(consts::NAME_RUNTIME_ARG_NAME)
            .unwrap()
    }

    pub fn token0_token_symbol(&self) -> String {
        self.query_token0_contract(consts::SYMBOL_RUNTIME_ARG_NAME)
            .unwrap()
    }

    pub fn token0_token_decimals(&self) -> u8 {
        self.query_token0_contract(consts::DECIMALS_RUNTIME_ARG_NAME)
            .unwrap()
    }

    pub fn token0_balance_of(&self, account: Key) -> Option<U256> {
        let item_key = base64::encode(&account.to_bytes().unwrap());

        let key = Key::Hash(self.token0_contract_hash().value());
        let value = self
            .context
            .query_dictionary_item(key, Some(consts::BALANCES_KEY_NAME.to_string()), item_key)
            .ok()?;

        Some(value.into_t::<U256>().unwrap())
    }

    pub fn token0_allowance(&self, owner: Key, spender: Key) -> Option<U256> {
        let mut preimage = Vec::new();
        preimage.append(&mut owner.to_bytes().unwrap());
        preimage.append(&mut spender.to_bytes().unwrap());
        let key_bytes = blake2b256(&preimage);
        let allowance_item_key = hex::encode(&key_bytes);

        let key = Key::Hash(self.token0_contract_hash().value());

        let value = self
            .context
            .query_dictionary_item(
                key,
                Some(consts::ALLOWANCES_KEY_NAME.to_string()),
                allowance_item_key,
            )
            .ok()?;

        Some(value.into_t::<U256>().unwrap())
    }

    pub fn token0_transfer(&mut self, recipient: Key, amount: U256, sender: Sender) {
        self.token0_call(
            sender,
            consts::TRANSFER_ENTRY_POINT_NAME,
            runtime_args! {
                consts::RECIPIENT_RUNTIME_ARG_NAME => recipient,
                consts::AMOUNT_RUNTIME_ARG_NAME => amount
            },
        );
    }

    pub fn token0_approve(&mut self, spender: Key, amount: U256, sender: Sender) {
        self.token0_call(
            sender,
            consts::APPROVE_ENTRY_POINT_NAME,
            runtime_args! {
                consts::SPENDER_RUNTIME_ARG_NAME => spender,
                consts::AMOUNT_RUNTIME_ARG_NAME => amount
            },
        );
    }

    pub fn token0_transfer_from(&mut self, owner: Key, recipient: Key, amount: U256, sender: Sender) {
        self.token0_call(
            sender,
            consts::TRANSFER_FROM_ENTRY_POINT_NAME,
            runtime_args! {
                consts::OWNER_RUNTIME_ARG_NAME => owner,
                consts::RECIPIENT_RUNTIME_ARG_NAME => recipient,
                consts::AMOUNT_RUNTIME_ARG_NAME => amount
            },
        );
    }

    //token1

    fn token1_call(&mut self, sender: Sender, method: &str, args: RuntimeArgs) {
        let Sender(address) = sender;
        let code = Code::Hash(self.token1_contract_hash().value(), method.to_string());
        let session = SessionBuilder::new(code, args)
            .with_address(address)
            .with_authorization_keys(&[address])
            .build();
        self.context.run(session);
    }

    pub fn token1_token_name(&self) -> String {
        self.query_token1_contract(consts::NAME_RUNTIME_ARG_NAME)
            .unwrap()
    }

    pub fn token1_token_symbol(&self) -> String {
        self.query_token1_contract(consts::SYMBOL_RUNTIME_ARG_NAME)
            .unwrap()
    }

    pub fn token1_token_decimals(&self) -> u8 {
        self.query_token1_contract(consts::DECIMALS_RUNTIME_ARG_NAME)
            .unwrap()
    }

    pub fn token1_balance_of(&self, account: Key) -> Option<U256> {
        let item_key = base64::encode(&account.to_bytes().unwrap());

        let key = Key::Hash(self.token1_contract_hash().value());
        let value = self
            .context
            .query_dictionary_item(key, Some(consts::BALANCES_KEY_NAME.to_string()), item_key)
            .ok()?;

        Some(value.into_t::<U256>().unwrap())
    }

    pub fn token1_allowance(&self, owner: Key, spender: Key) -> Option<U256> {
        let mut preimage = Vec::new();
        preimage.append(&mut owner.to_bytes().unwrap());
        preimage.append(&mut spender.to_bytes().unwrap());
        let key_bytes = blake2b256(&preimage);
        let allowance_item_key = hex::encode(&key_bytes);

        let key = Key::Hash(self.token1_contract_hash().value());

        let value = self
            .context
            .query_dictionary_item(
                key,
                Some(consts::ALLOWANCES_KEY_NAME.to_string()),
                allowance_item_key,
            )
            .ok()?;

        Some(value.into_t::<U256>().unwrap())
    }

    pub fn token1_transfer(&mut self, recipient: Key, amount: U256, sender: Sender) {
        self.token1_call(
            sender,
            consts::TRANSFER_ENTRY_POINT_NAME,
            runtime_args! {
                consts::RECIPIENT_RUNTIME_ARG_NAME => recipient,
                consts::AMOUNT_RUNTIME_ARG_NAME => amount
            },
        );
    }

    pub fn token1_approve(&mut self, spender: Key, amount: U256, sender: Sender) {
        self.token1_call(
            sender,
            consts::APPROVE_ENTRY_POINT_NAME,
            runtime_args! {
                consts::SPENDER_RUNTIME_ARG_NAME => spender,
                consts::AMOUNT_RUNTIME_ARG_NAME => amount
            },
        );
    }

    pub fn token1_transfer_from(&mut self, owner: Key, recipient: Key, amount: U256, sender: Sender) {
        self.token1_call(
            sender,
            consts::TRANSFER_FROM_ENTRY_POINT_NAME,
            runtime_args! {
                consts::OWNER_RUNTIME_ARG_NAME => owner,
                consts::RECIPIENT_RUNTIME_ARG_NAME => recipient,
                consts::AMOUNT_RUNTIME_ARG_NAME => amount
            },
        );
    }
}
