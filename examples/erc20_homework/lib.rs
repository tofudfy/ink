#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
    // #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::collections::HashMap;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Erc20 {
        // Stores a single `bool` value on the storage.
        total_supply: Balance,
        //
        balances: HashMap<AccountId, Balance>,

        /// Balances that are spendable by non-owners: (owner, spender) -> allowed
        allowances: HashMap<(AccountId, AccountId), Balance>,
    }

    /// Defines the event of your contract
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: Option<AccountId>,
        #[ink(topic)]
        spender: Option<AccountId>,
        value: Balance,
    }

    // PartialEq, 否则Error间无法比较 (==)
    // Debug, 否则无法assert进行debug
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
    }

    type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let allowances = HashMap::new();
            let mut balances = HashMap::new();
            balances.insert(caller, init_supply);

            Self::env()
                .emit_event(
                    Transfer {
                        from: None,
                        to: Some(caller),
                        value: init_supply,
                    }
                );

            Self {
                total_supply: init_supply,
                balances,
                allowances
            }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// Get the total supply
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            // ACTION: Return the total supply
            self.total_supply
        }

        /// check the balance of the owner
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            // ACTION: Return the balance of `owner`
            Self::balance_of_or_zero(&self, &owner)
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            // ACTION: `get` the balance of `owner`, then `unwrap_or` fallback to 0
            *self.balances.get(owner).unwrap_or(&0)
        }

        /// Approve the passed AccountId to spend the specified amount of tokens
        /// on the behalf of the message's sender.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            // ACTION: Get the `self.env().caller()` and store it as the `owner`
            let owner = self.env().caller();

            // ACTION: Insert the new allowance into the `allowances` HashMap
            self.allowances.insert((owner, spender), value);

            // ACTION: `emit` the `Approval` event you created using these values
            self.env()
                .emit_event(
                    Approval {
                        owner: Some(owner),
                        spender: Some(spender),
                        value,
                    }
                );

            Ok(())
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            // ACTION: Create a getter for the `allowances` HashMap
            Self::allowance_of_or_zero(self, &owner, &spender)
        }

        fn allowance_of_or_zero(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            // ACTION: `get` the `allowances` of `(owner, spender)` and `unwrap_or` return `0`.
            *self.allowances.get(&(*owner, *spender)).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            // ACTION: Get the allowance for `(from, self.env().caller())` using `allowance_of_or_zero`
            let allowance = Self::allowance_of_or_zero(self, &from, &self.env().caller());

            // ACTION: `if` the `allowance` is less than the `value`, exit early and return `false`
            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }

            // ACTION: `insert` the new allowance into the map for `(from, self.env().caller())`
            self.allowances.insert((from, self.env().caller()), allowance - value);

            // ACTION: Finally, call the `transfer_from_to` for `from` and `to`\
            // ACTION: Return true if everything was successful
            Self::transfer_from_to(self, from, to, value)
        }

        /// transfer the balance from sender to receiver
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            // ACTION: Call the `transfer_from_to` with `from` as `self.env().caller()`
            Self::transfer_from_to(self,self.env().caller(), to, value)
        }

        fn transfer_from_to(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            // ACTION: Get the balance for `from` and `to`
            let balance_from = Self::balance_of_or_zero(&self, &from);
            let balance_to = Self::balance_of_or_zero(&self, &to);

            // ACTION: If `from_balance` is less than `value`, return `false`
            if balance_from < value {
                return Err(Error::InsufficientBalance);
            }

            // ACTION: Insert new values for `from` and `to`
            //         * from_balance - value
            self.balances.insert(from, balance_from - value);
            self.balances.insert(to, balance_to + value);

            self.env()
                .emit_event(
                    Transfer {
                        from: Some(from),
                        to: Some(to),
                        value,
                    }
                );

            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    // #[cfg(feature = "ink-experimental-engine")]
    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn new_works() {
            let contract = Erc20::new(777);
            assert_eq!(contract.total_supply(), 777);
        }

        // the default address is AccountId::from([0x1; 32])
        #[ink::test]
        fn balance_works() {
            let contract = Erc20::new(100);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.total_supply(), 100);
            contract.transfer(AccountId::from([0x2; 32]), 20);

            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 80);
            assert_eq!(contract.balance_of(AccountId::from([0x2; 32])), 20);
        }

        #[ink::test]
        fn transfer_insufficient() {
            let mut contract = Erc20::new(100);

            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);

            assert_eq!(
                contract.transfer(AccountId::from([0x2; 32]), 120),
                Err(Error::InsufficientBalance)
            );
        }

        #[ink::test]
        fn allowance_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.total_supply(), 100);

            contract.approve(AccountId::from([0x2; 32]), 20);

            assert_eq!(contract.allowance(
                AccountId::from([0x1; 32]),
                AccountId::from([0x2; 32])
            ), 20);
        }

        /*
        #[ink::test]
        fn transfer_from_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);

            contract.approve(AccountId::from([0x2; 32]), 20);
            let contract_addr = ink_env::account_id::<ink_env::DefaultEnvironment>();
            ink_env::test::set_callee::<ink_env::DefaultEnvironment>(contract_addr);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(AccountId::from([0x2; 32]));

            contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x3; 32]), 10);
            assert_eq!(contract.balance_of(AccountId::from([0x3; 32])), 10);
        }

        #[ink::test]
        fn transfer_from_insufficient() {
            let mut contract = Erc20::new(100);
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

            // Check the balance of alice (default caller) and set 20 allowance to bob
            assert_eq!(contract.balance_of(accounts.alice), 100);
            contract.approve(accounts.bob, 20);

            // Set the contract as callee and Bob as caller.
            let contract_addr = ink_env::account_id::<ink_env::DefaultEnvironment>();
            ink_env::test::set_callee::<ink_env::DefaultEnvironment>(contract_addr);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.bob);

            assert_eq!(
                contract.transfer_from(accounts.alice, accounts.charlie, 10),
                Err(Error::InsufficientAllowance)
            );
        }
         */
    }
}
