use std::ops::DerefMut;
use scrypto::prelude::*;
use scrypto::prelude::rust::cmp::*;
use crate::common::*;
use crate::pool::pool::*;
use crate::hook_helpers::*;

// Metadata for the coin creator badge
static CREATOR_BADGE_NAME: &str = "Coin creator badge";

// Metadata for the flash loan transient NFT
static TRANSIENT_NFT_NAME: &str = "Flash loan transient NFT";

// Flash loan transient NFT data
#[derive(Debug, ScryptoSbor, NonFungibleData)]
struct FlashLoanData {
    coin_resource_address: ResourceAddress,
    coin_amount: Decimal,
    price: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct HookEnabledEvent {
    resource_address: Option<ResourceAddress>,
    hook_name: String,
    hook_address: HookInterfaceScryptoStub,
    operations: Vec<String>,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct HookDisabledEvent {
    resource_address: Option<ResourceAddress>,
    hook_name: String,
    hook_address: HookInterfaceScryptoStub,
    operations: Vec<String>,
}

#[derive(ScryptoSbor)]
struct PoolStruct {
    component_address: Global<Pool>,
    enabled_hooks: HooksPerOperation,
    creator_id: u64,
}

#[blueprint]
#[events(
    FairLaunchStartEvent,
    FairLaunchEndEvent,
    QuickLaunchEvent,
    RandomLaunchStartEvent,
    RandomLaunchEndEvent,
    BuyEvent,
    SellEvent,
    LiquidationEvent,
    FlashLoanEvent,
    BuyTicketEvent,
    FeeUpdateEvent,
    BurnEvent,
    AddLiquidityEvent,
    RemoveLiquidityEvent,
    HookEnabledEvent,
    HookDisabledEvent,
)]
#[types(
    u64,
    CreatorData,
    FlashLoanData,
    PoolStruct,
)]
mod radix_pump {
    enable_method_auth! {
        methods {
            forbid_symbols => restrict_to: [OWNER];
            forbid_names => restrict_to: [OWNER];
            new_fair_launch => PUBLIC;
            new_quick_launch => PUBLIC;
            new_random_launch => PUBLIC;
            buy => PUBLIC;
            sell => PUBLIC;
            get_fees => restrict_to: [OWNER];
            update_fees => restrict_to: [OWNER];
            owner_set_liquidation_mode => restrict_to: [OWNER];
            creator_set_liquidation_mode => PUBLIC;
            get_flash_loan => PUBLIC;
            return_flash_loan => PUBLIC;
            update_pool_fee_percentage => PUBLIC;
            get_pool_info => PUBLIC;
            update_time_limits => restrict_to: [OWNER];
            launch => PUBLIC;
            terminate_launch => PUBLIC;
            unlock => PUBLIC;
            register_hook => restrict_to: [OWNER];
            unregister_hook => restrict_to: [OWNER];
            owner_enable_hook => restrict_to: [OWNER];
            owner_disable_hook => restrict_to: [OWNER];
            creator_enable_hook => PUBLIC;
            creator_disable_hook => PUBLIC;
            burn => PUBLIC;
            buy_ticket => PUBLIC;
            redeem_ticket => PUBLIC;
            add_liquidity => PUBLIC;
            remove_liquidity => PUBLIC;
        }
    }

    enable_package_royalties! {
        new => Free;
        forbid_symbols => Free;
        forbid_names => Free;
        new_fair_launch => Usd(dec!("0.05"));
        new_quick_launch => Usd(dec!("0.05"));
        new_random_launch => Usd(dec!("0.05"));
        buy => Usd(dec!("0.005"));
        sell => Usd(dec!("0.005"));
        get_fees => Free;
        update_fees => Free;
        owner_set_liquidation_mode => Free;
        creator_set_liquidation_mode => Free;
        get_flash_loan => Usd(dec!("0.002"));
        return_flash_loan => Free;
        update_pool_fee_percentage => Free;
        get_pool_info => Free;
        update_time_limits => Free;
        launch => Free;
        terminate_launch => Free;
        unlock => Usd(dec!("0.005"));
        register_hook => Free;
        unregister_hook => Free;
        owner_enable_hook => Free;
        owner_disable_hook => Free;
        creator_enable_hook => Free;
        creator_disable_hook => Free;
        burn => Free;
        buy_ticket => Usd(dec!("0.005"));
        redeem_ticket => Free;
        add_liquidity => Free;
        remove_liquidity => Free;
    }

    struct RadixPump {
        owner_badge_address: ResourceAddress,
        base_coin_address: ResourceAddress,
        minimum_deposit: Decimal,
        creator_badge_resource_manager: ResourceManager,
        flash_loan_nft_resource_manager: ResourceManager,
        next_creator_badge_id: u64,
        last_transient_nft_id: u64,
        forbidden_symbols: KeyValueStore<String, u64>,
        forbidden_names: KeyValueStore<String, u64>,
        pools: KeyValueStore<ResourceAddress, PoolStruct>,
        creation_fee_percentage: Decimal,
        buy_sell_fee_percentage: Decimal,
        flash_loan_fee_percentage: Decimal,
        fee_vault: Vault,
        max_buy_sell_pool_fee_percentage: Decimal,
        max_flash_loan_pool_fee_percentage: Decimal,
        min_launch_duration: i64,
        min_lock_duration: i64,
        proxy_badge_vault: FungibleVault,
        hook_badge_vault: FungibleVault,
        read_only_hook_badge_vault: FungibleVault,
        registered_hooks: HookByName,
        registered_hooks_operations: HooksPerOperation,
        globally_enabled_hooks: HooksPerOperation,
    }

    impl RadixPump {

        // Component inteantiation
        pub fn new(
            owner_badge_address: ResourceAddress,
            base_coin_address: ResourceAddress,
            minimum_deposit: Decimal,
            creation_fee_percentage: Decimal,
            buy_sell_fee_percentage: Decimal,
            flash_loan_fee_percentage: Decimal,
        ) -> Global<RadixPump> {

            assert!(
                minimum_deposit > Decimal::ZERO,
                "Minimum deposit can't be zero or less",
            );
            assert!(
                creation_fee_percentage >= Decimal::ZERO && creation_fee_percentage < dec!(100),
                "Creation fee percentage can go from 0 (included) to 100 (excluded)",
            );
            assert!(
                buy_sell_fee_percentage >= Decimal::ZERO && buy_sell_fee_percentage < dec!(100),
                "Buy & sell fee percentage can go from 0 (included) to 100 (excluded)",
            );
            assert!(
                flash_loan_fee_percentage >= Decimal::ZERO && flash_loan_fee_percentage < dec!(100),
                "Flash loan fee percentage can go from 0 (included) to 100 (excluded)",
            );

            // Reserve a ComponentAddress for setting rules on resources
            let (address_reservation, component_address) = Runtime::allocate_component_address(RadixPump::blueprint_id());

            // Create a ResourceManager for minting coin creator badges
            let creator_badge_resource_manager = <scrypto::prelude::ResourceBuilder as RadixPumpResourceBuilder>::new_integer_non_fungible_with_registered_type::<CreatorData>(
                OwnerRole::Updatable(rule!(require(owner_badge_address)))
            )
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => CREATOR_BADGE_NAME, updatable;
                }
            ))
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(owner_badge_address));
            ))
            .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                non_fungible_data_updater => rule!(require(global_caller(component_address)));
                non_fungible_data_updater_updater => rule!(require(owner_badge_address));
            ))
            .burn_roles(burn_roles!(
                burner => rule!(allow_all);
                burner_updater => rule!(require(owner_badge_address));
            ))
            .create_with_no_initial_supply();

            // Create a ResourceManager for the flash loan transient NFT
            let flash_loan_nft_resource_manager = <scrypto::prelude::ResourceBuilder as RadixPumpResourceBuilder>::new_integer_non_fungible_with_registered_type::<FlashLoanData>(
                OwnerRole::Updatable(rule!(require(owner_badge_address)))
            )
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => TRANSIENT_NFT_NAME, updatable;
                }
            ))
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(deny_all);
            ))
            .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                non_fungible_data_updater => rule!(deny_all);
                non_fungible_data_updater_updater => rule!(deny_all);
            ))
            .burn_roles(burn_roles!(
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(deny_all);
            ))
            .deposit_roles(deposit_roles!(
                depositor => rule!(deny_all);
                depositor_updater => rule!(deny_all);
            ))
            .create_with_no_initial_supply();

            let hook_badge_bucket = ResourceBuilder::new_fungible(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .divisibility(0)
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => "Hook badge", updatable;
                }
            ))
            .mint_roles(mint_roles!(
                minter => rule!(deny_all);
                minter_updater => rule!(require(owner_badge_address));
            ))
            .burn_roles(burn_roles!(
                burner => rule!(deny_all);
                burner_updater => rule!(require(owner_badge_address));
            ))
            .mint_initial_supply(dec![1]);

            let read_only_hook_badge_bucket = ResourceBuilder::new_fungible(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .divisibility(0)
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => "RO hook badge", updatable;
                }
            ))
            .mint_roles(mint_roles!(
                minter => rule!(deny_all);
                minter_updater => rule!(require(owner_badge_address));
            ))
            .burn_roles(burn_roles!(
                burner => rule!(deny_all);
                burner_updater => rule!(require(owner_badge_address));
            ))
            .mint_initial_supply(dec![1]);

            let proxy_badge_bucket = ResourceBuilder::new_fungible(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .divisibility(0)
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => "Proxy badge", updatable;
                }
            ))
            .mint_roles(mint_roles!(
                minter => rule!(deny_all);
                minter_updater => rule!(require(owner_badge_address));
            ))
            .mint_initial_supply(dec![1]);

            // Instantiate the component
            Self {
                owner_badge_address: owner_badge_address,
                base_coin_address: base_coin_address.clone(),
                minimum_deposit: minimum_deposit,
                creator_badge_resource_manager: creator_badge_resource_manager,
                flash_loan_nft_resource_manager: flash_loan_nft_resource_manager,
                next_creator_badge_id: 1,
                last_transient_nft_id: 0,
                forbidden_symbols: KeyValueStore::new(),
                forbidden_names: KeyValueStore::new(),
                pools: KeyValueStore::new(),
                creation_fee_percentage: creation_fee_percentage,
                buy_sell_fee_percentage: buy_sell_fee_percentage,
                flash_loan_fee_percentage: flash_loan_fee_percentage,
                fee_vault: Vault::new(base_coin_address),
                max_buy_sell_pool_fee_percentage: dec!(10),
                max_flash_loan_pool_fee_percentage: dec!(10),
                min_launch_duration: 604800, // One week
                min_lock_duration: 7776000, // Three months
                proxy_badge_vault: FungibleVault::with_bucket(proxy_badge_bucket),
                hook_badge_vault: FungibleVault::with_bucket(hook_badge_bucket),
                read_only_hook_badge_vault: FungibleVault::with_bucket(read_only_hook_badge_bucket),
                registered_hooks: KeyValueStore::new(),
                registered_hooks_operations: HooksPerOperation::new(),
                globally_enabled_hooks: HooksPerOperation::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .with_address(address_reservation)
            .globalize()
        }

        // The component owner can prevent users to create coins with well known symbols
        pub fn forbid_symbols(
            &mut self,
            symbols: Vec<String>,
        ) {
            for symbol in symbols.iter() {
                self.forbidden_symbols.insert(symbol.trim().to_uppercase(), 0);
            }
        }

        // The component owner can prevent users to create coins with well known name
        pub fn forbid_names(
            &mut self,
            names: Vec<String>,
        ) {
            for name in names.iter() {
                self.forbidden_names.insert(name.trim().to_uppercase(), 0);
            }
        }

        fn check_fees(
            &mut self,
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee_percentage: Decimal,
        ) {
            assert!(
                buy_pool_fee_percentage >= Decimal::ZERO && buy_pool_fee_percentage < self.max_buy_sell_pool_fee_percentage,
                "Buy pool fee percentage can go from 0 (included) to {} (excluded)",
                self.max_buy_sell_pool_fee_percentage,
            );
            assert!(
                sell_pool_fee_percentage >= Decimal::ZERO && sell_pool_fee_percentage < self.max_buy_sell_pool_fee_percentage,
                "Sell pool fee percentage can go from 0 (included) to {} (excluded)",
                self.max_buy_sell_pool_fee_percentage,
            );
            assert!(
                flash_loan_pool_fee_percentage >= Decimal::ZERO && flash_loan_pool_fee_percentage < self.max_flash_loan_pool_fee_percentage,
                "Flash loan pool fee percentage can go from 0 (included) to {} (excluded)",
                self.max_flash_loan_pool_fee_percentage,
            );
        }

        fn check_metadata(
            &mut self,
            mut coin_symbol: String,
            mut coin_name: String,
            mut coin_icon_url: String,
            mut coin_info_url: String,
        ) -> (String, String, String, String) {
            coin_symbol = coin_symbol.trim().to_uppercase();
            assert!(
                coin_symbol.len() > 0,
                "Coin symbol can't be empty",
            );
            assert!(
                self.forbidden_symbols.get(&coin_symbol).is_none(),
                "Symbol already used",
            );
            self.forbidden_symbols.insert(coin_symbol.clone(), self.next_creator_badge_id);
            coin_name = coin_name.trim().to_string();
            assert!(
                coin_name.len() > 0,
                "Coin name can't be empty",
            );
            let uppercase_coin_name = coin_name.to_uppercase();
            assert!(
                self.forbidden_names.get(&uppercase_coin_name).is_none(),
                "Name already used",
            );
            self.forbidden_names.insert(uppercase_coin_name, self.next_creator_badge_id);
            coin_icon_url = coin_icon_url.trim().to_string();
            coin_info_url = coin_info_url.trim().to_string();

            (coin_symbol, coin_name, coin_icon_url, coin_info_url)
        }

        pub fn new_random_launch(
            &mut self,
            mut coin_symbol: String,
            mut coin_name: String,
            mut coin_icon_url: String,
            coin_description: String,
            mut coin_info_url: String,
            ticket_price: Decimal,
            winning_tickets: u32,
            coins_per_winning_ticket: Decimal, 
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee_percentage: Decimal,
        ) -> Bucket {
            self.check_fees(buy_pool_fee_percentage, sell_pool_fee_percentage, flash_loan_pool_fee_percentage);

            (coin_symbol, coin_name, coin_icon_url, coin_info_url) =
                self.check_metadata(coin_symbol, coin_name, coin_icon_url, coin_info_url);

            let (pool, coin_resource_address) = Pool::new_random_launch(
                self.owner_badge_address,
                self.proxy_badge_vault.resource_address(),
                self.hook_badge_vault.resource_address(),
                coin_symbol.clone(),
                coin_name.clone(),
                coin_icon_url,
                coin_description,
                coin_info_url,
                ticket_price,
                winning_tickets,
                coins_per_winning_ticket,
                buy_pool_fee_percentage,
                sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage,
                self.next_creator_badge_rule(),
                self.base_coin_address,
            );
            self.pools.insert(
                coin_resource_address,
                PoolStruct {
                    component_address: pool,
                    enabled_hooks: HooksPerOperation::new(),
                    creator_id: self.next_creator_badge_id,
                }
            );

            self.mint_creator_badge(
                coin_resource_address,
                coin_name,
                coin_symbol,
                PoolMode::WaitingForLaunch,
            )
        }

        pub fn new_fair_launch(
            &mut self,
            mut coin_symbol: String,
            mut coin_name: String,
            mut coin_icon_url: String,
            coin_description: String,
            mut coin_info_url: String,
            launch_price: Decimal,
            creator_locked_percentage: Decimal,
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee_percentage: Decimal,
        ) -> Bucket {
            self.check_fees(buy_pool_fee_percentage, sell_pool_fee_percentage, flash_loan_pool_fee_percentage);

            (coin_symbol, coin_name, coin_icon_url, coin_info_url) =
                self.check_metadata(coin_symbol, coin_name, coin_icon_url, coin_info_url);

            let (pool, coin_resource_address) = Pool::new_fair_launch(
                self.owner_badge_address,
                self.proxy_badge_vault.resource_address(),
                self.hook_badge_vault.resource_address(),
                coin_symbol.clone(),
                coin_name.clone(),
                coin_icon_url,
                coin_description,
                coin_info_url,
                launch_price,
                creator_locked_percentage,
                buy_pool_fee_percentage,
                sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage,
                self.next_creator_badge_rule(),
                self.base_coin_address,
            );
            self.pools.insert(
                coin_resource_address,
                PoolStruct {
                    component_address: pool,
                    enabled_hooks: HooksPerOperation::new(),
                    creator_id: self.next_creator_badge_id,
                }
            );

            self.mint_creator_badge(
                coin_resource_address,
                coin_name,
                coin_symbol,
                PoolMode::WaitingForLaunch,
            )
        }

        pub fn new_quick_launch(
            &mut self,
            mut base_coin_bucket: Bucket,
            mut coin_symbol: String,
            mut coin_name: String,
            mut coin_icon_url: String,
            coin_description: String,
            mut coin_info_url: String,
            coin_supply: Decimal,
            coin_price: Decimal,
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee_percentage: Decimal,
        ) -> (Bucket, Bucket, Vec<Bucket>) {
            assert!(
                base_coin_bucket.resource_address() == self.base_coin_address,
                "Wrong base coin deposited",
            );
            assert!(
                base_coin_bucket.amount() >= self.minimum_deposit,
                "Insufficient base coin deposit",
            );

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    self.creation_fee_percentage * base_coin_bucket.amount() / 100,
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            self.check_fees(buy_pool_fee_percentage, sell_pool_fee_percentage, flash_loan_pool_fee_percentage);

            (coin_symbol, coin_name, coin_icon_url, coin_info_url) =
                self.check_metadata(coin_symbol, coin_name, coin_icon_url, coin_info_url);

            let (pool, creator_coin_bucket, hook_argument, event) = Pool::new_quick_launch(
                self.owner_badge_address,
                self.proxy_badge_vault.resource_address(),
                self.hook_badge_vault.resource_address(),
                base_coin_bucket,
                coin_symbol.clone(),
                coin_name.clone(),
                coin_icon_url,
                coin_description,
                coin_info_url,
                coin_supply,
                coin_price,
                buy_pool_fee_percentage,
                sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage,
                self.next_creator_badge_rule(),
            );

            self.emit_pool_event(event);

            let coin_address = creator_coin_bucket.resource_address();
            self.pools.insert(
                coin_address,
                PoolStruct {
                    component_address: pool,
                    enabled_hooks: HooksPerOperation::new(),
                    creator_id: self.next_creator_badge_id,
                }
            );

            let creator_badge_bucket = self.mint_creator_badge(
                creator_coin_bucket.resource_address(),
                coin_name,
                coin_symbol,
                PoolMode::Normal,
            );

            let buckets = self.execute_hooks(
                &vec![vec![],vec![],vec![]],
                &hook_argument,
            );

            (creator_badge_bucket, creator_coin_bucket, buckets)
        }

        fn emit_pool_event(
            &self,
            event: AnyPoolEvent,
        ) {
            match event {
                AnyPoolEvent::FairLaunchStartEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::FairLaunchEndEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::QuickLaunchEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::RandomLaunchStartEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::RandomLaunchEndEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::BuyEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::SellEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::LiquidationEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::FlashLoanEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::BuyTicketEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::FeeUpdateEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::BurnEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::AddLiquidityEvent(event) => Runtime::emit_event(event),
                AnyPoolEvent::RemoveLiquidityEvent(event) => Runtime::emit_event(event),
            }
        }

        pub fn buy(
            &mut self,
            coin_address: ResourceAddress,
            mut base_coin_bucket: Bucket,
        ) -> (Bucket, Vec<Bucket>) {
            assert!(
                base_coin_bucket.resource_address() == self.base_coin_address,
                "Wrong base coin",
            );

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    base_coin_bucket.amount() * self.buy_sell_fee_percentage / dec!(100),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            let pool = self.pools.get(&coin_address).expect("Coin not found");

            let (coin_bucket, hook_argument, event) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.buy(base_coin_bucket)
            );

            self.emit_pool_event(event);

            let pool_enabled_hooks = pool.enabled_hooks.get_all_hooks(hook_argument.operation);
            drop(pool);
            let buckets = self.execute_hooks(
                &pool_enabled_hooks,
                &hook_argument,
            );

            (coin_bucket, buckets)
        }

        pub fn sell(
            &mut self,
            coin_bucket: Bucket,
        ) -> (Bucket, Vec<Bucket>) {
            let coin_address = coin_bucket.resource_address();
            let pool = self.pools.get(&coin_address).expect("Coin not found");

            let (mut base_coin_bucket, hook_argument, event) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.sell(coin_bucket)
            );

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    base_coin_bucket.amount() * self.buy_sell_fee_percentage / dec!(100),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            self.emit_pool_event(event);

            let pool_enabled_hooks = pool.enabled_hooks.get_all_hooks(hook_argument.operation);
            drop(pool);
            let buckets = self.execute_hooks(
                &pool_enabled_hooks,
                &hook_argument,
            );

            (base_coin_bucket, buckets)
        }

        pub fn get_fees(
            &mut self,
        ) -> Bucket {
            self.fee_vault.take_all()
        }

        pub fn update_fees(
            &mut self,
            creation_fee_percentage: Decimal,
            buy_sell_fee_percentage: Decimal,
            flash_loan_fee_percentage: Decimal,
            max_buy_sell_pool_fee_percentage: Decimal,
            max_flash_loan_pool_fee_percentage: Decimal,
        ) {
            assert!(
                creation_fee_percentage >= Decimal::ZERO && creation_fee_percentage < dec!(100),
                "Creation fee percentage can go from 0 (included) to 100 (excluded)",
            );  
            assert!(
                buy_sell_fee_percentage >= Decimal::ZERO && buy_sell_fee_percentage < dec!(100),
                "Buy & sell fee percentage can go from 0 (included) to 100 (excluded)",
            );
            assert!(
                flash_loan_fee_percentage >= Decimal::ZERO && flash_loan_fee_percentage < dec!(100),
                "Flash loan fee percentage can go from 0 (included) to 100 (excluded)",
            );
            assert!(
                max_buy_sell_pool_fee_percentage >= Decimal::ZERO && max_buy_sell_pool_fee_percentage <= dec!(100),
                "Max buy sell pool fee percentage can go from 0 (included) to 100 (included)",
            );
            assert!(
                max_flash_loan_pool_fee_percentage >= Decimal::ZERO && max_flash_loan_pool_fee_percentage <= dec!(100),
                "Max flash loan pool fee percentage can go from 0 (included) to 100 (included)",
            );

            self.creation_fee_percentage = creation_fee_percentage;
            self.buy_sell_fee_percentage = buy_sell_fee_percentage;
            self.flash_loan_fee_percentage = flash_loan_fee_percentage;
            self.max_buy_sell_pool_fee_percentage = max_buy_sell_pool_fee_percentage;
            self.max_flash_loan_pool_fee_percentage = max_flash_loan_pool_fee_percentage;
        }

        pub fn owner_set_liquidation_mode(
            &mut self,
            coin_address: ResourceAddress,
        ) {
            let pool = self.pools.get(&coin_address).expect("Coin not found");
            let (mode, event) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.set_liquidation_mode()
            );

            self.emit_pool_event(event);

            self.creator_badge_resource_manager.update_non_fungible_data(
                &NonFungibleLocalId::integer(pool.creator_id.into()),
                "pool_mode",
                mode,
            );
        }

        pub fn creator_set_liquidation_mode(
            &mut self,
            creator_proof: Proof,
        ) {
            let creator_data = self.get_creator_data(creator_proof);

            let pool = self.pools.get(&creator_data.coin_resource_address).expect("Coin not found");
            let (mode, event) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.set_liquidation_mode()
            );

            self.emit_pool_event(event);

            self.creator_badge_resource_manager.update_non_fungible_data(
                &NonFungibleLocalId::integer(creator_data.id.into()),
                "pool_mode",
                mode,
            );
        }

        pub fn get_flash_loan(
            &mut self,
            coin_address: ResourceAddress,
            amount: Decimal
        ) -> (Bucket, Bucket) {
            let pool = self.pools.get_mut(&coin_address).expect("Coin not found");
            let (coin_bucket, price) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.get_flash_loan(amount)
            );

            self.last_transient_nft_id += 1;

            let transient_nft_bucket = self.flash_loan_nft_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.last_transient_nft_id),
                FlashLoanData {
                    coin_resource_address: coin_address,
                    coin_amount: amount,
                    price: price,
                }
            );

            (coin_bucket, transient_nft_bucket)
        }

        pub fn return_flash_loan(
            &mut self,
            transient_nft_bucket: Bucket,
            mut base_coin_bucket: Bucket,
            coin_bucket: Bucket,
        ) -> Vec<Bucket> {
            assert!(
                transient_nft_bucket.resource_address() == self.flash_loan_nft_resource_manager.address(),
                "Wrong NFT",
            );
            assert!(
                base_coin_bucket.resource_address() == self.base_coin_address,
                "Wrong base coin",
            );

            let flash_loan_data = transient_nft_bucket.as_non_fungible().non_fungible::<FlashLoanData>().data();
            assert!(
                flash_loan_data.coin_resource_address == coin_bucket.resource_address(),
                "Wrong coin",
            );
            assert!(
                flash_loan_data.coin_amount <= coin_bucket.amount(),
                "Not enough coins",
            );

            transient_nft_bucket.burn();

            let pool = self.pools.get(&coin_bucket.resource_address()).unwrap();

            // In order to avoid price manipulation affecting the fees, take the maximum among the
            // price at the moment the flash loan was granted and the current price.
            let mut price = pool.component_address.get_pool_info().last_price;
            price = max(price, flash_loan_data.price);

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    flash_loan_data.coin_amount * price * self.flash_loan_fee_percentage / dec!(100),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            let (hook_argument, event) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.return_flash_loan(
                    base_coin_bucket,
                    coin_bucket,
                    price,
                )
            );

            self.emit_pool_event(event);

            let pool_enabled_hooks = pool.enabled_hooks.get_all_hooks(hook_argument.operation);
            drop(pool);
            self.execute_hooks(
                &pool_enabled_hooks,
                &hook_argument,
            )
        }

        pub fn update_pool_fee_percentage(
            &mut self,
            creator_proof: Proof,
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee_percentage: Decimal,
        ) {
            self.check_fees(
                buy_pool_fee_percentage,
                sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage,
            );

            let creator_data = self.get_creator_data(creator_proof);

            let pool = self.pools.get(&creator_data.coin_resource_address).unwrap();

            let event = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.update_pool_fee_percentage(
                    buy_pool_fee_percentage,
                    sell_pool_fee_percentage,
                    flash_loan_pool_fee_percentage,
                )
            );

            self.emit_pool_event(event);
        }

        pub fn get_pool_info(
            &self,
            coin_address: ResourceAddress,
        ) -> PoolInfo {
            let pool = self.pools.get(&coin_address).expect("Coin not found");
            let mut pool_info = pool.component_address.get_pool_info();

            pool_info.total_buy_fee_percentage = dec!(1000000) / ((100 - pool_info.total_buy_fee_percentage) * (100 - self.buy_sell_fee_percentage)) - dec!(100);
            pool_info.total_sell_fee_percentage = pool_info.total_sell_fee_percentage + self.buy_sell_fee_percentage * (100 - pool_info.total_sell_fee_percentage) / dec!(100);
            pool_info.total_flash_loan_fee_percentage = pool_info.total_flash_loan_fee_percentage + self.flash_loan_fee_percentage;
            pool_info.flash_loan_nft_resource_address = Some(self.flash_loan_nft_resource_manager.address());
            pool_info.hooks_badge_resource_address = Some(self.hook_badge_vault.resource_address());
            pool_info.read_only_hooks_badge_resource_address = Some(self.read_only_hook_badge_vault.resource_address());

            pool_info
        }

        fn get_creator_data(
            &self,
            creator_proof: Proof
        ) -> CreatorData {
            creator_proof.check_with_message(
                self.creator_badge_resource_manager.address(),
                "Wrong badge",
            )
            .as_non_fungible()
            .non_fungible::<CreatorData>()
            .data()
        }

        pub fn update_time_limits(
            &mut self,
            min_launch_duration: i64,
            min_lock_duration: i64,
        ) {
            assert!(
                min_launch_duration > 0,
                "Min launch duration must be bigger than zero",
            );
            self.min_launch_duration = min_launch_duration;

            assert!(
                min_lock_duration > 0,
                "Min lock duration must be bigger than zero",
            );
            self.min_lock_duration = min_lock_duration;
        }

        pub fn launch(
            &mut self,
            creator_proof: Proof,
            end_launch_time: i64,
            unlocking_time: i64,
        ) -> Vec<Bucket> {

            assert!(
                end_launch_time >= Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch + self.min_launch_duration,
                "Launch time too short",
            );
            assert!(
                unlocking_time >= end_launch_time + self.min_lock_duration,
                "Lock time too short",
            );

            let creator_data = self.get_creator_data(creator_proof);
            let coin_address = creator_data.coin_resource_address;
            let pool = self.pools.get(&coin_address).unwrap();

            let (mode, hook_argument, event) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.launch(end_launch_time, unlocking_time)
            );

            self.emit_pool_event(event);

            self.creator_badge_resource_manager.update_non_fungible_data(
                &NonFungibleLocalId::integer(creator_data.id.into()),
                "pool_mode",
                mode,
            );

            let pool_enabled_hooks = pool.enabled_hooks.get_all_hooks(hook_argument.operation);
            drop(pool);
            self.execute_hooks(
                &pool_enabled_hooks,
                &hook_argument,
            )
        }

        fn execute_hooks(
            &mut self,
            pool_enabled_hooks: &Vec<Vec<String>>,
            hook_argument: &HookArgument,
        ) -> Vec<Bucket> {
            let mut additional_buckets: Vec<Bucket> = vec![];

            let mut hook_badge_bucket = self.hook_badge_vault.take(dec!(1));
            let mut the_badge_i_gave_you = hook_badge_bucket.resource_address();

            let mut additional_operations_round: Vec<Vec<(HookArgument, HookInterfaceScryptoStub)>> = vec![vec![],vec![],vec![]];

            for execution_round in 0..3 {
                let merged_hooks = self.globally_enabled_hooks.merge(
                    hook_argument.operation,
                    &pool_enabled_hooks[execution_round],
                    execution_round,
                );

                let registered_hooks_per_operation =
                    self.registered_hooks_operations.get_hooks(hook_argument.operation, execution_round);

                for hook in merged_hooks.iter() {
                    let hook_info = self.registered_hooks.get_mut(&hook);

                    // Ignore hoooks that have been unregistered by the componet owner; do not panic
                    if hook_info.is_none() || !registered_hooks_per_operation.iter().any(|x| x == hook) {
                        continue;
                    }

                    let (
                        temp_badge_bucket,
                        opt_bucket,
                        events,
                        hook_arguments,
                    ) = hook_info.unwrap().deref_mut().component_address.hook(
                        hook_argument.clone(),
                        hook_badge_bucket,
                    );
                    assert!(
                        temp_badge_bucket.resource_address() == the_badge_i_gave_you && temp_badge_bucket.amount() == Decimal::ONE,
                        "Hey hook, where's my badge gone?",
                    );
                    hook_badge_bucket = temp_badge_bucket;

                    // An hook can generate any number of Pool events by calling Pool methods
                    for event in events.iter() {
                        self.emit_pool_event(event.clone());
                    }

                    // An hook can return a Bucket for the user
                    match opt_bucket {
                        None => {},
                        Some(bucket) => additional_buckets.push(bucket),
                    }

                    match execution_round {
                        0 => {
                            // A round 0 hook can recursively trigger the execution of other hooks
                            for argument in hook_arguments.iter() {
                                // An hook executed on a pool can also trigger hooks on different
                                // pools!
                                let pool2 = self.pools.get(&argument.coin_address);
                                match pool2 {
                                    None => {},
                                    Some(pool2) => {
                                        let pool2_enabled_hooks = pool2.enabled_hooks.get_all_hooks(argument.operation);
                                        // For execution rounds 2 and 3
                                        for execution_round2 in 1..3 {
                                            // Get all of the hook enabled for the operation
                                            // globally or for the pool
                                            let merged_hooks = self.globally_enabled_hooks.merge(
                                                argument.operation,
                                                &pool2_enabled_hooks[execution_round2],
                                                execution_round2,
                                            );
                                            for hook2 in merged_hooks.iter() {
                                                let hook2_info = self.registered_hooks.get(&hook2);
                                                // Select only the registered hooks that allow
                                                // recursion
                                                match hook2_info {
                                                    None => {},
                                                    Some(hook2_info) => {
                                                        if !hook2_info.allow_recursion || hook2_info.round == 0 {
                                                            continue;
                                                        }
                                                        // Put them into an array for later use
                                                        additional_operations_round[hook2_info.round].push(
                                                            (argument.clone(), hook2_info.component_address)
                                                        );
                                                    },
                                                }
                                            }
                                        }
                                    },
                                }
                            }
                        },
                        1 | 2 => {
                            for op in additional_operations_round[execution_round].iter_mut() {
                                let (
                                    temp_badge_bucket,
                                    opt_bucket,
                                    events,
                                    _,
                                ) = op.1.hook(
                                    op.0.clone(),
                                    hook_badge_bucket,
                                );
                                assert!(
                                    temp_badge_bucket.resource_address() == the_badge_i_gave_you && temp_badge_bucket.amount() == Decimal::ONE,
                                    "Hey hook, where's my badge gone?",
                                );
                                hook_badge_bucket = temp_badge_bucket;

                                // An hook can generate any number of Pool events by calling Pool methods
                                for event in events.iter() {
                                    self.emit_pool_event(event.clone());
                                }

                                // An hook can return a Bucket for the user
                                match opt_bucket {
                                    None => {},
                                    Some(bucket) => additional_buckets.push(bucket),
                                }
                            }
                        },
                        _ => {},
                    }
                }

                // At the end of round 1 switch to the read only badge
                if execution_round == 1 {
                    self.hook_badge_vault.put(hook_badge_bucket);
                    hook_badge_bucket = self.read_only_hook_badge_vault.take(dec!(1));
                    the_badge_i_gave_you = hook_badge_bucket.resource_address();
                }
            }

            self.read_only_hook_badge_vault.put(hook_badge_bucket);

            additional_buckets
        }

        pub fn terminate_launch(
            &mut self,
            creator_proof: Proof,
        ) -> (Option<Bucket>, Option<Vec<Bucket>>) {
            let creator_data = self.get_creator_data(creator_proof);
            let coin_address = creator_data.coin_resource_address;

            let pool = self.pools.get(&coin_address).unwrap();

            let (mut bucket, mode, hook_argument, event) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.terminate_launch()
            );

            if event.is_some() {
                self.emit_pool_event(event.unwrap());
            }

            if mode.is_some() {
                self.creator_badge_resource_manager.update_non_fungible_data(
                    &NonFungibleLocalId::integer(creator_data.id.into()),
                    "pool_mode",
                    mode.unwrap(),
                );
            }

            match bucket {
                None => {},
                Some(ref mut bucket) => self.fee_vault.put(
                    bucket.take_advanced(
                        self.creation_fee_percentage * bucket.amount() / 100,
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    )
                ),
            }

            let buckets = match hook_argument {
                None => None,
                Some(hook_argument) => {
                    let pool_enabled_hooks = pool.enabled_hooks.get_all_hooks(hook_argument.operation);
                    drop(pool);
                    Some(
                        self.execute_hooks(
                            &pool_enabled_hooks,
                            &hook_argument,
                        )
                    )
                },
            };

            (bucket, buckets)
        }

        fn mint_creator_badge(
            &mut self,
            coin_resource_address: ResourceAddress,
            coin_name: String,
            coin_symbol: String,
            pool_mode: PoolMode
        ) -> Bucket {
            let creator_badge = self.creator_badge_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.next_creator_badge_id.into()),
                CreatorData {
                    id: self.next_creator_badge_id,
                    coin_resource_address: coin_resource_address,
                    coin_name: coin_name,
                    coin_symbol: coin_symbol,
                    creation_date: Clock::current_time_rounded_to_seconds(),
                    pool_mode: pool_mode,
                }
            );

            self.next_creator_badge_id += 1;

            creator_badge
        }

        fn next_creator_badge_rule(&mut self) -> AccessRuleNode {
            AccessRuleNode::ProofRule(
                ProofRule::Require (
                    ResourceOrNonFungible::NonFungible (
                        NonFungibleGlobalId::new(
                            self.creator_badge_resource_manager.address(),
                            NonFungibleLocalId::integer(self.next_creator_badge_id.into()),
                        )
                    )
                )
            )
        }

        pub fn unlock(
            &mut self,
            creator_proof: Proof,
            amount: Option<Decimal>,
            sell: bool,
        ) -> (Bucket, Vec<Bucket>) {
            let coin_address = self.get_creator_data(creator_proof).coin_resource_address;
            let pool = self.pools.get(&coin_address).unwrap();

            let coin_bucket = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.unlock(amount)
            );

            match sell {
                false => (coin_bucket, vec![]),
                true => {
                    let (mut base_coin_bucket, hook_argument, event) =
                        self.proxy_badge_vault.authorize_with_amount(
                            1,
                            || pool.component_address.sell(coin_bucket)
                        );

                    self.fee_vault.put(
                        base_coin_bucket.take_advanced(
                            base_coin_bucket.amount() * self.buy_sell_fee_percentage / dec!(100),
                            WithdrawStrategy::Rounded(RoundingMode::ToZero),
                        )
                     );

                    self.emit_pool_event(event);

                    let pool_enabled_hooks = pool.enabled_hooks.get_all_hooks(hook_argument.operation);
                    drop(pool);
                    let buckets = self.execute_hooks(
                        &pool_enabled_hooks,
                        &hook_argument,
                    );

                    (base_coin_bucket, buckets)
                },
            }
        }

        pub fn register_hook(
            &mut self,
            name: String,
            operations: Vec<String>,
            component_address: HookInterfaceScryptoStub,
        ) {
            let (round, allow_recursion) = component_address.get_hook_info();
            assert!(
                round < 3,
                "Non existent round",
            );
            assert!(
                round != 0 || !allow_recursion,
                "Round 0 hooks can't be called recursively",
            );

            self.registered_hooks_operations.add_hook(
                &name,
                &operations,
                round
            );

            self.registered_hooks.insert(
                name,
                HookInfo {
                    component_address: component_address,
                    round: round,
                    allow_recursion: allow_recursion,
                },
            );
        }

        pub fn unregister_hook(
            &mut self,
            name: String,
            operations: Option<Vec<String>>,
        ) {
            let hook_info = self.registered_hooks.get(&name);
            match hook_info {
                None => {},

                Some(hook_info) => {
                    match operations {
                        None => {
                            self.registered_hooks.remove(&name);
                        },

                        Some(operations) =>
                            self.registered_hooks_operations.remove_hook(
                                &name,
                                &operations,
                                hook_info.round,
                            ),
                    }
                },
            }
        }

        pub fn owner_enable_hook(
            &mut self,
            name: String,
            operations: Vec<String>,
        ) {
            let hook_info = self.registered_hooks.get(&name).expect("Unknown hook");

            for operation in operations.iter() {
                assert!(
                    self.registered_hooks_operations.hook_exists(
                        &name,
                        &operation,
                        hook_info.round,
                    ),
                    "Hook {} not registered for operation {}",
                    name,
                    operation,
                );
            }

            self.globally_enabled_hooks.add_hook(
                &name,
                &operations,
                hook_info.round,
            );

            Runtime::emit_event(
                HookEnabledEvent {
                    resource_address: None,
                    hook_name: name,
                    hook_address: hook_info.component_address,
                    operations: operations,
                }
            );
        }

        pub fn owner_disable_hook(
            &mut self,
            name: String,
            operations: Vec<String>,
        ) {
            let hook_info = self.registered_hooks.get(&name).expect("Unknown hook");

            self.globally_enabled_hooks.remove_hook(
                &name,
                &operations,
                hook_info.round,
            );

            Runtime::emit_event(
                HookDisabledEvent {
                    resource_address: None,
                    hook_name: name,
                    hook_address: hook_info.component_address,
                    operations: operations,
                }
            );
        }

        pub fn creator_enable_hook(
            &mut self,
            creator_proof: Proof,
            name: String,
            operations: Vec<String>,
        ) {
            let hook_info = self.registered_hooks.get(&name).expect("Unknown hook");

            for operation in operations.iter() {
                assert!(
                    self.registered_hooks_operations.hook_exists(
                        &name,
                        &operation,
                        hook_info.round,
                    ),
                    "Hook {} not registered for operation {}",
                    name,
                    operation,
                );
            }

            let coin_address = self.get_creator_data(creator_proof).coin_resource_address;

            self.pools.get_mut(&coin_address).unwrap().enabled_hooks.add_hook(
                &name,
                &operations,
                hook_info.round,
            );
            
            Runtime::emit_event(
                HookEnabledEvent {
                    resource_address: Some(coin_address),
                    hook_name: name,
                    hook_address: hook_info.component_address,
                    operations: operations,
                }
            );
        }

        pub fn creator_disable_hook(
            &mut self,
            creator_proof: Proof,
            name: String,
            operations: Vec<String>,
        ) {
            let hook_info = self.registered_hooks.get(&name).expect("Unknown hook");

            let coin_address = self.get_creator_data(creator_proof).coin_resource_address;

            self.pools.get_mut(&coin_address).unwrap().enabled_hooks.remove_hook(
                &name,
                &operations,
                hook_info.round,
            );

            Runtime::emit_event(
                HookDisabledEvent {
                    resource_address: Some(coin_address),
                    hook_name: name,
                    hook_address: hook_info.component_address,
                    operations: operations,
                }
            );
        }

        pub fn burn(
            &mut self,
            creator_proof: Proof,
            amount: Decimal,
        ) {
            let coin_address = self.get_creator_data(creator_proof).coin_resource_address;
            let pool = self.pools.get(&coin_address).unwrap();

            let event = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.burn(amount)
            );

            self.emit_pool_event(event);
        }

        pub fn buy_ticket(
            &mut self,
            coin_address: ResourceAddress,
            amount: u32,
            mut base_coin_bucket: Bucket,
        ) -> (Bucket, Vec<Bucket>) {
            assert!(
                base_coin_bucket.resource_address() == self.base_coin_address,
                "Wrong base coin",
            ); 
            assert!(
                amount > 0,
                "Can't buy zero tickets",
            );

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    base_coin_bucket.amount() * self.buy_sell_fee_percentage / dec!(100),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            let pool = self.pools.get(&coin_address).expect("Coin not found");

            let (ticket_bucket, hook_argument, event) =
                self.proxy_badge_vault.authorize_with_amount(
                    1,
                    || pool.component_address.buy_ticket(
                        amount,
                        base_coin_bucket
                    )
                );

            self.emit_pool_event(event);

            let pool_enabled_hooks = pool.enabled_hooks.get_all_hooks(hook_argument.operation);
            drop(pool);
            let buckets = self.execute_hooks(
                &pool_enabled_hooks,
                &hook_argument,
            );

            (ticket_bucket, buckets)
        }

        pub fn redeem_ticket(
            &mut self,
            ticket_bucket: Bucket,
        ) -> (
            Bucket,
            Option<Bucket>,
            Option<Vec<Bucket>>,
            Option<Vec<Bucket>>,
        ) {
            let ticket_id = &ticket_bucket.as_non_fungible().non_fungible_local_ids()[0];
            let ticket_data = ResourceManager::from_address(ticket_bucket.resource_address()).get_non_fungible_data::<TicketData>(ticket_id);
            let pool = self.pools.get(&ticket_data.coin_resource_address).expect("Coin not found");

            let (base_coin_bucket, coin_bucket, hook_argument_lose, hook_argument_win) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.redeem_ticket(ticket_bucket)
            );

            let pool_enabled_hooks_lose = match hook_argument_lose {
                None => None,
                Some(ref hook_argument) => Some(pool.enabled_hooks.get_all_hooks(hook_argument.operation)),
            };
            let pool_enabled_hooks_win = match hook_argument_win {
                None => None,
                Some(ref hook_argument) => Some(pool.enabled_hooks.get_all_hooks(hook_argument.operation)),
            };

            drop(pool);

            let lose_buckets = match hook_argument_lose {
                None => None,
                Some(hook_argument) => Some(
                    self.execute_hooks(
                        &pool_enabled_hooks_lose.unwrap(),
                        &hook_argument,
                    )
                ),
            };

            let win_buckets = match hook_argument_win {
                None => None,
                Some(hook_argument) => Some(
                    self.execute_hooks(
                        &pool_enabled_hooks_win.unwrap(),
                        &hook_argument,
                    )
                ),
            };

            (base_coin_bucket, coin_bucket, lose_buckets, win_buckets)
        }

        pub fn add_liquidity(
            &mut self,
            base_coin_bucket: Bucket,
            coin_bucket: Bucket,
        ) -> (
            Bucket,
            Bucket,
            Vec<Bucket>,
        ) {
            let coin_address = coin_bucket.resource_address();
            let pool = self.pools.get(&coin_address).expect("Coin not found");

            let (lp_bucket, remainings_bucket, hook_argument, event) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.add_liquidity(
                    base_coin_bucket,
                    coin_bucket,
                )
            );

            self.emit_pool_event(event);

            let pool_enabled_hooks = pool.enabled_hooks.get_all_hooks(hook_argument.operation);
            drop(pool);
            let buckets = self.execute_hooks(
                &pool_enabled_hooks,
                &hook_argument,
            );

            (lp_bucket, remainings_bucket, buckets)
        }

        pub fn remove_liquidity(
            &mut self,
            lp_bucket: Bucket,
        ) -> (
            Bucket,
            Option<Bucket>,
            Vec<Bucket>,
        ) {
            let lp_id = &lp_bucket.as_non_fungible().non_fungible_local_ids()[0];
            let lp_data = ResourceManager::from_address(lp_bucket.resource_address()).get_non_fungible_data::<LPData>(lp_id);
            let pool = self.pools.get(&lp_data.coin_resource_address).expect("Coin not found");

            let (base_coin_bucket, coin_bucket, hook_argument, event) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || pool.component_address.remove_liquidity(lp_bucket)
            );

            self.emit_pool_event(event);

            let pool_enabled_hooks = pool.enabled_hooks.get_all_hooks(hook_argument.operation);
            drop(pool);
            let buckets = self.execute_hooks(
                &pool_enabled_hooks,
                &hook_argument,
            );

            (base_coin_bucket, coin_bucket, buckets)
        }
    }
}
