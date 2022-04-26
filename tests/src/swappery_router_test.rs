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
use constants as consts;

#[derive(Copy, Clone)]
struct TestContext {
    erc20_token0: ContractHash,
    erc20_token1: ContractHash,
    wcspr_token: ContractHash,
    swappery_pair_0_1: ContractPackageHash,
    swappery_router: ContractPackageHash,
}