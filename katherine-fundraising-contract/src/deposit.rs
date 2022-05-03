use core::panic;

use crate::*;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{env, log, near_bindgen, PromiseOrValue};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

#[near_bindgen]
impl FungibleTokenReceiver for KatherineFundraising {
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let kickstarter_id = match msg.parse::<KickstarterId>() {
            Ok(_id) => _id,
            Err(_) => panic!("Invalid KickstarterId."),
        };
        let mut kickstarter: Kickstarter = self.internal_get_kickstarter(kickstarter_id);
        let amount = Balance::from(amount);
        if env::predecessor_account_id() == self.metapool_contract_address {
            // Deposit is in stNEAR.
            self.assert_min_deposit_amount(amount);
            log!(
                "DEPOSIT: {} stNEAR deposited from {} to KickstarterId {}",
                amount,
                sender_id.as_ref(),
                msg
            );
            self.process_supporter_deposit(sender_id.as_ref(), &amount, &mut kickstarter);
        } else {
            // Deposit is in a Kickstarter Token.
            log!(
                "DEPOSIT: {} pTOKEN deposited from {} to KickstarterId {}",
                amount,
                sender_id.as_ref(),
                msg
            );
            self.process_kickstarter_deposit(&amount, &mut kickstarter);
        }
        // Return unused amount
        PromiseOrValue::Value(U128::from(0))
    }
}

#[near_bindgen]
impl KatherineFundraising {
    fn assert_min_deposit_amount(&self, amount: Balance) {
        assert!(
            amount >= self.min_deposit_amount,
            "minimum deposit amount is {}",
            self.min_deposit_amount
        );
    }

    /// Process a stNEAR deposit to Katherine Contract.
    fn process_supporter_deposit(
        &mut self,
        supporter_id: &AccountId,
        amount: &Balance,
        kickstarter: &mut Kickstarter,
    ) {
        // Update Kickstarter
        kickstarter.assert_within_funding_period();
        kickstarter.assert_enough_reward_tokens();

        let new_total_deposited = kickstarter.total_deposited + amount;
        assert!(
            new_total_deposited <= kickstarter.deposits_hard_cap,
            "The deposits hard cap cannot be exceeded!"
        );
        kickstarter.total_deposited = new_total_deposited;
        kickstarter.update_supporter_deposits(&supporter_id, amount);
        self.kickstarters
            .replace(kickstarter.id as u64, &kickstarter);

        // Update Supporter.
        let mut supporter = self.internal_get_supporter(&supporter_id);
        supporter.total_in_deposits += amount;
        supporter.supported_projects.insert(&kickstarter.id);
        self.supporters.insert(&supporter_id, &supporter);
    }

    /// Process a reward token deposit to Katherine Contract.
    fn process_kickstarter_deposit(
        &mut self,
        amount: &Balance,
        kickstarter: &mut Kickstarter,
    ) {
        assert_eq!(
            &env::predecessor_account_id(),
            &kickstarter.token_contract_address,
            "Deposited tokens do not correspond to the Kickstarter contract."
        );
        assert!(
            get_current_epoch_millis() < kickstarter.close_timestamp,
            "Kickstarter Tokens should be provided before the funding period ends."
        );
        let max_tokens_to_release = self.calculate_max_tokens_to_release(&kickstarter);
        let min_tokens_to_allow_support = max_tokens_to_release
            + self.calculate_katherine_fee(max_tokens_to_release);
        kickstarter.available_reward_tokens += amount;
        kickstarter.enough_reward_tokens = {
            kickstarter.available_reward_tokens >= min_tokens_to_allow_support
        };
        self.kickstarters
            .replace(kickstarter.id as u64, &kickstarter);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[allow(unused_imports)]
mod tests {
    use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
    use near_contract_standards::storage_management::StorageManagement;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, Balance};
    use near_sdk::{MockedBlockchain, ValidatorId};
    use std::convert::TryInto;

    use super::*;

    fn get_time_millis(ctx: &VMContextBuilder) -> u64 {
        ctx.context.block_timestamp / 1_000_000
    }

    fn acc_metapool() -> ValidAccountId {
        "metapool".try_into().unwrap()
    }

    fn acc_owner() -> ValidAccountId {
        "owner".try_into().unwrap()
    }

    const STARTING_TIMESTAMP: u64 = 100_000_000_000_000_000;

    fn setup_contract(predecessor: ValidAccountId) -> (VMContextBuilder, KatherineFundraising) {
        let mut context = VMContextBuilder::new();
        testing_env!(context.build());
        testing_env!(context
            .predecessor_account_id(predecessor)
            .block_timestamp(STARTING_TIMESTAMP)
            .build());
        let contract = KatherineFundraising::new(
            acc_owner().to_string(), // owner
            0,
            acc_metapool().to_string(),
            1,
        );
        (context, contract)
    }

    #[test]
    fn add_supporter_with_ext_callback() {
        let supporter = accounts(0);
        let kickstarter_owner = accounts(1);
        let kickstarter_token_acc = accounts(2);
        let (mut ctx, mut ctr) = setup_contract(acc_owner());

        // create a kickstarter
        let kickstarter_id = ctr.create_kickstarter(
            "first_kickstarter".to_owned(),
            "FK".to_owned(),
            kickstarter_owner.to_string(),
            get_time_millis(&ctx),
            get_time_millis(&ctx) + 1_000 * 60 * 5,
            kickstarter_token_acc.to_string(),
        );
        // become a supporter
        testing_env!(ctx.predecessor_account_id(acc_metapool()).build());
        let promise = ctr.ft_on_transfer(supporter.clone(), 1.into(), kickstarter_id.to_string());

        match promise {
            PromiseOrValue::Promise(_) => {
                println!("error, method returned a promise");
                std::panic!();
            }
            PromiseOrValue::Value(v) => v,
        };

        let kickstarter_data = ctr.get_kickstarter(kickstarter_id);
        assert_eq!(
            kickstarter_data.total_supporters, 1,
            "incorrrect number of supporters for kickstarter"
        );
    }
}
