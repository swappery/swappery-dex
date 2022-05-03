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
use casper_erc20::Address;
use crate::constants as consts;
use crate::test_call::{make_erc20_transfer_request, erc20_check_allowance_of, erc20_check_balance_of};

#[derive(Copy, Clone)]
struct TestContext {
    token0_contract: ContractHash,
    token1_contract: ContractHash,
    wcspr_contract: ContractHash,
    pair_0_1_package: ContractPackageHash,
    pair_0_1_contract: ContractHash,
    router_package: ContractPackageHash,
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

    let install_request_wcspr_token = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        consts::CONTRACT_WCSPR_TOKEN,
        runtime_args! {
            consts::ARG_NAME => consts::WCSPR_NAME,
            consts::ARG_SYMBOL => consts::WCSPR_SYMBOL,
            consts::ARG_DECIMALS => consts::WCSPR_DECIMALS,
            consts::ARG_TOTAL_SUPPLY => U256::from(consts::WCSPR_TOTAL_SUPPLY),
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
    builder.exec(install_request_wcspr_token).expect_success().commit();

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

    let wcspr_contract = account
        .named_keys()
        .get(consts::WCSPR_CONTRACT_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let install_request_router = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        consts::CONTRACT_SWAPPERY_ROUTER,
        runtime_args! {
            consts::FEETO_KEY_NAME => Address::from(AccountHash::new([10u8; 32])),
            consts::FEETO_SETTER_KEY_NAME => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            consts::WCSPR_CONTRACT_KEY_NAME => wcspr_contract,
            consts::ARG_CONTRACT_KEY_NAME => consts::ROUTER_CONTRACT_KEY_NAME,
        }
    )
    .build();

    let install_request_pair_0_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        consts::CONTRACT_SWAPPERY_PAIR,
        runtime_args! {
            consts::ARG_NAME => consts::PAIR_NAME,
            consts::ARG_SYMBOL => consts::PAIR_SYMBOL,
            consts::ARG_DECIMALS => consts::PAIR_DECIMALS,
            consts::ARG_TOTAL_SUPPLY => U256::from(consts::PAIR_TOTAL_SUPPLY),
            consts::ARG_CONTRACT_KEY_NAME => consts::PAIR_CONTRACT_KEY_NAME,
            consts::ARG_TOKEN0 => Key::from(token0_contract),
            consts::ARG_TOKEN1 => Key::from(token1_contract),
        },
    )
    .build();

    builder.exec(install_request_pair_0_1).expect_success().commit();
    builder.exec(install_request_router).expect_success().commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let router_package = account
        .named_keys()
        .get(consts::ROUTER_CONTRACT_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have contract package hash");

    let pair_0_1_package = account
        .named_keys()
        .get(consts::PAIR_CONTRACT_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have contract package hash");

    let pair_0_1_contract = account
        .named_keys()
        .get(consts::PAIR_CONTRACT_HASH_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let create_pair_0_1_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        router_package,
        None,
        consts::METHOD_CREATE_PAIR,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(token0_contract),
            consts::ARG_TOKEN1 => Key::from(token1_contract),
            consts::ARG_PAIR => Address::from(pair_0_1_package),
        },
    )
    .build();

    builder.exec(create_pair_0_1_request).expect_success().commit();

    let test_context = TestContext {
        token0_contract,
        token1_contract,
        wcspr_contract,
        pair_0_1_package,
        pair_0_1_contract,
        router_package,
    };

    (builder, test_context)
}

#[test]
fn should_setup_context() {
    let _ = setup();
}

#[test]
fn should_add_liquidity() {
    let (mut builder, test_context) = setup();
    
    let token0_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token0_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    let token1_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token1_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    builder.exec(token0_transfer_request).expect_success().commit();
    builder.exec(token1_transfer_request).expect_success().commit();

    let token0_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token0_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(100_000u64),
        }
    )
    .build();

    let token1_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(100_000u64),
        }
    )
    .build();

    builder.exec(token0_approve_request).expect_success().commit();
    builder.exec(token1_approve_request).expect_success().commit();

    let router_allowance: U256 = erc20_check_allowance_of(&mut builder, Key::Account(*consts::ACCOUNT_1_ADDR), Key::Hash(test_context.router_package.value()));
    assert_eq!(router_allowance, U256::from(100_000u64));
    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(50_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(add_liquidity_request).expect_success().commit();
    let lp_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.pair_0_1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(lp_balance, U256::from(37_729u64));
}

#[test]
fn should_not_mint_liquidity_with_amount_less_than_minimum() {
    let (mut builder, test_context) = setup();
    
    let token0_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token0_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    let token1_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token1_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    builder.exec(token0_transfer_request).expect_success().commit();
    builder.exec(token1_transfer_request).expect_success().commit();

    let token0_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token0_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    let token1_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    builder.exec(token0_approve_request).expect_success().commit();
    builder.exec(token1_approve_request).expect_success().commit();

    let router_allowance: U256 = erc20_check_allowance_of(&mut builder, Key::Account(*consts::ACCOUNT_1_ADDR), Key::Hash(test_context.router_package.value()));
    assert_eq!(router_allowance, U256::from(50_000u64));
    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(20_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(add_liquidity_request).expect_success().commit();

    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(20_000u64),
            consts::ARG_AMOUNT0_MIN => U256::from(50_000u64),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(add_liquidity_request).expect_failure().commit();
}

#[test]
fn should_remove_liquidity() {
    let (mut builder, test_context) = setup();
    
    let token0_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token0_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    let token1_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token1_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    builder.exec(token0_transfer_request).expect_success().commit();
    builder.exec(token1_transfer_request).expect_success().commit();

    let token0_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token0_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    let token1_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    builder.exec(token0_approve_request).expect_success().commit();
    builder.exec(token1_approve_request).expect_success().commit();

    let router_allowance: U256 = erc20_check_allowance_of(&mut builder, Key::Account(*consts::ACCOUNT_1_ADDR), Key::Hash(test_context.router_package.value()));
    assert_eq!(router_allowance, U256::from(50_000u64));
    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(50_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(add_liquidity_request).expect_success().commit();
    let lp_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.pair_0_1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(lp_balance, U256::from(37_729u64));

    let lp_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.pair_0_1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(30_000u64),
        }
    )
    .build();

    builder.exec(lp_approve_request).expect_success().commit();

    let remove_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_REMOVE_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_LIQUIDITY => U256::from(30_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(remove_liquidity_request).expect_success().commit();

    let lp_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.pair_0_1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(lp_balance, U256::from(7_729u64));

    let token0_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.token0_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(token0_balance, U256::from(93_238u64));
}

#[test]
fn should_not_remove_liquidity_over_balance() {
    let (mut builder, test_context) = setup();
    
    let token0_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token0_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    let token1_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token1_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    builder.exec(token0_transfer_request).expect_success().commit();
    builder.exec(token1_transfer_request).expect_success().commit();

    let token0_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token0_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    let token1_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    builder.exec(token0_approve_request).expect_success().commit();
    builder.exec(token1_approve_request).expect_success().commit();

    let router_allowance: U256 = erc20_check_allowance_of(&mut builder, Key::Account(*consts::ACCOUNT_1_ADDR), Key::Hash(test_context.router_package.value()));
    assert_eq!(router_allowance, U256::from(50_000u64));
    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(50_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(add_liquidity_request).expect_success().commit();
    let lp_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.pair_0_1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(lp_balance, U256::from(37_729u64));

    let lp_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.pair_0_1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(40_000u64),
        }
    )
    .build();

    builder.exec(lp_approve_request).expect_success().commit();

    let remove_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_REMOVE_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_LIQUIDITY => U256::from(40_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(remove_liquidity_request).expect_failure().commit();
}

#[test]
fn should_swap_exact_tokens_for_tokens() {
    let (mut builder, test_context) = setup();
    
    let token0_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token0_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    let token1_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token1_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    builder.exec(token0_transfer_request).expect_success().commit();
    builder.exec(token1_transfer_request).expect_success().commit();

    let token0_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token0_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    let token1_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    builder.exec(token0_approve_request).expect_success().commit();
    builder.exec(token1_approve_request).expect_success().commit();

    let router_allowance: U256 = erc20_check_allowance_of(&mut builder, Key::Account(*consts::ACCOUNT_1_ADDR), Key::Hash(test_context.router_package.value()));
    assert_eq!(router_allowance, U256::from(50_000u64));
    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(50_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(add_liquidity_request).expect_success().commit();
    let lp_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.pair_0_1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(lp_balance, U256::from(37_729u64));

    let mut path: Vec<ContractHash> = Vec::new();
    path.push(test_context.token0_contract);
    path.push(test_context.token1_contract);

    let swap_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_SWAP_EXACT_TOKENS_FOR_TOKENS,
        runtime_args! {
            consts::ARG_AMOUNT_IN => U256::from(10_000u64),
            consts::ARG_AMOUNT_OUT_MIN => U256::zero(),
            consts::ARG_PATH => path,
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(swap_request).expect_success().commit();
    let token1_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.token1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(token1_balance, U256::from(62_481u64));
}

#[test]
fn should_swap_exact_tokens_for_tokens_reverse() {
    let (mut builder, test_context) = setup();
    
    let token0_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token0_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    let token1_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token1_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    builder.exec(token0_transfer_request).expect_success().commit();
    builder.exec(token1_transfer_request).expect_success().commit();

    let token0_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token0_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    let token1_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(100_000u64),
        }
    )
    .build();

    builder.exec(token0_approve_request).expect_success().commit();
    builder.exec(token1_approve_request).expect_success().commit();

    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(50_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(add_liquidity_request).expect_success().commit();
    let lp_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.pair_0_1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(lp_balance, U256::from(37_729u64));

    let mut path: Vec<ContractHash> = Vec::new();
    path.push(test_context.token1_contract);
    path.push(test_context.token0_contract);

    let swap_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_SWAP_EXACT_TOKENS_FOR_TOKENS,
        runtime_args! {
            consts::ARG_AMOUNT_IN => U256::from(10_000u64),
            consts::ARG_AMOUNT_OUT_MIN => U256::zero(),
            consts::ARG_PATH => path,
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(swap_request).expect_success().commit();
    let token1_balance = erc20_check_balance_of(&mut builder, &test_context.token1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(token1_balance, U256::from(40_000u64));
    let token0_balance = erc20_check_balance_of(&mut builder, &test_context.token0_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(token0_balance, U256::from(74_991u64));
}

#[test]
fn should_swap_tokens_for_exact_tokens() {
    let (mut builder, test_context) = setup();
    
    let token0_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token0_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    let token1_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token1_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    builder.exec(token0_transfer_request).expect_success().commit();
    builder.exec(token1_transfer_request).expect_success().commit();

    let token0_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token0_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    let token1_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    builder.exec(token0_approve_request).expect_success().commit();
    builder.exec(token1_approve_request).expect_success().commit();

    let router_allowance: U256 = erc20_check_allowance_of(&mut builder, Key::Account(*consts::ACCOUNT_1_ADDR), Key::Hash(test_context.router_package.value()));
    assert_eq!(router_allowance, U256::from(50_000u64));
    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(50_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(add_liquidity_request).expect_success().commit();
    let lp_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.pair_0_1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(lp_balance, U256::from(37_729u64));

    let mut path: Vec<ContractHash> = Vec::new();
    path.push(test_context.token0_contract);
    path.push(test_context.token1_contract);

    let swap_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_SWAP_TOKENS_FOR_EXACT_TOKENS,
        runtime_args! {
            consts::ARG_AMOUNT_OUT => U256::from(10_000u64),
            consts::ARG_AMOUNT_IN_MAX => U256::from(100_000u64),
            consts::ARG_PATH => path,
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(swap_request).expect_success().commit();

    let token1_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.token1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(token1_balance, U256::from(60_000u64));
}

#[test]
fn should_swap_tokens_for_exact_tokens_reverse() {
    let (mut builder, test_context) = setup();
    
    let token0_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token0_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    let token1_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token1_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    builder.exec(token0_transfer_request).expect_success().commit();
    builder.exec(token1_transfer_request).expect_success().commit();

    let token0_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token0_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    let token1_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(100_000u64),
        }
    )
    .build();

    builder.exec(token0_approve_request).expect_success().commit();
    builder.exec(token1_approve_request).expect_success().commit();

    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(50_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(add_liquidity_request).expect_success().commit();
    let lp_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.pair_0_1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(lp_balance, U256::from(37_729u64));

    let mut path: Vec<ContractHash> = Vec::new();
    path.push(test_context.token1_contract);
    path.push(test_context.token0_contract);

    let swap_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_SWAP_TOKENS_FOR_EXACT_TOKENS,
        runtime_args! {
            consts::ARG_AMOUNT_OUT => U256::from(10_000u64),
            consts::ARG_AMOUNT_IN_MAX => U256::from(100_000u64),
            consts::ARG_PATH => path,
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(swap_request).expect_success().commit();

    let token0_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.token0_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(token0_balance, U256::from(80_000u64));

    let token1_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.token1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(token1_balance, U256::from(24_949u64));
}


#[test]
fn should_mint_fee_to_feeto_address() {
    let (mut builder, test_context) = setup();
    
    let token0_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token0_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    let token1_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token1_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    builder.exec(token0_transfer_request).expect_success().commit();
    builder.exec(token1_transfer_request).expect_success().commit();

    let token0_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token0_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(100_000u64),
        }
    )
    .build();

    let token1_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(100_000u64),
        }
    )
    .build();

    builder.exec(token0_approve_request).expect_success().commit();
    builder.exec(token1_approve_request).expect_success().commit();

    let router_allowance: U256 = erc20_check_allowance_of(&mut builder, Key::Account(*consts::ACCOUNT_1_ADDR), Key::Hash(test_context.router_package.value()));
    assert_eq!(router_allowance, U256::from(100_000u64));
    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(50_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(add_liquidity_request).expect_success().commit();
    let lp_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.pair_0_1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(lp_balance, U256::from(37_729u64));

    let mut path: Vec<ContractHash> = Vec::new();
    path.push(test_context.token0_contract);
    path.push(test_context.token1_contract);

    let swap_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_SWAP_EXACT_TOKENS_FOR_TOKENS,
        runtime_args! {
            consts::ARG_AMOUNT_IN => U256::from(10_000u64),
            consts::ARG_AMOUNT_OUT_MIN => U256::zero(),
            consts::ARG_PATH => path,
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(swap_request).expect_success().commit();
    // let token1_balance = erc20_check_balance_of(&mut builder, &test_context.token1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    // assert_eq!(token1_balance, U256::zero());

    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();
    builder.exec(add_liquidity_request).expect_success().commit();

    let fee_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.pair_0_1_contract, Key::from(AccountHash::new([10u8; 32])));
    assert_eq!(fee_balance, U256::from(2u64));
}

#[test]
fn should_get_error_set_feeto_without_permission() {
    let (mut builder, test_context) = setup();

    let set_feeto_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None, 
        consts::METHOD_SET_FEETO,
        runtime_args! {
            consts::FEETO_KEY_NAME => Key::Account(AccountHash::new([111u8; 32])),
        }
    )
    .build();

    builder.exec(set_feeto_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == consts::ERROR_PERMISSION),
        "{:?}",
        error
    );
}

#[test]
fn should_get_error_set_feeto_setter_without_permission() {
    let (mut builder, test_context) = setup();

    let set_feeto_setter_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None, 
        consts::METHOD_SET_FEETO_SETTER,
        runtime_args! {
            consts::FEETO_SETTER_KEY_NAME => Key::Account(AccountHash::new([111u8; 32])),
        }
    )
    .build();

    builder.exec(set_feeto_setter_request).commit();

    let error = builder.get_error().expect("should have error");
    assert!(
        matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == consts::ERROR_PERMISSION),
        "{:?}",
        error
    );
}

#[test]
fn should_swap_exact_tokens_for_tokens_supporting_fee() {
    let (mut builder, test_context) = setup();
    
    let token0_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token0_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    let token1_transfer_request = make_erc20_transfer_request(
        Key::Account(*DEFAULT_ACCOUNT_ADDR),
        &test_context.token1_contract,
        Key::Account(*consts::ACCOUNT_1_ADDR),
        U256::from(100_000u64),
    );

    builder.exec(token0_transfer_request).expect_success().commit();
    builder.exec(token1_transfer_request).expect_success().commit();

    let token0_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token0_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    let token1_approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.token1_contract,
        consts::METHOD_APPROVE,
        runtime_args! {
            consts::ARG_OWNER => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_SPENDER => Key::Hash(test_context.router_package.value()),
            consts::ARG_AMOUNT => U256::from(50_000u64),
        }
    )
    .build();

    builder.exec(token0_approve_request).expect_success().commit();
    builder.exec(token1_approve_request).expect_success().commit();

    let router_allowance: U256 = erc20_check_allowance_of(&mut builder, Key::Account(*consts::ACCOUNT_1_ADDR), Key::Hash(test_context.router_package.value()));
    assert_eq!(router_allowance, U256::from(50_000u64));
    let add_liquidity_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_ADD_LIQUIDITY,
        runtime_args! {
            consts::ARG_TOKEN0 => Key::from(test_context.token0_contract),
            consts::ARG_TOKEN1 => Key::from(test_context.token1_contract),
            consts::ARG_AMOUNT0_DESIRED => U256::from(30_000u64),
            consts::ARG_AMOUNT1_DESIRED => U256::from(50_000u64),
            consts::ARG_AMOUNT0_MIN => U256::zero(),
            consts::ARG_AMOUNT1_MIN => U256::zero(),
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(add_liquidity_request).expect_success().commit();
    let lp_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.pair_0_1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(lp_balance, U256::from(37_729u64));

    let mut path: Vec<ContractHash> = Vec::new();
    path.push(test_context.token0_contract);
    path.push(test_context.token1_contract);

    let swap_request: ExecuteRequest = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *consts::ACCOUNT_1_ADDR,
        test_context.router_package,
        None,
        consts::METHOD_SWAP_EXACT_TOKENS_FOR_TOKENS,
        runtime_args! {
            consts::ARG_AMOUNT_IN => U256::from(10_000u64),
            consts::ARG_AMOUNT_OUT_MIN => U256::zero(),
            consts::ARG_PATH => path,
            consts::ARG_TO => Key::Account(*consts::ACCOUNT_1_ADDR),
            consts::ARG_DEAD_LINE => U256::MAX,
        },
    )
    .build();

    builder.exec(swap_request).expect_success().commit();
    let token1_balance: U256 = erc20_check_balance_of(&mut builder, &test_context.token1_contract, Key::Account(*consts::ACCOUNT_1_ADDR));
    assert_eq!(token1_balance, U256::from(62_481u64));
}
