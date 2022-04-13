#[cfg(test)]
mod test_fixture;

#[cfg(test)]
mod tests {
    use casper_types::{Key, U256, contracts::ContractPackageHash, account::AccountHash };

    use casper_erc20::Address;

    use crate::test_fixture::{Sender, TestFixture};

    #[test]
    fn should_install() {
        let fixture = TestFixture::install_contract();
        assert_eq!(fixture.pair_token_name(), TestFixture::TOKEN_NAME);
        assert_eq!(fixture.pair_token_symbol(), TestFixture::TOKEN_SYMBOL);
        assert_eq!(fixture.pair_token_decimals(), TestFixture::TOKEN_DECIMALS);
        assert_eq!(
            fixture.pair_balance_of(Key::from(fixture.ali)),
            Some(U256::zero())
        );
        assert_eq!(fixture.pair_reserve0(), U256::zero());
        assert_eq!(fixture.pair_reserve1(), U256::zero());
        assert_eq!(fixture.pair_token0(), Address::from(fixture.token0_contract_package_hash()));
        assert_eq!(fixture.pair_token1(), Address::from(fixture.token1_contract_package_hash()));
    }

    #[test]
    fn should_transfer() {
        let mut fixture = TestFixture::install_contract();
        assert_eq!(fixture.pair_balance_of(Key::from(fixture.bob)), None);
        assert_eq!(
            fixture.pair_balance_of(Key::from(fixture.ali)),
            Some(TestFixture::token_total_supply())
        );
        let transfer_amount_1 = U256::from(42u64);
        fixture.pair_transfer(
            Key::from(fixture.bob),
            transfer_amount_1,
            Sender(fixture.ali),
        );
        assert_eq!(
            fixture.pair_balance_of(Key::from(fixture.bob)),
            Some(transfer_amount_1)
        );
        assert_eq!(
            fixture.pair_balance_of(Key::from(fixture.ali)),
            Some(TestFixture::token_total_supply() - transfer_amount_1)
        );

        let transfer_amount_2 = U256::from(20u64);
        fixture.pair_transfer(
            Key::from(fixture.ali),
            transfer_amount_2,
            Sender(fixture.bob),
        );
        assert_eq!(
            fixture.pair_balance_of(Key::from(fixture.ali)),
            Some(TestFixture::token_total_supply() - transfer_amount_1 + transfer_amount_2),
        );
        assert_eq!(
            fixture.pair_balance_of(Key::from(fixture.bob)),
            Some(transfer_amount_1 - transfer_amount_2)
        );
    }

    #[test]
    fn should_transfer_full_amount() {
        let mut fixture = TestFixture::install_contract();

        let initial_ali_balance = fixture.pair_balance_of(Key::from(fixture.ali)).unwrap();
        assert_eq!(fixture.pair_balance_of(Key::from(fixture.bob)), None);

        fixture.pair_transfer(
            Key::from(fixture.bob),
            initial_ali_balance,
            Sender(fixture.ali),
        );

        assert_eq!(
            fixture.pair_balance_of(Key::from(fixture.bob)),
            Some(initial_ali_balance)
        );
        assert_eq!(
            fixture.pair_balance_of(Key::from(fixture.ali)),
            Some(U256::zero())
        );

        fixture.pair_transfer(
            Key::from(fixture.ali),
            initial_ali_balance,
            Sender(fixture.bob),
        );

        assert_eq!(
            fixture.pair_balance_of(Key::from(fixture.bob)),
            Some(U256::zero())
        );
        assert_eq!(
            fixture.pair_balance_of(Key::from(fixture.ali)),
            Some(initial_ali_balance)
        );
    }

    #[should_panic(expected = "ApiError::User(65534) [131070]")]
    #[test]
    fn should_not_transfer_with_insufficient_balance() {
        let mut fixture = TestFixture::install_contract();

        let initial_ali_balance = fixture.pair_balance_of(Key::from(fixture.ali)).unwrap();
        assert_eq!(fixture.pair_balance_of(Key::from(fixture.bob)), None);

        fixture.pair_transfer(
            Key::from(fixture.bob),
            initial_ali_balance + U256::one(),
            Sender(fixture.ali),
        );
    }

    #[test]
    fn should_transfer_from() {
        let approve_amount = U256::from(100u64);
        let transfer_amount = U256::from(42u64);
        assert!(approve_amount > transfer_amount);

        let mut fixture = TestFixture::install_contract();

        let owner = fixture.ali;
        let spender = fixture.bob;
        let recipient = fixture.joe;

        let owner_balance_before = fixture
            .pair_balance_of(Key::from(owner))
            .expect("owner should have balance");
        fixture.pair_approve(Key::from(spender), approve_amount, Sender(owner));
        assert_eq!(
            fixture.pair_allowance(Key::from(owner), Key::from(spender)),
            Some(approve_amount)
        );

        fixture.pair_transfer_from(
            Key::from(owner),
            Key::from(recipient),
            transfer_amount,
            Sender(spender),
        );

        assert_eq!(
            fixture.pair_balance_of(Key::from(owner)),
            Some(owner_balance_before - transfer_amount),
            "should decrease balance of the owner"
        );
        assert_eq!(
            fixture.pair_allowance(Key::from(owner), Key::from(spender)),
            Some(approve_amount - transfer_amount),
            "should decrease allowance of the spender"
        );
        assert_eq!(
            fixture.pair_balance_of(Key::from(recipient)),
            Some(transfer_amount),
            "recipient should receive tokens"
        );
    }

    #[should_panic(expected = "ApiError::User(65533) [131069]")]
    #[test]
    fn should_not_transfer_from_more_than_approved() {
        let approve_amount = U256::from(100u64);
        let transfer_amount = U256::from(42u64);
        assert!(approve_amount > transfer_amount);

        let mut fixture = TestFixture::install_contract();

        let owner = fixture.ali;
        let spender = fixture.bob;
        let recipient = fixture.joe;

        fixture.pair_approve(Key::from(spender), approve_amount, Sender(owner));
        assert_eq!(
            fixture.pair_allowance(Key::from(owner), Key::from(spender)),
            Some(approve_amount)
        );

        fixture.pair_transfer_from(
            Key::from(owner),
            Key::from(recipient),
            approve_amount + U256::one(),
            Sender(spender),
        );
    }

    #[test]
    fn should_transfer_token0_to_pair() {
        let mut fixture = TestFixture::install_contract();
        let pair_contract_package_hash: ContractPackageHash = fixture.pair_contract_package_hash();
        fixture.token0_transfer(
            Key::from(pair_contract_package_hash),
            U256::from(200u64),
            Sender(fixture.ali),
        );

        assert_eq!(
            fixture.token0_balance_of(Key::from(pair_contract_package_hash)),
            Some(U256::from(200u64))
        );
    }

    #[test]
    fn should_mint() {
        let mut fixture = TestFixture::install_contract();
        let pair_contract_package_hash: ContractPackageHash = fixture.pair_contract_package_hash();
        fixture.token0_transfer(
            Key::from(pair_contract_package_hash),
            U256::from(5_000u64),
            Sender(fixture.ali),
        );
        fixture.token1_transfer(
            Key::from(pair_contract_package_hash),
            U256::from(10_000u64),
            Sender(fixture.bob),
        );

        fixture.pair_mint(
            Key::from(fixture.joe),
            Sender(fixture.ali),
        );
        
        assert_eq!(
            fixture.pair_balance_of(Key::from(AccountHash::new([0u8; 32]))),
            Some(U256::from(1000u64))
        );

        // assert_eq!(
        //     fixture.pair_balance_of(Key::from(fixture.joe)),
        //     Some(U256::zero())
        // );

    }

    #[test]
    fn should_burn() {
        let mut fixture = TestFixture::install_contract();
        let pair_contract_package_hash: ContractPackageHash = fixture.pair_contract_package_hash();
        fixture.token0_transfer(
            Key::from(pair_contract_package_hash),
            U256::from(5_000u64),
            Sender(fixture.ali),
        );
        fixture.token1_transfer(
            Key::from(pair_contract_package_hash),
            U256::from(10_000u64),
            Sender(fixture.bob),
        );

        fixture.pair_mint(
            Key::from(fixture.ali),
            Sender(fixture.ali),
        );

        fixture.pair_transfer(
            Key::from(pair_contract_package_hash),
            U256::from(3000),
            Sender(fixture.ali),
        );

        fixture.pair_burn(
            Key::from(fixture.ali),
            Sender(fixture.ali),
        );
        
        assert_eq!(
            fixture.pair_balance_of(Key::from(AccountHash::new([0u8; 32]))),
            Some(U256::from(1000u64))
        );
    }

    #[test]
    fn should_swap() {
        let mut fixture = TestFixture::install_contract();
        let pair_contract_package_hash: ContractPackageHash = fixture.pair_contract_package_hash();
        fixture.token0_transfer(
            Key::from(pair_contract_package_hash),
            U256::from(5_000u64),
            Sender(fixture.ali),
        );
        fixture.token1_transfer(
            Key::from(pair_contract_package_hash),
            U256::from(10_000u64),
            Sender(fixture.bob),
        );

        fixture.pair_mint(
            Key::from(fixture.ali),
            Sender(fixture.ali),
        );

        fixture.token0_transfer(
            Key::from(pair_contract_package_hash),
            U256::from(1000u64),
            Sender(fixture.ali),
        );

        fixture.pair_swap(
            U256::zero(),
            U256::from(1500u64),
            Key::from(fixture.ali),
            Sender(fixture.ali),
        );
        
        assert_eq!(
            fixture.pair_balance_of(Key::from(AccountHash::new([0u8; 32]))),
            Some(U256::from(1000u64))
        );
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
