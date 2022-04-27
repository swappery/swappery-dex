use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
};
use casper_execution_engine::core::{
    engine_state::{ExecuteRequest},
};
use casper_types::{
    bytesrepr::FromBytes, runtime_args, CLTyped,
    ContractHash, ContractPackageHash, Key, RuntimeArgs, U256,
};
use crate::constants as consts;

pub(crate) fn erc20_check_total_supply(
    builder: &mut InMemoryWasmTestBuilder,
    erc20_contract_hash: &ContractHash,
) -> U256 {
    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let erc20_test_contract_hash = account
        .named_keys()
        .get(consts::ERC20_TEST_CALL_KEY)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have test contract hash");

    let check_total_supply_args = runtime_args! {
        consts::ARG_TOKEN_CONTRACT => *erc20_contract_hash,
    };

    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        erc20_test_contract_hash,
        None,
        consts::CHECK_TOTAL_SUPPLY_ENTRYPOINT,
        check_total_supply_args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, erc20_test_contract_hash)
}

pub(crate) fn get_test_result<T: FromBytes + CLTyped>(
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

    builder.get_value(*contract_hash, consts::RESULT_KEY)
}

pub(crate) fn erc20_check_balance_of(
    builder: &mut InMemoryWasmTestBuilder,
    erc20_contract_hash: &ContractHash,
    address: Key,
) -> U256 {
    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let erc20_test_contract_hash = account
        .named_keys()
        .get(consts::ERC20_TEST_CALL_KEY)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have test contract hash");

    let check_balance_args = runtime_args! {
        consts::ARG_TOKEN_CONTRACT => *erc20_contract_hash,
        consts::ARG_ADDRESS => address,
    };
    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        erc20_test_contract_hash,
        None,
        consts::CHECK_BALANCE_OF_ENTRYPOINT,
        check_balance_args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, erc20_test_contract_hash)
}

pub(crate) fn erc20_check_allowance_of(
    builder: &mut InMemoryWasmTestBuilder,
    owner: Key,
    spender: Key,
) -> U256 {
    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");
    let erc20_contract_hash = account
        .named_keys()
        .get(consts::TOKEN0_CONTRACT_HASH_KEY_NAME)
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have test contract hash");
    let erc20_test_contract_hash = account
        .named_keys()
        .get(consts::ERC20_TEST_CALL_KEY)
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have test contract hash");

    let check_balance_args = runtime_args! {
        consts::ARG_TOKEN_CONTRACT => erc20_contract_hash,
        consts::ARG_OWNER => owner,
        consts::ARG_SPENDER => spender,
    };
    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        erc20_test_contract_hash,
        None,
        consts::CHECK_ALLOWANCE_OF_ENTRYPOINT,
        check_balance_args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, erc20_test_contract_hash)
}

pub(crate) fn make_erc20_transfer_request(
    sender: Key,
    erc20_token: &ContractHash,
    recipient: Key,
    amount: U256,
) -> ExecuteRequest {
    match sender {
        Key::Account(sender) => ExecuteRequestBuilder::contract_call_by_hash(
            sender,
            *erc20_token,
            consts::METHOD_TRANSFER,
            runtime_args! {
                consts::ARG_AMOUNT => amount,
                consts::ARG_RECIPIENT => recipient,
            },
        )
        .build(),
        Key::Hash(contract_package_hash) => ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            ContractPackageHash::new(contract_package_hash),
            None,
            consts::METHOD_TRANSFER_AS_STORED_CONTRACT,
            runtime_args! {
                consts::ARG_TOKEN_CONTRACT => *erc20_token,
                consts::ARG_AMOUNT => amount,
                consts::ARG_RECIPIENT => recipient,
            },
        )
        .build(),
        _ => panic!("Unknown variant"),
    }
}

pub(crate) fn make_erc20_approve_request(
    sender: Key,
    erc20_token: &ContractHash,
    spender: Key,
    amount: U256,
) -> ExecuteRequest {
    match sender {
        Key::Account(sender) => ExecuteRequestBuilder::contract_call_by_hash(
            sender,
            *erc20_token,
            consts::METHOD_APPROVE,
            runtime_args! {
                consts::ARG_SPENDER => spender,
                consts::ARG_AMOUNT => amount,
            },
        )
        .build(),
        Key::Hash(contract_hash) => ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            ContractPackageHash::new(contract_hash),
            None,
            consts::METHOD_APPROVE_AS_STORED_CONTRACT,
            runtime_args! {
                consts::ARG_TOKEN_CONTRACT => *erc20_token,
                consts::ARG_SPENDER => spender,
                consts::ARG_AMOUNT => amount,
            },
        )
        .build(),
        _ => panic!("Unknown variant"),
    }
}