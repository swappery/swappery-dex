use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST,
    DEFAULT_ACCOUNT_ADDR, MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_execution_engine::core::{
    engine_state::{Error as CoreError, ExecuteRequest},
    execution::Error as ExecError,
};
use casper_types::{
    account::AccountHash, runtime_args, system::mint,
    ContractHash, ContractPackageHash, Key, RuntimeArgs, U256,
    ApiError,
};
use crate::constants as consts;
use crate::test_call::{make_erc20_transfer_request, erc20_check_balance_of};

#[derive(Copy, Clone)]
struct TestContext {
    token0_contract: ContractHash,
    token1_contract: ContractHash,
    pair_package: ContractPackageHash,
    pair_contract: ContractHash,
}

fn setup() -> (InMemoryWasmTestBuilder, TestContext) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&*DEFAULT_RUN_GENESIS_REQUEST);

    let id: Option<u64> = None;
    let transfer_args = runtime_args! {
        mint::ARG_TARGET => *consts::ACCOUNT_1_ADDR,
        mint::ARG_AMOUNT => MINIMUM_ACCOUNT_CREATION_BALANCE,
        mint::ARG_ID => id,
    };

    let transfer_request =
        ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_args).build();

    let install_request_token0 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        consts::CONTRACT_ERC20_TOKEN,
        runtime_args! {
            consts::ARG_NAME => consts::TOKEN0_NAME,
            consts::ARG_SYMBOL => consts::TOKEN0_SYMBOL,
            consts::ARG_DECIMALS => consts::TOKEN0_DECIMALS,
            consts::ARG_TOTAL_SUPPLY => U256::from(consts::TOKEN0_TOTAL_SUPPLY),
            consts::ARG_CONTRACT_KEY_NAME => consts::TOKEN0_CONTRACT_KEY_NAME,
        },
    )
    .build();

    let install_request_token1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        consts::CONTRACT_ERC20_TOKEN,
        runtime_args! {
            consts::ARG_NAME => consts::TOKEN1_NAME,
            consts::ARG_SYMBOL => consts::TOKEN1_SYMBOL,
            consts::ARG_DECIMALS => consts::TOKEN1_DECIMALS,
            consts::ARG_TOTAL_SUPPLY => U256::from(consts::TOKEN1_TOTAL_SUPPLY),
            consts::ARG_CONTRACT_KEY_NAME => consts::TOKEN1_CONTRACT_KEY_NAME,
        },
    )
    .build();

    let install_request_test_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        consts::CONTRACT_ERC20_TEST_CALL,
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

    let token0_contract = account
        .named_keys()
        .get(consts::TOKEN0_CONTRACT_HASH_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let token1_contract = account
        .named_keys()
        .get(consts::TOKEN1_CONTRACT_HASH_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let tokens = 
    if token0_contract.lt(&token1_contract) { (token0_contract, token1_contract) }
    else { (token1_contract, token0_contract) };

    let install_request_pair = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        consts::CONTRACT_SWAPPERY_PAIR,
        runtime_args! {
            consts::ARG_NAME => consts::PAIR_NAME,
            consts::ARG_SYMBOL => consts::PAIR_SYMBOL,
            consts::ARG_DECIMALS => consts::PAIR_DECIMALS,
            consts::ARG_TOTAL_SUPPLY => U256::from(consts::PAIR_TOTAL_SUPPLY),
            consts::ARG_CONTRACT_KEY_NAME => consts::PAIR_CONTRACT_KEY_NAME,
            consts::ARG_TOKEN0 => ContractHash::from(tokens.0),
            consts::ARG_TOKEN1 => ContractHash::from(tokens.1),
        },
    )
    .build();

    builder.exec(install_request_pair).expect_success().commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let pair_package = account
        .named_keys()
        .get(consts::PAIR_CONTRACT_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have contract package hash");

    let pair_contract = account
        .named_keys()
        .get(consts::PAIR_CONTRACT_HASH_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let test_context = TestContext {
        token0_contract: tokens.0,
        token1_contract: tokens.1,
        pair_package,
        pair_contract,
    };

    (builder, test_context)
}

#[test]
fn should_mint_and_burn_lp_token() {
    let (mut builder, TestContext { token0_contract, token1_contract, pair_package, pair_contract, .. }) = setup();
        
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    let pair_key = Key::from(pair_package);
    
    // let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    // assert_eq!(owner_balance, U256::from(consts::TOKEN0_TOTAL_SUPPLY));
    // let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    // assert_eq!(owner_balance, U256::from(consts::TOKEN1_TOTAL_SUPPLY));

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(consts::TOKEN0_TOTAL_SUPPLY));
    builder.exec(token0_transfer_request).expect_success().commit();
    
    let pair_balance = erc20_check_balance_of(&mut builder, &token0_contract, pair_key);
    assert_eq!(pair_balance, U256::from(consts::TOKEN0_TOTAL_SUPPLY));

    let token1_transfer_request = make_erc20_transfer_request(owner_key, &token1_contract, pair_key, U256::from(consts::TOKEN1_TOTAL_SUPPLY));
    builder.exec(token1_transfer_request).expect_success().commit();
        
    let pair_balance = erc20_check_balance_of(&mut builder, &token1_contract, pair_key);
    assert_eq!(pair_balance, U256::from(consts::TOKEN1_TOTAL_SUPPLY));

    let pair_mint_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        consts::METHOD_MINT,
        runtime_args!{
            consts::ARG_TO => owner_key,
            consts::ARG_FEETO => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        },
    ).build();
    builder.exec(pair_mint_request).expect_success().commit();

    let owner_balance = erc20_check_balance_of(&mut builder, &pair_contract, owner_key);
    
    let pair_transfer_request = make_erc20_transfer_request(owner_key, &pair_contract, pair_key, owner_balance);
    builder.exec(pair_transfer_request).expect_success().commit();

    let owner_balance = erc20_check_balance_of(&mut builder, &pair_contract, owner_key);
    assert_eq!(owner_balance, U256::zero());

    let pair_burn_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        consts::METHOD_BURN,
        runtime_args!{
            consts::ARG_TO => owner_key,
            consts::ARG_FEETO => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        },
    ).build();
    builder.exec(pair_burn_request).expect_success().commit();

    let pair_balance = erc20_check_balance_of(&mut builder, &pair_contract, pair_key);
    assert_eq!(pair_balance, U256::zero());

    // let pair_balance = erc20_check_balance_of(&mut builder, &token0_contract, pair_key);
    // let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    // assert_eq!(pair_balance + owner_balance, U256::from(consts::TOKEN0_TOTAL_SUPPLY));

    // let pair_balance = erc20_check_balance_of(&mut builder, &token1_contract, pair_key);
    // let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    // assert_eq!(pair_balance + owner_balance, U256::from(consts::TOKEN1_TOTAL_SUPPLY));
}

#[test]
fn should_swap_tokens_with_pair() {
    let (mut builder, TestContext { token0_contract, token1_contract, pair_package, pair_contract, .. }) = setup();
        
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    let pair_key = Key::from(pair_package);
    
    // let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    // assert_eq!(owner_balance, U256::from(consts::TOKEN0_TOTAL_SUPPLY));
    // let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    // assert_eq!(owner_balance, U256::from(consts::TOKEN1_TOTAL_SUPPLY));

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(100_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let token1_transfer_request = make_erc20_transfer_request(owner_key, &token1_contract, pair_key, U256::from(400_000u64));
    builder.exec(token1_transfer_request).expect_success().commit();

    let pair_mint_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        consts::METHOD_MINT,
        runtime_args!{
            consts::ARG_TO => owner_key,
            consts::ARG_FEETO => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        },
    ).build();
    builder.exec(pair_mint_request).expect_success().commit();

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(50_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let swap_token_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        consts::METHOD_SWAP,
        runtime_args!{
            consts::ARG_AMOUNT0 => U256::zero(),
            consts::ARG_AMOUNT1 => U256::from(100_000u64),
            consts::ARG_TO => owner_key,
        },
    ).build();
    builder.exec(swap_token_request).expect_success().commit();

    let pair_balance = erc20_check_balance_of(&mut builder, &pair_contract, pair_key);
    assert_eq!(pair_balance, U256::zero());

    let pair_balance = erc20_check_balance_of(&mut builder, &token0_contract, pair_key);
    let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    assert_eq!(pair_balance, U256::from(150_000u64));
    // assert_eq!(pair_balance + owner_balance, U256::from(consts::TOKEN0_TOTAL_SUPPLY));

    let pair_balance = erc20_check_balance_of(&mut builder, &token1_contract, pair_key);
    let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    assert_eq!(pair_balance, U256::from(300_000u64));
    // assert_eq!(pair_balance + owner_balance, U256::from(consts::TOKEN1_TOTAL_SUPPLY));
}

#[test]
fn should_not_swap_tokens_above_reserves() {
    let (mut builder, TestContext { token0_contract, token1_contract, pair_package, .. }) = setup();
        
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    let pair_key = Key::from(pair_package);
    
    let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    // assert_eq!(owner_balance, U256::from(consts::TOKEN0_TOTAL_SUPPLY));
    let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    // assert_eq!(owner_balance, U256::from(consts::TOKEN1_TOTAL_SUPPLY));

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(100_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let token1_transfer_request = make_erc20_transfer_request(owner_key, &token1_contract, pair_key, U256::from(400_000u64));
    builder.exec(token1_transfer_request).expect_success().commit();

    let pair_mint_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        consts::METHOD_MINT,
        runtime_args!{
            consts::ARG_TO => owner_key,
            consts::ARG_FEETO => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        },
    ).build();
    builder.exec(pair_mint_request).expect_success().commit();

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(50_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let swap_token_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        consts::METHOD_SWAP,
        runtime_args!{
            consts::ARG_AMOUNT0 => U256::zero(),
            consts::ARG_AMOUNT1 => U256::from(400_000u64),
            consts::ARG_TO => owner_key,
        },
    ).build();
    builder.exec(swap_token_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == consts::ERROR_INSUFFICIENT_LIQUIDITY),
        "{:?}",
        error
    );
}

#[test]
fn should_not_swap_over_limits() {
    let (mut builder, TestContext { token0_contract, token1_contract, pair_package, .. }) = setup();
        
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    let pair_key = Key::from(pair_package);
    
    let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    assert_eq!(owner_balance, U256::from(consts::TOKEN0_TOTAL_SUPPLY));
    let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    assert_eq!(owner_balance, U256::from(consts::TOKEN1_TOTAL_SUPPLY));

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(100_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let token1_transfer_request = make_erc20_transfer_request(owner_key, &token1_contract, pair_key, U256::from(400_000u64));
    builder.exec(token1_transfer_request).expect_success().commit();

    let pair_mint_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        consts::METHOD_MINT,
        runtime_args!{
            consts::ARG_TO => owner_key,
            consts::ARG_FEETO => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        },
    ).build();
    builder.exec(pair_mint_request).expect_success().commit();

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(50_000u64));
    builder.exec(token0_transfer_request).expect_success().commit();

    let swap_token_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        consts::METHOD_SWAP,
        runtime_args!{
            consts::ARG_AMOUNT0 => U256::zero(),
            consts::ARG_AMOUNT1 => U256::from(200_000u64),
            consts::ARG_TO => owner_key,
        },
    ).build();
    builder.exec(swap_token_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == consts::ERROR_K),
        "{:?}",
        error
    );
}

#[test]
fn should_mint_minimum_liquidity_to_zero_address() {
    let (mut builder, TestContext { token0_contract, token1_contract, pair_package, pair_contract, .. }) = setup();
        
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    let zero_key = Key::from(AccountHash::new([0u8; 32]));
    let pair_key = Key::from(pair_package);
    
    let owner_balance = erc20_check_balance_of(&mut builder, &token0_contract, owner_key);
    assert_eq!(owner_balance, U256::from(consts::TOKEN0_TOTAL_SUPPLY));
    let owner_balance = erc20_check_balance_of(&mut builder, &token1_contract, owner_key);
    assert_eq!(owner_balance, U256::from(consts::TOKEN1_TOTAL_SUPPLY));

    let token0_transfer_request = make_erc20_transfer_request(owner_key, &token0_contract, pair_key, U256::from(consts::TOKEN0_TOTAL_SUPPLY));
    builder.exec(token0_transfer_request).expect_success().commit();
    
    let pair_balance = erc20_check_balance_of(&mut builder, &token0_contract, pair_key);
    assert_eq!(pair_balance, U256::from(consts::TOKEN0_TOTAL_SUPPLY));

    let token1_transfer_request = make_erc20_transfer_request(owner_key, &token1_contract, pair_key, U256::from(consts::TOKEN1_TOTAL_SUPPLY));
    builder.exec(token1_transfer_request).expect_success().commit();
        
    let pair_balance = erc20_check_balance_of(&mut builder, &token1_contract, pair_key);
    assert_eq!(pair_balance, U256::from(consts::TOKEN1_TOTAL_SUPPLY));

    let pair_mint_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        pair_package,
        None,
        consts::METHOD_MINT,
        runtime_args!{
            consts::ARG_TO => owner_key,
            consts::ARG_FEETO => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        },
    ).build();
    builder.exec(pair_mint_request).expect_success().commit();

    let owner_balance = erc20_check_balance_of(&mut builder, &pair_contract, zero_key);
    assert_eq!(owner_balance, U256::from(1_000u64));
}