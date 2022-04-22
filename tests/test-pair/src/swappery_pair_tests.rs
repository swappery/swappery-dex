use once_cell::sync::Lazy;

use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST,
    DEFAULT_ACCOUNT_ADDR, MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_execution_engine::core::{
    engine_state::{Error as CoreError, ExecuteRequest},
    execution::Error as ExecError,
};
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, system::mint, CLTyped,
    ContractHash, ContractPackageHash, Key, PublicKey, RuntimeArgs, SecretKey, U256,
    ApiError,
};

const EXAMPLE_ERC20_TOKEN: &str = "erc20_token.wasm";
const SWAPPERY_PAIR: &str = "swappery_pair.wasm";
const CONTRACT_ERC20_TEST_CALL: &str = "erc20_test_call.wasm";
const NAME_KEY: &str = "name";
const SYMBOL_KEY: &str = "symbol";
const DECIMALS_KEY: &str = "decimals";
const TOTAL_SUPPLY_KEY: &str = "total_supply";
const BALANCES_KEY: &str = "balances";
const ALLOWANCES_KEY: &str = "allowances";

const ARG_NAME: &str = "name";
const ARG_SYMBOL: &str = "symbol";
const ARG_DECIMALS: &str = "decimals";
const ARG_TOTAL_SUPPLY: &str = "total_supply";
const ARG_CONTRACT_KEY_NAME: &str = "contract_key_name";
const ARG_TOKEN0: &str = "token0";
const ARG_TOKEN1: &str = "token1";
const ARG_TO: &str = "to";
const ARG_AMOUNT0: &str = "amount0";
const ARG_AMOUNT1: &str = "amount1";

const PAIR_NAME: &str = "SwapperyPair";
const PAIR_SYMBOL: &str = "SWP";
const PAIR_DECIMALS: u8 = 8;
const PAIR_TOTAL_SUPPLY: u64 = 0;
const PAIR_CONTRACT_KEY_NAME: &str = "swappery_pair";
const PAIR_CONTRACT_HASH_KEY_NAME: &str = "swappery_pair_contract_hash";

const TOKEN0_NAME: &str = "PairTestToken0";
const TOKEN0_SYMBOL: &str = "PTT0";
const TOKEN0_DECIMALS: u8 = 8;
const TOKEN0_TOTAL_SUPPLY: u64 = 1_000_000;
const TOKEN0_CONTRACT_KEY_NAME: &str = "token0";
const TOKEN0_CONTRACT_HASH_KEY_NAME: &str = "token0_contract_hash";

const TOKEN1_NAME: &str = "PairTestToken1";
const TOKEN1_SYMBOL: &str = "PTT1";
const TOKEN1_DECIMALS: u8 = 8;
const TOKEN1_TOTAL_SUPPLY: u64 = 2_000_000;
const TOKEN1_CONTRACT_KEY_NAME: &str = "token1";
const TOKEN1_CONTRACT_HASH_KEY_NAME: &str = "token1_contract_hash";

const METHOD_TRANSFER: &str = "transfer";
const ARG_AMOUNT: &str = "amount";
const ARG_RECIPIENT: &str = "recipient";

const METHOD_APPROVE: &str = "approve";
const ARG_OWNER: &str = "owner";
const ARG_SPENDER: &str = "spender";

const METHOD_TRANSFER_FROM: &str = "transfer_from";

static ACCOUNT_1_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes(&[221u8; 32]).unwrap());
static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_1_SECRET_KEY));
static ACCOUNT_1_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_1_PUBLIC_KEY.to_account_hash());

static ACCOUNT_2_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes(&[212u8; 32]).unwrap());
static ACCOUNT_2_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_2_SECRET_KEY));
static ACCOUNT_2_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_2_PUBLIC_KEY.to_account_hash());

const TOKEN_OWNER_ADDRESS_1: Key = Key::Account(AccountHash::new([42; 32]));
const TOKEN_OWNER_AMOUNT_1: u64 = 1_000_000;
const TOKEN_OWNER_ADDRESS_2: Key = Key::Hash([42; 32]);
const TOKEN_OWNER_AMOUNT_2: u64 = 2_000_000;

const METHOD_MINT: &str = "mint";
const METHOD_BURN: &str = "burn";
const METHOD_SWAP: &str = "swap";

const CHECK_TOTAL_SUPPLY_ENTRYPOINT: &str = "check_total_supply";
const CHECK_BALANCE_OF_ENTRYPOINT: &str = "check_balance_of";
const CHECK_ALLOWANCE_OF_ENTRYPOINT: &str = "check_allowance_of";
const ARG_TOKEN_CONTRACT: &str = "token_contract";
const ARG_ADDRESS: &str = "address";
const RESULT_KEY: &str = "result";
const ERC20_TEST_CALL_KEY: &str = "erc20_test_call";

const METHOD_TRANSFER_AS_STORED_CONTRACT: &str = "transfer_as_stored_contract";
const METHOD_APPROVE_AS_STORED_CONTRACT: &str = "approve_as_stored_contract";
const METHOD_FROM_AS_STORED_CONTRACT: &str = "transfer_from_as_stored_contract";


const ERROR_INSUFFICIENT_LIQUIDITY: u16 = u16::MAX - 2;
const ERROR_K: u16 = u16::MAX - 9;

/// Converts hash addr of Account into Hash, and Hash into Account
///
/// This is useful for making sure ERC20 library respects different variants of Key when storing
/// balances.
fn invert_erc20_address(address: Key) -> Key {
    match address {
        Key::Account(account_hash) => Key::Hash(account_hash.value()),
        Key::Hash(contract_hash) => Key::Account(AccountHash::new(contract_hash)),
        _ => panic!("Unsupported Key variant"),
    }
}

#[derive(Copy, Clone)]
struct TestContext {
    token0_package: ContractPackageHash,
    token0_contract: ContractHash,
    token1_package: ContractPackageHash,
    token1_contract: ContractHash,
    pair_package: ContractPackageHash,
    pair_contract: ContractHash,
    erc20_test_call: ContractPackageHash,
}

fn setup() -> (InMemoryWasmTestBuilder, TestContext) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&*DEFAULT_RUN_GENESIS_REQUEST);

    let id: Option<u64> = None;
    let transfer_args = runtime_args! {
        mint::ARG_TARGET => *ACCOUNT_1_ADDR,
        mint::ARG_AMOUNT => MINIMUM_ACCOUNT_CREATION_BALANCE,
        mint::ARG_ID => id,
    };

    let transfer_request =
        ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_args).build();

    let install_request_token0 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        EXAMPLE_ERC20_TOKEN,
        runtime_args! {
            ARG_NAME => TOKEN0_NAME,
            ARG_SYMBOL => TOKEN0_SYMBOL,
            ARG_DECIMALS => TOKEN0_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN0_TOTAL_SUPPLY),
            ARG_CONTRACT_KEY_NAME => TOKEN0_CONTRACT_KEY_NAME,
        },
    )
    .build();

    let install_request_token1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        EXAMPLE_ERC20_TOKEN,
        runtime_args! {
            ARG_NAME => TOKEN1_NAME,
            ARG_SYMBOL => TOKEN1_SYMBOL,
            ARG_DECIMALS => TOKEN1_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN1_TOTAL_SUPPLY),
            ARG_CONTRACT_KEY_NAME => TOKEN1_CONTRACT_KEY_NAME,
        },
    )
    .build();

    let install_request_test_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_ERC20_TEST_CALL,
        RuntimeArgs::default(),
    )
    .build();

    builder.exec(transfer_request).expect_success().commit();
    builder.exec(install_request_token0).expect_success().commit();
    builder.exec(install_request_token1).expect_success().commit();
    builder.exec(install_request_test_call).expect_success().commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let token0_package = account
        .named_keys()
        .get(TOKEN0_CONTRACT_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have contract package hash");

    let token0_contract = account
        .named_keys()
        .get(TOKEN0_CONTRACT_HASH_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let token1_package = account
        .named_keys()
        .get(TOKEN1_CONTRACT_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have contract package hash");

    let token1_contract = account
        .named_keys()
        .get(TOKEN1_CONTRACT_HASH_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");
        
    let erc20_test_call = account
        .named_keys()
        .get(ERC20_TEST_CALL_KEY)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have contract hash");

    let install_request_pair = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        SWAPPERY_PAIR,
        runtime_args! {
            ARG_NAME => PAIR_NAME,
            ARG_SYMBOL => PAIR_SYMBOL,
            ARG_DECIMALS => PAIR_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(PAIR_TOTAL_SUPPLY),
            ARG_CONTRACT_KEY_NAME => PAIR_CONTRACT_KEY_NAME,
            ARG_TOKEN0 => ContractHash::from(token0_contract),
            ARG_TOKEN1 => ContractHash::from(token1_contract),
        },
    )
    .build();

    builder.exec(install_request_pair).expect_success().commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let pair_package = account
        .named_keys()
        .get(PAIR_CONTRACT_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have contract package hash");

    let pair_contract = account
        .named_keys()
        .get(PAIR_CONTRACT_HASH_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let test_context = TestContext {
        token0_package,
        token0_contract,
        token1_package,
        token1_contract,
        pair_package,
        pair_contract,
        erc20_test_call,
    };

    (builder, test_context)
}

fn erc20_check_total_supply(
    builder: &mut InMemoryWasmTestBuilder,
    erc20_contract_hash: &ContractHash,
) -> U256 {
    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let erc20_test_contract_hash = account
        .named_keys()
        .get(ERC20_TEST_CALL_KEY)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have test contract hash");

    let check_total_supply_args = runtime_args! {
        ARG_TOKEN_CONTRACT => *erc20_contract_hash,
    };

    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        erc20_test_contract_hash,
        None,
        CHECK_TOTAL_SUPPLY_ENTRYPOINT,
        check_total_supply_args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, erc20_test_contract_hash)
}

fn get_test_result<T: FromBytes + CLTyped>(
    builder: &mut InMemoryWasmTestBuilder,
    erc20_test_contract_hash: ContractPackageHash,
) -> T {
    let contract_package = builder
        .get_contract_package(erc20_test_contract_hash)
        .expect("should have contract package");
    let enabled_versions = contract_package.enabled_versions();
    let (_version, contract_hash) = enabled_versions
        .iter()
        .rev()
        .next()
        .expect("should have latest version");

    builder.get_value(*contract_hash, RESULT_KEY)
}

fn erc20_check_balance_of(
    builder: &mut InMemoryWasmTestBuilder,
    erc20_contract_hash: &ContractHash,
    address: Key,
) -> U256 {
    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let erc20_test_contract_hash = account
        .named_keys()
        .get(ERC20_TEST_CALL_KEY)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have test contract hash");

    let check_balance_args = runtime_args! {
        ARG_TOKEN_CONTRACT => *erc20_contract_hash,
        ARG_ADDRESS => address,
    };
    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        erc20_test_contract_hash,
        None,
        CHECK_BALANCE_OF_ENTRYPOINT,
        check_balance_args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, erc20_test_contract_hash)
}

fn erc20_check_allowance_of(
    builder: &mut InMemoryWasmTestBuilder,
    owner: Key,
    spender: Key,
) -> U256 {
    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");
    let erc20_contract_hash = account
        .named_keys()
        .get(TOKEN0_CONTRACT_HASH_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have test contract hash");
    let erc20_test_contract_hash = account
        .named_keys()
        .get(ERC20_TEST_CALL_KEY)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have test contract hash");

    let check_balance_args = runtime_args! {
        ARG_TOKEN_CONTRACT => erc20_contract_hash,
        ARG_OWNER => owner,
        ARG_SPENDER => spender,
    };
    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        erc20_test_contract_hash,
        None,
        CHECK_ALLOWANCE_OF_ENTRYPOINT,
        check_balance_args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, erc20_test_contract_hash)
}

fn make_erc20_transfer_request(
    sender: Key,
    erc20_token: &ContractHash,
    recipient: Key,
    amount: U256,
) -> ExecuteRequest {
    match sender {
        Key::Account(sender) => ExecuteRequestBuilder::contract_call_by_hash(
            sender,
            *erc20_token,
            METHOD_TRANSFER,
            runtime_args! {
                ARG_AMOUNT => amount,
                ARG_RECIPIENT => recipient,
            },
        )
        .build(),
        Key::Hash(contract_package_hash) => ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            ContractPackageHash::new(contract_package_hash),
            None,
            METHOD_TRANSFER_AS_STORED_CONTRACT,
            runtime_args! {
                ARG_TOKEN_CONTRACT => *erc20_token,
                ARG_AMOUNT => amount,
                ARG_RECIPIENT => recipient,
            },
        )
        .build(),
        _ => panic!("Unknown variant"),
    }
}

fn make_erc20_approve_request(
    sender: Key,
    erc20_token: &ContractHash,
    spender: Key,
    amount: U256,
) -> ExecuteRequest {
    match sender {
        Key::Account(sender) => ExecuteRequestBuilder::contract_call_by_hash(
            sender,
            *erc20_token,
            METHOD_APPROVE,
            runtime_args! {
                ARG_SPENDER => spender,
                ARG_AMOUNT => amount,
            },
        )
        .build(),
        Key::Hash(contract_hash) => ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            ContractPackageHash::new(contract_hash),
            None,
            METHOD_APPROVE_AS_STORED_CONTRACT,
            runtime_args! {
                ARG_TOKEN_CONTRACT => *erc20_token,
                ARG_SPENDER => spender,
                ARG_AMOUNT => amount,
            },
        )
        .build(),
        _ => panic!("Unknown variant"),
    }
}

#[test]
fn should_mint_and_burn_lp_token() {
    let (mut builder, TestContext { token0_package, token0_contract, token1_package, token1_contract, pair_package, pair_contract, .. }) = setup();
        
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    let pair_key = Key::from(pair_package);
    
    let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    assert_eq!(owner_balance, U256::from(TOKEN0_TOTAL_SUPPLY));
    let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    assert_eq!(owner_balance, U256::from(TOKEN1_TOTAL_SUPPLY));

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(TOKEN0_TOTAL_SUPPLY));
    builder.exec(token0_transfer_request).expect_success().commit();
    
    let pair_balance = erc20_check_balance_of(&mut builder, &token0_contract, pair_key);
    assert_eq!(pair_balance, U256::from(TOKEN0_TOTAL_SUPPLY));

    let token1_transfer_request = make_erc20_transfer_request(owner_key, &token1_contract, pair_key, U256::from(TOKEN1_TOTAL_SUPPLY));
    builder.exec(token1_transfer_request).expect_success().commit();
        
    let pair_balance = erc20_check_balance_of(&mut builder, &token1_contract, pair_key);
    assert_eq!(pair_balance, U256::from(TOKEN1_TOTAL_SUPPLY));

    let pair_mint_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        METHOD_MINT,
        runtime_args!{
            ARG_TO => owner_key,
        },
    ).build();
    builder.exec(pair_mint_request).expect_success().commit();

    let owner_balance = erc20_check_balance_of(&mut builder, &pair_contract, owner_key);
    // assert_eq!(owner_balance, U256::zero());

    let pair_transfer_request = make_erc20_transfer_request(owner_key, &pair_contract, pair_key, owner_balance);
    builder.exec(pair_transfer_request).expect_success().commit();

    let owner_balance = erc20_check_balance_of(&mut builder, &pair_contract, owner_key);
    assert_eq!(owner_balance, U256::zero());

    let pair_burn_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        METHOD_BURN,
        runtime_args!{
            ARG_TO => owner_key,
        },
    ).build();
    builder.exec(pair_burn_request).expect_success().commit();

    let pair_balance = erc20_check_balance_of(&mut builder, &pair_contract, pair_key);
    assert_eq!(pair_balance, U256::zero());

    let pair_balance = erc20_check_balance_of(&mut builder, &token0_contract, pair_key);
    let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    assert_eq!(pair_balance + owner_balance, U256::from(TOKEN0_TOTAL_SUPPLY));

    let pair_balance = erc20_check_balance_of(&mut builder, &token1_contract, pair_key);
    let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    assert_eq!(pair_balance + owner_balance, U256::from(TOKEN1_TOTAL_SUPPLY));
}

#[test]
fn should_swap_tokens_with_pair() {
    let (mut builder, TestContext { token0_package, token0_contract, token1_package, token1_contract, pair_package, pair_contract, .. }) = setup();
        
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    let pair_key = Key::from(pair_package);
    
    let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    assert_eq!(owner_balance, U256::from(TOKEN0_TOTAL_SUPPLY));
    let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    assert_eq!(owner_balance, U256::from(TOKEN1_TOTAL_SUPPLY));

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(100_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let token1_transfer_request = make_erc20_transfer_request(owner_key, &token1_contract, pair_key, U256::from(400_000u64));
    builder.exec(token1_transfer_request).expect_success().commit();

    let pair_mint_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        METHOD_MINT,
        runtime_args!{
            ARG_TO => owner_key,
        },
    ).build();
    builder.exec(pair_mint_request).expect_success().commit();

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(50_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let swap_token_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        METHOD_SWAP,
        runtime_args!{
            ARG_AMOUNT0 => U256::zero(),
            ARG_AMOUNT1 => U256::from(100_000u64),
            ARG_TO => owner_key,
        },
    ).build();
    builder.exec(swap_token_request).expect_success().commit();

    let pair_balance = erc20_check_balance_of(&mut builder, &pair_contract, pair_key);
    assert_eq!(pair_balance, U256::zero());

    let pair_balance = erc20_check_balance_of(&mut builder, &token0_contract, pair_key);
    let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    assert_eq!(pair_balance, U256::from(150_000u64));
    assert_eq!(pair_balance + owner_balance, U256::from(TOKEN0_TOTAL_SUPPLY));

    let pair_balance = erc20_check_balance_of(&mut builder, &token1_contract, pair_key);
    let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    assert_eq!(pair_balance, U256::from(300_000u64));
    assert_eq!(pair_balance + owner_balance, U256::from(TOKEN1_TOTAL_SUPPLY));
}

#[test]
fn should_not_swap_tokens_above_reserves() {
    let (mut builder, TestContext { token0_package, token0_contract, token1_package, token1_contract, pair_package, pair_contract, .. }) = setup();
        
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    let pair_key = Key::from(pair_package);
    
    let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    assert_eq!(owner_balance, U256::from(TOKEN0_TOTAL_SUPPLY));
    let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    assert_eq!(owner_balance, U256::from(TOKEN1_TOTAL_SUPPLY));

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(100_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let token1_transfer_request = make_erc20_transfer_request(owner_key, &token1_contract, pair_key, U256::from(400_000u64));
    builder.exec(token1_transfer_request).expect_success().commit();

    let pair_mint_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        METHOD_MINT,
        runtime_args!{
            ARG_TO => owner_key,
        },
    ).build();
    builder.exec(pair_mint_request).expect_success().commit();

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(50_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let swap_token_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        METHOD_SWAP,
        runtime_args!{
            ARG_AMOUNT0 => U256::zero(),
            ARG_AMOUNT1 => U256::from(400_000u64),
            ARG_TO => owner_key,
        },
    ).build();
    builder.exec(swap_token_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == ERROR_INSUFFICIENT_LIQUIDITY),
        "{:?}",
        error
    );
}

#[test]
fn should_not_swap_over_limits() {
    let (mut builder, TestContext { token0_package, token0_contract, token1_package, token1_contract, pair_package, pair_contract, .. }) = setup();
        
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    let pair_key = Key::from(pair_package);
    
    let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    assert_eq!(owner_balance, U256::from(TOKEN0_TOTAL_SUPPLY));
    let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    assert_eq!(owner_balance, U256::from(TOKEN1_TOTAL_SUPPLY));

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(100_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let token1_transfer_request = make_erc20_transfer_request(owner_key, &token1_contract, pair_key, U256::from(400_000u64));
    builder.exec(token1_transfer_request).expect_success().commit();

    let pair_mint_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        METHOD_MINT,
        runtime_args!{
            ARG_TO => owner_key,
        },
    ).build();
    builder.exec(pair_mint_request).expect_success().commit();

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(50_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let swap_token_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        METHOD_SWAP,
        runtime_args!{
            ARG_AMOUNT0 => U256::zero(),
            ARG_AMOUNT1 => U256::from(200_000u64),
            ARG_TO => owner_key,
        },
    ).build();
    builder.exec(swap_token_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == ERROR_K),
        "{:?}",
        error
    );
}

#[test]
fn should_mint_minimum_liquidity_to_zero_address() {
    let (mut builder, TestContext { token0_package, token0_contract, token1_package, token1_contract, pair_package, pair_contract, .. }) = setup();
        
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    let zero_key = Key::from(AccountHash::new([0u8; 32]));
    let pair_key = Key::from(pair_package);
    
    let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    assert_eq!(owner_balance, U256::from(TOKEN0_TOTAL_SUPPLY));
    let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    assert_eq!(owner_balance, U256::from(TOKEN1_TOTAL_SUPPLY));

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(TOKEN0_TOTAL_SUPPLY));
    builder.exec(token0_transfer_request).expect_success().commit();
    
    let pair_balance = erc20_check_balance_of(&mut builder, &token0_contract, pair_key);
    assert_eq!(pair_balance, U256::from(TOKEN0_TOTAL_SUPPLY));

    let token1_transfer_request = make_erc20_transfer_request(owner_key, &token1_contract, pair_key, U256::from(TOKEN1_TOTAL_SUPPLY));
    builder.exec(token1_transfer_request).expect_success().commit();
        
    let pair_balance = erc20_check_balance_of(&mut builder, &token1_contract, pair_key);
    assert_eq!(pair_balance, U256::from(TOKEN1_TOTAL_SUPPLY));

    let pair_mint_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        METHOD_MINT,
        runtime_args!{
            ARG_TO => owner_key,
        },
    ).build();
    builder.exec(pair_mint_request).expect_success().commit();

    let owner_balance = erc20_check_balance_of(&mut builder, &pair_contract, zero_key);
    assert_eq!(owner_balance, U256::from(1_000u64));
}