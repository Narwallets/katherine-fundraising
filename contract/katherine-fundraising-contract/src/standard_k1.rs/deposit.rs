use crate::*;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{env, log, near_bindgen, PromiseOrValue};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

#[near_bindgen]
impl FungibleTokenReceiver for StandardK1Contract {
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
        let amount = amount.0;
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
            self.process_kickstarter_deposit(amount, &mut kickstarter);
        }
        // Return unused amount
        PromiseOrValue::Value(U128::from(0))
    }
}

#[near_bindgen]
impl StandardK1Contract {
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
        supporter.supported_projects.insert(&kickstarter.id);
        self.supporters.insert(&supporter_id, &supporter);
    }

    /// Process a reward token deposit to Katherine Contract.
    fn process_kickstarter_deposit(
        &mut self,
        amount: Balance,
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
        let amount = kickstarter.less_to_yocto_decimals(amount);
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
