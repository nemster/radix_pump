use scrypto::prelude::*;
use scrypto::prelude::rust::cmp::*;
use std::ops::Deref;
use crate::pool::*;
use crate::hook::*;
use crate::hook_helpers::*;
use crate::hook::hook::*;

// Metadata for the coin creator badge
static CREATOR_BADGE_NAME: &str = "Coin creator badge";

// Metadata for the flash loan transient NFT
static TRANSIENT_NFT_NAME: &str = "Flash loan transient NFT";

// Coin creator badge NFT data
#[derive(Debug, ScryptoSbor, NonFungibleData)]
struct CreatorData {
    coin_resource_address: ResourceAddress,
    coin_name: String,
    coin_symbol: String,
    creation_date: Instant,
}

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
    hook_address: Global<Hook>,
    operations: Vec<String>,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct HookDisabledEvent {
    resource_address: Option<ResourceAddress>,
    hook_name: String,
    hook_address: Global<Hook>,
    operations: Vec<String>,
}

#[blueprint]
#[events(
    FairLaunchStartEvent,
    FairLaunchEndEvent,
    QuickLaunchEvent,
    BuyEvent,
    SellEvent,
    LiquidationEvent,
    FlashLoanEvent,
    HookEnabledEvent,
    HookDisabledEvent,
)]
#[types(
    u64,
    CreatorData,
    FlashLoanData,
)]
mod radix_pump {

    enable_method_auth! {
        methods {
            forbid_symbols => restrict_to: [OWNER];
            forbid_names => restrict_to: [OWNER];
            new_fair_launch => PUBLIC;
            new_quick_launch => PUBLIC;
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
            creator_enable_hook => restrict_to: [OWNER];
            creator_disable_hook => restrict_to: [OWNER];
        }
    }

    enable_package_royalties! {
        new => Free;
        forbid_symbols => Free;
        forbid_names => Free;
        new_fair_launch => Usd(dec!("0.05"));
        new_quick_launch => Usd(dec!("0.05"));
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
                non_fungible_data_updater => rule!(deny_all);
                non_fungible_data_updater_updater => rule!(deny_all);
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

            let hooks_badge_resource_manager = ResourceBuilder::new_fungible(OwnerRole::Updatable(rule!(require(owner_badge_address))))
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
            .create_with_no_initial_supply();

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
                hooks_badge_vault: Vault::with_bucket(hooks_badge_resource_manager.mint(1)),
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
            );
            self.pools.insert(
                coin_resource_address,
                pool,
            );

            self.mint_creator_badge(
                coin_resource_address,
                coin_name,
                coin_symbol,
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
        ) -> (Bucket, Bucket) {
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
            );
            self.pools.insert(
                creator_coin_bucket.resource_address(),
                pool,
            );

            let creator_badge_bucket = self.mint_creator_badge(
                creator_coin_bucket.resource_address(),
                coin_name,
                coin_symbol,
            );

            (creator_badge_bucket, creator_coin_bucket)
        }

        pub fn buy(
            &mut self,
            coin_address: ResourceAddress,
            mut base_coin_bucket: Bucket,
        ) -> Bucket {
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

            self.pools.get_mut(&coin_address)
            .expect("Coin not found")
            .buy(base_coin_bucket)
        }

        pub fn sell(
            &mut self,
            coin_bucket: Bucket,
        ) -> Bucket {
            let mut base_coin_bucket = self.pools.get_mut(&coin_bucket.resource_address())
            .expect("Coin not found")
            .sell(coin_bucket);

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    base_coin_bucket.amount() * self.buy_sell_fee_percentage / dec!(100),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            base_coin_bucket
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
            self.pools.get_mut(&coin_address).expect("Coin not found").set_liquidation_mode();
        }

        pub fn creator_set_liquidation_mode(
            &mut self,
            creator_proof: Proof,
        ) {
            let creator_data = self.get_creator_data(creator_proof);

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
        ) {
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
            let (_, _, mut price, _, _, _, _, _, _, _, _) = pool.get_pool_info();
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
        ) -> (
            Decimal,
            Decimal,
            Decimal,
            Decimal,
            Decimal,
            Decimal,
            PoolMode,
            Option<i64>,
            Option<i64>,
            Option<Decimal>,
            Option<Decimal>,
            ResourceAddress,
            ResourceAddress,
        ) {
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
            ) = self.pools.get(&coin_address).expect("Coin not found").get_pool_info();

            (
                base_coin_amount,
                coin_amount,
                last_price,
                self.buy_sell_fee_percentage + buy_pool_fee_percentage * (100 - self.buy_sell_fee_percentage) / dec!(100),
                sell_pool_fee_percentage + self.buy_sell_fee_percentage * (100 - sell_pool_fee_percentage) / dec!(100),
                flash_loan_pool_fee_percentage + self.flash_loan_fee_percentage,
                pool_mode,
                end_launch_time,
                unlocking_time,
                initial_locked_amount,
                unlocked_amount,
                self.flash_loan_nft_resource_manager.address(),
                self.hooks_badge_vault.resource_address(),
            )
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

            let coin_address = self.get_creator_data(creator_proof).coin_resource_address;
            let mut pool = self.pools.get_mut(&coin_address).unwrap();

            pool.launch(end_launch_time, unlocking_time);

            let merged_hooks = self.globally_enabled_hooks.merge(
                HookableOperation::PostFairLaunch,
                &pool.enabled_hooks.get_hooks(HookableOperation::PostFairLaunch),
            );

            let hook_argument = HookArgument {
                coin_address: coin_address,
                operation: HookableOperation::PostFairLaunch,
                amount: None,
            };

            let mut additional_buckets: Vec<Bucket> = vec![];
            for hook in merged_hooks.iter() {
                let hook_address = self.registered_hooks.get(&hook);

                let hook_output = match hook_address {
                    None => None,
                    Some(hook_address) =>
                        self.hooks_badge_vault.as_fungible().authorize_with_amount(
                            1,
                            || {
                                hook_address.deref().hook(hook_argument.clone())
                            }
                        ),
                };

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
        ) -> Bucket {
            let mut base_coin_bucket = self.pools.get_mut(&self.get_creator_data(creator_proof).coin_resource_address)
            .unwrap()
            .terminate_launch();

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    self.creation_fee_percentage * base_coin_bucket.amount() / 100,
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            base_coin_bucket
        }

        fn mint_creator_badge(
            &mut self,
            coin_resource_address: ResourceAddress,
            coin_name: String,
            coin_symbol: String,
        ) -> Bucket {
            let creator_badge = self.creator_badge_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.next_creator_badge_id.into()),
                CreatorData {
                    coin_resource_address: coin_resource_address,
                    coin_name: coin_name,
                    coin_symbol: coin_symbol,
                    creation_date: Clock::current_time_rounded_to_seconds(),
                }
            );

            self.next_creator_badge_id += 1;

            creator_badge
        }

        fn next_creator_badge_rule(&mut self) -> AccessRule {
            AccessRule::Protected(
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
            )
        }

        pub fn unlock(
            &mut self,
            creator_proof: Proof,
            amount: Option<Decimal>,
            sell: bool,
        ) -> Bucket {
            let mut pool = self.pools.get_mut(&self.get_creator_data(creator_proof).coin_resource_address)
            .unwrap();

            let coin_bucket = pool.unlock(amount);

            match sell {
                false => coin_bucket,
                true => {
                    let mut base_coin_bucket = pool.sell(coin_bucket);

                    self.fee_vault.put(
                        base_coin_bucket.take_advanced(
                            base_coin_bucket.amount() * self.buy_sell_fee_percentage / dec!(100),
                            WithdrawStrategy::Rounded(RoundingMode::ToZero),
                        )
                     );

                    base_coin_bucket
                },
            }
        }

        pub fn register_hook(
            &mut self,
            name: String,
            operations: Vec<String>,
            component_address: Global<Hook>,
        ) {
            self.registered_hooks_operations.add_hook(&name, &operations);

            self.registered_hooks.insert(name, component_address); // TODO: What if it already exists?
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
    }
}
