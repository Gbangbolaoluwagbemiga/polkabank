#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod polkabank {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct PolkaBank {
        balances: Mapping<AccountId, u128>,
        
        /// The total supply of tokens in circulation
        total_supply: u128,
        
        /// The owner of the contract (only they can mint new tokens)
        owner: AccountId,
    }

    /// Events that the contract emits for important operations
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: AccountId,
        value: u128,
    }

    #[ink(event)]
    pub struct Mint {
        #[ink(topic)]
        to: AccountId,
        value: u128,
    }

    /// Custom errors that may occur during contract operations
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        InsufficientBalance,
        NotOwner,
        InvalidRecipient,
                Overflow,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl PolkaBank {
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            
            Self {
                balances: Mapping::default(),
                total_supply: 0,
                owner: caller,
            }
        }

        #[ink(constructor)]
        pub fn new_with_supply(initial_supply: u128) -> Self {
            let caller = Self::env().caller();
            let mut balances = Mapping::default();
                        balances.insert(caller, &initial_supply);
            
            Self {
                balances,
                total_supply: initial_supply,
                owner: caller,
            }
        }

     
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, amount: u128) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }

            let current_balance = self.balances.get(to).unwrap_or(0);
            
            let new_balance = current_balance
                .checked_add(amount)
                .ok_or(Error::Overflow)?;
            
            self.total_supply = self.total_supply
                .checked_add(amount)
                .ok_or(Error::Overflow)?;
            
            self.balances.insert(to, &new_balance);
            
            self.env().emit_event(Mint { to, value: amount });
            
            Ok(())
        }


        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u128 {
            self.balances.get(account).unwrap_or(0)
        }


        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: u128) -> Result<()> {
            let caller = self.env().caller();
            
            if caller == to {
                return Ok(());
            }
            
            let sender_balance = self.balances.get(caller).unwrap_or(0);
            
            if sender_balance < amount {
                return Err(Error::InsufficientBalance);
            }
            
            let recipient_balance = self.balances.get(to).unwrap_or(0);
            
            let new_sender_balance = sender_balance
                .checked_sub(amount)
                .ok_or(Error::InsufficientBalance)?;
            let new_recipient_balance = recipient_balance
                .checked_add(amount)
                .ok_or(Error::Overflow)?;
            
            self.balances.insert(caller, &new_sender_balance);
            self.balances.insert(to, &new_recipient_balance);
            
            // Emit a transfer event
            self.env().emit_event(Transfer {
                from: Some(caller),
                to,
                value: amount,
            });
            
            Ok(())
        }

        /// Get the total supply of tokens in circulation
        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }

        /// Get the owner of the contract
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Transfer ownership to a new owner
        /// Only the current owner can call this
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            
            self.owner = new_owner;
            Ok(())
        }
    }


}