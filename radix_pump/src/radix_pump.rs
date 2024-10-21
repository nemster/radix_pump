use std::ops::Deref;
use scrypto::prelude::*;
use scrypto::prelude::rust::cmp::*;
use crate::common::*;
use crate::pool::*;
use crate::hook::*;
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
pub struct HookEnabledEvent {
    resource_address: Option<ResourceAddress>,
    hook_name: String,
    hook_address: HookInterfaceScryptoStub,
    operations: Vec<String>,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct HookDisabledEvent {
    resource_address: Option<ResourceAddress>,
    hook_name: String,
    hook_address: HookInterfaceScryptoStub,
    operations: Vec<String>,
}

#[blueprint]
#[events(
    FairLaunchStartEvent,
    FairLaunchEndEvent,
    QuickLaunchEvent,
    RandomLaunchStartEvent,
    BuyEvent,
    SellEvent,
    LiquidationEvent,
    FlashLoanEvent,
    HookEnabledEvent,
    HookDisabledEvent,
    BuyTicketEvent,
)]
#[types(
    u64,
    CreatorData,
    FlashLoanData,
)]
mod radix_pump {

    extern_blueprint!(
        "package_sim1pk3cmat8st4ja2ms8mjqy2e9ptk8y6cx40v4qnfrkgnxcp2krkpr92",
        RandomComponent {
            fn request_random(
                &self, address: ComponentAddress,
                method_name: String,
                on_error: String,
                key: u32,
                badge_opt:
                Option<FungibleBucket>,
                expected_fee: u8
            ) -> u32;
        }
    );
    const RNG: Global<RandomComponent> = global_component!(
        RandomComponent,
        "component_sim1crmulhl5yrk6hh4jsyldps5sdrp08r5v9wusupvzxgqvhlp4k00px7"
    );

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
            random_callback => PUBLIC;
            random_on_error => PUBLIC;
            redeem_ticket => PUBLIC;
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
        random_callback => Free;
        random_on_error => Free;
        redeem_ticket => Free;
    }

    struct RadixPump {
        base_coin_address: ResourceAddress,
        minimum_deposit: Decimal,
        creator_badge_resource_manager: ResourceManager,
        flash_loan_nft_resource_manager: ResourceManager,
        next_creator_badge_id: u64,
        last_transient_nft_id: u64,
        forbidden_symbols: KeyValueStore<String, u64>,
        forbidden_names: KeyValueStore<String, u64>,
        pools: KeyValueStore<ResourceAddress, Pool>,
        creation_fee_percentage: Decimal,
        buy_sell_fee_percentage: Decimal,
        flash_loan_fee_percentage: Decimal,
        fee_vault: Vault,
        max_buy_sell_pool_fee_percentage: Decimal,
        max_flash_loan_pool_fee_percentage: Decimal,
        min_launch_duration: i64,
        min_lock_duration: i64,
        hooks_badge_vault: Vault,
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

            let hooks_badge_bucket = ResourceBuilder::new_fungible(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .divisibility(0)
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => "Hooks badge", updatable;
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

            // Instantiate the component
            Self {
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
                hooks_badge_vault: Vault::with_bucket(hooks_badge_bucket.into()),
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
                self.next_creator_badge_id,
            );
            self.pools.insert(
                coin_resource_address,
                pool,
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
                self.next_creator_badge_id,
            );
            self.pools.insert(
                coin_resource_address,
                pool,
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

            let (pool, creator_coin_bucket) = Pool::new_quick_launch(
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
                self.next_creator_badge_id,
            );
            let coin_address = creator_coin_bucket.resource_address();
            self.pools.insert(
                coin_address,
                pool,
            );

            let creator_badge_bucket = self.mint_creator_badge(
                creator_coin_bucket.resource_address(),
                coin_name,
                coin_symbol,
                PoolMode::Normal,
            );

            let hook_argument = HookArgument {
                coin_address: coin_address,
                operation: HookableOperation::PostQuickLaunch,
                amount: Some(coin_supply),
                mode: PoolMode::Normal,
                price: Some(coin_price),
            };
            let buckets = self.execute_hooks(
                &vec![],
                &hook_argument,
            );

            (creator_badge_bucket, creator_coin_bucket, buckets)
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

            let mut pool = self.pools.get_mut(&coin_address).expect("Coin not found");

            let (coin_bucket, price, mode) = pool.buy(base_coin_bucket);

            let hook_argument = HookArgument {
                coin_address: coin_address,
                operation: HookableOperation::PostBuy,
                amount: Some(coin_bucket.amount()),
                mode: mode,
                price: Some(price),
            };
            let pool_enabled_hooks = pool.enabled_hooks.get_hooks(hook_argument.operation);
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
            let amount = coin_bucket.amount();
            let coin_address = coin_bucket.resource_address();
            let mut pool = self.pools.get_mut(&coin_address).expect("Coin not found");

            let (mut base_coin_bucket, price, mode) = pool.sell(coin_bucket);

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    base_coin_bucket.amount() * self.buy_sell_fee_percentage / dec!(100),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            let hook_argument = HookArgument {
                coin_address: coin_address,
                operation: HookableOperation::PostSell,
                amount: Some(amount),
                mode: mode,
                price: Some(price),
            };
            let pool_enabled_hooks = pool.enabled_hooks.get_hooks(hook_argument.operation);
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
            let creator_id = self.pools.get_mut(&coin_address).expect("Coin not found").set_liquidation_mode();

            self.creator_badge_resource_manager.update_non_fungible_data(
                &NonFungibleLocalId::integer(creator_id.into()),
                "pool_mode",
                PoolMode::Liquidation,
            );
        }

        pub fn creator_set_liquidation_mode(
            &mut self,
            creator_proof: Proof,
        ) {
            let creator_data = self.get_creator_data(creator_proof);

            self.creator_badge_resource_manager.update_non_fungible_data(
                &NonFungibleLocalId::integer(creator_data.id.into()),
                "pool_mode",
                PoolMode::Liquidation,
            );

            self.pools.get_mut(&creator_data.coin_resource_address).unwrap().set_liquidation_mode();
        }

        pub fn get_flash_loan(
            &mut self,
            coin_address: ResourceAddress,
            amount: Decimal
        ) -> (Bucket, Bucket) {
            let (coin_bucket, price) = self.pools.get_mut(&coin_address).expect("Coin not found").get_flash_loan(amount);

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

            let mut pool = self.pools.get_mut(&coin_bucket.resource_address()).unwrap();

            // In order to avoid price manipulation affecting the fees, take the maximum among the
            // price at the moment the flash loan was granted and the current price.
            let (_, _, mut price, _, _, _, mode, _, _, _, _, _, _, _) = pool.get_pool_info();
            price = max(price, flash_loan_data.price);

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    flash_loan_data.coin_amount * price * self.flash_loan_fee_percentage / dec!(100),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            pool.return_flash_loan(
                base_coin_bucket,
                coin_bucket,
                price,
            );

            let hook_argument = HookArgument {
                coin_address: flash_loan_data.coin_resource_address,
                operation: HookableOperation::PostReturnFlashLoan,
                amount: Some(flash_loan_data.coin_amount),
                mode: mode,
                price: Some(price),
            };
            let pool_enabled_hooks = pool.enabled_hooks.get_hooks(hook_argument.operation);
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

            self.pools.get_mut(&creator_data.coin_resource_address).unwrap()
            .update_pool_fee_percentage(
                buy_pool_fee_percentage,
                sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage,
            );
        }

        pub fn get_pool_info(
            &self,
            coin_address: ResourceAddress,
        ) -> PoolInfo {
            let (
                base_coin_amount,
                coin_amount,
                last_price,
                buy_pool_fee_percentage,
                sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage,
                pool_mode,
                end_launch_time,
                unlocking_time,
                initial_locked_amount,
                unlocked_amount,
                ticket_price,
                winning_tickets,
                coins_per_winning_ticket,
            ) = self.pools.get(&coin_address).expect("Coin not found").get_pool_info();

            PoolInfo {
                base_coin_amount: base_coin_amount,
                coin_amount: coin_amount,
                last_price: last_price,
                total_buy_fee_percentage: dec!(1000000) / ((100 - buy_pool_fee_percentage) * (100 - self.buy_sell_fee_percentage)) - dec!(100),
                total_sell_fee_percentage: sell_pool_fee_percentage + self.buy_sell_fee_percentage * (100 - sell_pool_fee_percentage) / dec!(100),
                total_flash_loan_fee_percentage: flash_loan_pool_fee_percentage + self.flash_loan_fee_percentage,
                pool_mode: pool_mode,
                end_launch_time: end_launch_time,
                unlocking_time: unlocking_time,
                initial_locked_amount: initial_locked_amount,
                unlocked_amount: unlocked_amount,
                flash_loan_nft_resource_address: self.flash_loan_nft_resource_manager.address(),
                hooks_badge_resource_address: self.hooks_badge_vault.resource_address(),
                ticket_price: ticket_price,
                winning_tickets: winning_tickets,
                coins_per_winning_ticket: coins_per_winning_ticket,
            }
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
            let mut pool = self.pools.get_mut(&coin_address).unwrap();

            let price = pool.launch(end_launch_time, unlocking_time);

            self.creator_badge_resource_manager.update_non_fungible_data(
                &NonFungibleLocalId::integer(creator_data.id.into()),
                "pool_mode",
                PoolMode::Launching,
            );

            let hook_argument = HookArgument {
                coin_address: coin_address,
                operation: HookableOperation::PostFairLaunch,
                amount: None,
                mode: PoolMode::Launching,
                price: Some(price),
            };
            let pool_enabled_hooks = pool.enabled_hooks.get_hooks(hook_argument.operation);
            drop(pool);
            self.execute_hooks(
                &pool_enabled_hooks,
                &hook_argument,
            )
        }

        fn execute_hooks(
            &self,
            pool_enabled_hooks: &Vec<String>,
            hook_argument: &HookArgument,
        ) -> Vec<Bucket> {
            let merged_hooks = self.globally_enabled_hooks.merge(
                hook_argument.operation,
                pool_enabled_hooks,
            );

            let registered_hooks_per_operation = self.registered_hooks_operations.get_hooks(hook_argument.operation);

            let mut additional_buckets: Vec<Bucket> = vec![];

            for hook in merged_hooks.iter() {
                let hook_address = self.registered_hooks.get(&hook);

                // Ignore hoooks that have been unregistered by the componet owner; do not panic
                if hook_address.is_none() || !registered_hooks_per_operation.iter().any(|x| x == hook) {
                    continue;
                }

                let hook_output = self.hooks_badge_vault.as_fungible().authorize_with_amount(
                    1,
                    || { hook_address.unwrap().deref().hook(hook_argument.clone()) }
                );

                match hook_output {
                    None => {},
                    Some(bucket) => additional_buckets.push(bucket),
                }
            }

            additional_buckets
        }

        pub fn terminate_launch(
            &mut self,
            creator_proof: Proof,
        ) -> (Option<Bucket>, Option<Vec<Bucket>>) {
            let creator_data = self.get_creator_data(creator_proof);
            let coin_address = creator_data.coin_resource_address;

            let mut pool = self.pools.get_mut(&coin_address).unwrap();

            let (mut bucket, price, supply, operation) = pool.terminate_launch();

            match operation {
                None => {
                    let mut key: u32 = 1;

                    while bucket.amount() >= Decimal::ONE {
                        RNG.request_random(
                            Runtime::global_address(),
                            "random_callback".to_string(),
                            "random_on_error".to_string(),
                            key,
                            Some(bucket.take(Decimal::ONE).as_fungible()),
                            0, // TODO: try to find a value
                        );

                        key += 1;
                    }

                    bucket.burn();

                    (None, None)
                },
                Some(operation) => {
                    self.fee_vault.put(
                        bucket.take_advanced(
                            self.creation_fee_percentage * bucket.amount() / 100,
                            WithdrawStrategy::Rounded(RoundingMode::ToZero),
                        )
                    );

                    self.creator_badge_resource_manager.update_non_fungible_data(
                        &NonFungibleLocalId::integer(creator_data.id.into()),
                        "pool_mode",
                        PoolMode::Normal,
                    );

                    let hook_argument = HookArgument {
                        coin_address: coin_address,
                        operation: operation,
                        amount: Some(supply),
                        mode: PoolMode::Normal,
                        price: Some(price),
                    };
                    let pool_enabled_hooks = pool.enabled_hooks.get_hooks(hook_argument.operation);
                    drop(pool);
                        let buckets = self.execute_hooks(
                        &pool_enabled_hooks,
                        &hook_argument,
                    );

                    (Some(bucket), Some(buckets))
                }
            }
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
            let mut pool = self.pools.get_mut(&coin_address).unwrap();

            let coin_bucket = pool.unlock(amount);
            let unlocked_amount = coin_bucket.amount();

            match sell {
                false => (coin_bucket, vec![]),
                true => {
                    let (mut base_coin_bucket, price, mode) =  pool.sell(coin_bucket);

                    self.fee_vault.put(
                        base_coin_bucket.take_advanced(
                            base_coin_bucket.amount() * self.buy_sell_fee_percentage / dec!(100),
                            WithdrawStrategy::Rounded(RoundingMode::ToZero),
                        )
                     );

                    let hook_argument = HookArgument {
                        coin_address: coin_address,
                        operation: HookableOperation::PostSell,
                        amount: Some(unlocked_amount),
                        mode: mode,
                        price: Some(price),
                    };
                    let pool_enabled_hooks = pool.enabled_hooks.get_hooks(hook_argument.operation);
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
            self.registered_hooks_operations.add_hook(&name, &operations);

            self.registered_hooks.insert(name, component_address);
        }

        pub fn unregister_hook(
            &mut self,
            name: String,
            operations: Option<Vec<String>>,
        ) { 
            match operations {
                None => {
                    self.registered_hooks.remove(&name);
                },

                Some(operations) =>
                    self.registered_hooks_operations.remove_hook(&name, &operations),
            }
        }

        pub fn owner_enable_hook(
            &mut self,
            name: String,
            operations: Vec<String>,
        ) {
            let hook_address = self.registered_hooks.get(&name).expect("Unknown hook");

            for operation in operations.iter() {
                assert!(
                    self.registered_hooks_operations.hook_exists(&name, &operation),
                    "Hook {} not registered for operation {}",
                    name,
                    operation,
                );
            }

            self.globally_enabled_hooks.add_hook(&name, &operations);

            Runtime::emit_event(
                HookEnabledEvent {
                    resource_address: None,
                    hook_name: name,
                    hook_address: *hook_address,
                    operations: operations,
                }
            );
        }

        pub fn owner_disable_hook(
            &mut self,
            name: String,
            operations: Vec<String>,
        ) {
            let hook_address = self.registered_hooks.get(&name).expect("Unknown hook");

            self.globally_enabled_hooks.remove_hook(&name, &operations);

            Runtime::emit_event(
                HookDisabledEvent {
                    resource_address: None,
                    hook_name: name,
                    hook_address: *hook_address,
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
            let hook_address = self.registered_hooks.get(&name).expect("Unknown hook");

            for operation in operations.iter() {
                assert!(
                    self.registered_hooks_operations.hook_exists(&name, &operation),
                    "Hook {} not registered for operation {}",
                    name,
                    operation,
                );
            }

            let coin_address = self.get_creator_data(creator_proof).coin_resource_address;

            self.pools.get_mut(&coin_address).unwrap().enabled_hooks.add_hook(&name, &operations);
            
            Runtime::emit_event(
                HookEnabledEvent {
                    resource_address: Some(coin_address),
                    hook_name: name,
                    hook_address: *hook_address,
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
            let hook_address = self.registered_hooks.get(&name).expect("Unknown hook");

            let coin_address = self.get_creator_data(creator_proof).coin_resource_address;

            self.pools.get_mut(&coin_address).unwrap().enabled_hooks.remove_hook(&name, &operations);

            Runtime::emit_event(
                HookDisabledEvent {
                    resource_address: Some(coin_address),
                    hook_name: name,
                    hook_address: *hook_address,
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
            self.pools.get_mut(&coin_address).unwrap().burn(amount);
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

            let mut pool = self.pools.get_mut(&coin_address).expect("Coin not found");

            let (ticket_bucket, price) = pool.buy_ticket(
                amount,
                base_coin_bucket
            );
 
            let hook_argument = HookArgument {
                coin_address: coin_address,
                operation: HookableOperation::PostBuyTicket,
                amount: Some(amount.into()),
                mode: PoolMode::Launching,
                price: Some(price),
            };
            let pool_enabled_hooks = pool.enabled_hooks.get_hooks(hook_argument.operation);
            drop(pool);
            let buckets = self.execute_hooks(
                &pool_enabled_hooks,
                &hook_argument,
            );
 
            (ticket_bucket, buckets)
        }

        pub fn random_callback(
            &mut self,
            _key: u32,
            badge: FungibleBucket,
            random_seed: Vec<u8>
        ) {
            let mut pool = self.pools.get_mut(&badge.resource_address()).expect("Coin not found");
            pool.extract_tickets(random_seed);

            badge.burn();
        }

        pub fn random_on_error(
            &self,
            _key: u32,
            badge: FungibleBucket
        ) {
            badge.burn();
        }

        pub fn redeem_ticket(
            &mut self,
            ticket_bucket: Bucket,
        ) -> (
            Bucket,
            Bucket,
            Option<Vec<Bucket>>,
            Option<Vec<Bucket>>,
        ) {
            let ticket_id = &ticket_bucket.as_non_fungible().non_fungible_local_ids()[0];
            let ticket_data = ResourceManager::from_address(ticket_bucket.resource_address()).get_non_fungible_data::<TicketData>(ticket_id);
            let mut pool = self.pools.get_mut(&ticket_data.coin_resource_address).expect("Coin not found");
            let (base_coin_bucket, coin_bucket, lose, win, mode) = pool.redeem_ticket(ticket_bucket);

            let pool_enabled_hooks_lose = pool.enabled_hooks.get_hooks(HookableOperation::PostRedeemLousingTicket);
            let hook_argument_lose = HookArgument {
                coin_address: coin_bucket.resource_address(),
                operation: HookableOperation::PostRedeemLousingTicket,
                amount: Some(Decimal::try_from(lose).unwrap()),
                mode: mode,
                price: None,
            };

            let pool_enabled_hooks_win = pool.enabled_hooks.get_hooks(HookableOperation::PostRedeemWinningTicket);
            let hook_argument_win = HookArgument {
                coin_address: coin_bucket.resource_address(),
                operation: HookableOperation::PostRedeemWinningTicket,
                amount: Some(Decimal::try_from(win).unwrap()),
                mode: mode,
                price: None,
            };

            drop(pool);

            let lose_buckets: Option<Vec<Bucket>>;
            if lose > 0 {
                lose_buckets = Some(
                    self.execute_hooks(
                        &pool_enabled_hooks_lose,
                        &hook_argument_lose,
                    )
                );
            } else {
                lose_buckets = None;
            }

            let win_buckets: Option<Vec<Bucket>>;
            if win > 0 {
                win_buckets = Some(
                    self.execute_hooks(
                        &pool_enabled_hooks_win,
                        &hook_argument_win,
                    )
                );
            } else {
                win_buckets = None;
            }


            (base_coin_bucket, coin_bucket, lose_buckets, win_buckets)
        }
    }
}
