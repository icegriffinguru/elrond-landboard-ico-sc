#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();


/// Manage ICO of a new ESDT
#[elrond_wasm::contract]
pub trait LandboardIco {
    #[init]
    fn init(&self, token_id: TokenIdentifier, locked_token_id: TokenIdentifier, token_price: BigUint, min_buy_limit: BigUint, max_buy_limit: BigUint) {
        require!(
            token_id.is_valid_esdt_identifier(),
            "Invalid token identifier"
        );
        require!(
            locked_token_id.is_valid_esdt_identifier(),
            "Invalid locked token identifier"
        );
        self.token_id().set(&token_id);
        self.locked_token_id().set(&locked_token_id);
        self.token_price().set(&token_price);
        self.min_buy_limit().set(&min_buy_limit);
        self.max_buy_limit().set(&max_buy_limit);

        self.sale_started().set(false);
        self.is_allowlist_enabled().set(false);
    }


    /// endpoint - only owner ///
    #[only_owner]
    #[endpoint(startSale)]
    fn start_sale(&self) -> SCResult<()> {
        self.sale_started().set(true);

        Ok(())
    }

    #[only_owner]
    #[endpoint(stopSale)]
    fn stop_sale(&self) -> SCResult<()> {
        self.sale_started().set(false);

        Ok(())
    }

    #[only_owner]
    #[endpoint(updateTokenPrice)]
    fn update_token_price(&self, token_price: BigUint) -> SCResult<()> {
        self.token_price().set(&token_price);

        Ok(())
    }

    #[only_owner]
    #[endpoint(updateMinBuyLimit)]
    fn update_min_buy_limit(&self, min_buy_limit: BigUint) -> SCResult<()> {
        self.min_buy_limit().set(&min_buy_limit);

        Ok(())
    }

    #[only_owner]
    #[endpoint(updateMaxBuyLimit)]
    fn update_max_buy_limit(&self, min_buy_limit: BigUint) -> SCResult<()> {
        self.min_buy_limit().set(&min_buy_limit);

        Ok(())
    }
    
    // withdraw EGLD
    #[only_owner]
    #[endpoint(withdrawEgld)]
    fn withdraw_egld(&self) -> SCResult<()> {
        let balance = self.blockchain().get_sc_balance(&TokenIdentifier::egld(), 0);
        require!(balance != 0, "not enough egld");

        let caller = self.blockchain().get_caller();
        
        self.send().direct(&caller, &TokenIdentifier::egld(), 0, &balance, &[]);

        Ok(())
    }

    // withdraw esdt
    #[only_owner]
    #[endpoint(withdrawEsdt)]
    fn withdraw_esdt(&self, token_id: TokenIdentifier, amount: BigUint) -> SCResult<()> {
        let balance = self.blockchain().get_sc_balance(&token_id, 0);
        require!(amount <= balance, "not enough token of given token_id");

        let caller = self.blockchain().get_caller();
        
        self.send().direct(&caller, &token_id, 0, &amount, &[]);

        Ok(())
    }

    #[only_owner]
    #[endpoint(enableAllowlist)]
    fn enable_allowlist(&self) -> SCResult<()> {
        self.is_allowlist_enabled().set(true);

        Ok(())
    }

    #[only_owner]
    #[endpoint(disableAllowlist)]
    fn disable_allowlist(&self) -> SCResult<()> {
        self.is_allowlist_enabled().set(false);

        Ok(())
    }

    #[only_owner]
    #[endpoint(populateAllowlist)]
    fn populate_allowlist(&self, addresses: ManagedVec<ManagedAddress>) -> SCResult<()> {
        self.allowlist().extend(&addresses);

        Ok(())
    }


    /// endpoint ///
    
    #[payable("*")]
    #[endpoint(buy)]
    fn buy(&self, #[payment_amount] paid_amount: BigUint) -> SCResult<()> {
        self.require_activation();

        let caller = self.blockchain().get_caller();

        let is_allowlist_enabled = self.is_allowlist_enabled().get();
        if is_allowlist_enabled {
            require!(
                self.allowlist().contains(&caller),
                "The allowlist is enabled. Only eligible addresses can mint!"
            );
        }
        
        require!(paid_amount >= self.min_buy_limit().get(), "cannot buy less than min_buy_limit at once");
        require!(paid_amount <= self.max_buy_limit().get(), "cannot buy more than max_buy_limit at once");
        
        let bought_amount = self.bought_amount().get(&caller).unwrap_or_default();
        require!(&bought_amount + &paid_amount <= self.max_buy_limit().get(), "cannot buy more than max_buy_limit in total");

        let token_price = self.token_price().get();
        let token_id = self.token_id().get();
        let locked_token_id = self.locked_token_id().get();
        let available_token_amount = self.blockchain().get_sc_balance(&token_id, 0);
        let available_locked_token_amount = self.blockchain().get_sc_balance(&locked_token_id, 0);

        let token_amount = BigUint::from(1_000_000_000_000_000_000u64) * &paid_amount / &token_price * &BigUint::from(20u64) / &BigUint::from(100u64);
        let locked_token_amount = BigUint::from(1_000_000_000_000_000_000u64) * &paid_amount / &token_price * &BigUint::from(80u64) / &BigUint::from(100u64);
        require!(token_amount <= available_token_amount, "not enough tokens available");
        require!(locked_token_amount <= available_locked_token_amount, "not enough locked tokens available");

        self.send().direct(&caller, &token_id, 0, &token_amount, &[]);
        self.send().direct(&caller, &locked_token_id, 0, &locked_token_amount, &[]);

        self.bought_amount().insert(caller, &bought_amount + &paid_amount);

        Ok(())
    }


    /// private functions ///
    
    fn require_activation(&self) {
        require!(self.sale_started().get(), "sale is not started");
    }


    /// storage ///

    #[view(getTokenId)]
    #[storage_mapper("token_id")]
    fn token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getLockedTokenId)]
    #[storage_mapper("locked_token_id")]
    fn locked_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    // 1 ESDT price in EGLD-wei
    #[view(getTokenPrice)]
    #[storage_mapper("token_price")]
    fn token_price(&self) -> SingleValueMapper<BigUint>;

    // buy_limit in EGLD
    #[view(getMinBuyLimit)]
    #[storage_mapper("min_buy_limit")]
    fn min_buy_limit(&self) -> SingleValueMapper<BigUint>;

    #[view(getMaxBuyLimit)]
    #[storage_mapper("max_buy_limit")]
    fn max_buy_limit(&self) -> SingleValueMapper<BigUint>;

    #[view(isSaleStarted)]
    #[storage_mapper("sale_started")]
    fn sale_started(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("bought_amount")]
    fn bought_amount(&self) -> MapMapper<ManagedAddress, BigUint>;

    #[view(isAllowlistEnabled)]
    #[storage_mapper("is_allowlist_enabled")]
    fn is_allowlist_enabled(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("allowlist")]
    fn allowlist(&self) -> SetMapper<ManagedAddress>;
}
