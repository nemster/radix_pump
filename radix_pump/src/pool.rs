use scrypto::prelude::*;
use scrypto::prelude::rust::cmp::*;
use random::Random;
use crate::common::*;
use crate::loan_safe_vault::*;

#[derive(Debug, ScryptoSbor, PartialEq)]
struct QuickLaunchDetails {
    ignored_coins: Decimal,
}

#[derive(Debug, ScryptoSbor, PartialEq)]
struct FairLaunchDetails {
    end_launch_time: i64,
    creator_locked_percentage: Decimal,
    locked_vault: Vault,
    unlocking_time: i64,
    initial_locked_amount: Decimal,
    unlocked_amount: Decimal,
    resource_manager: ResourceManager,
}

#[derive(ScryptoSbor, PartialEq)]
struct RandomLaunchDetails {
    end_launch_time: i64,
    winners_vault: Vault,
    locked_vault: Vault,
    unlocking_time: i64,
    ticket_price: Decimal,
    winning_tickets: u32,
    coins_per_winning_ticket: Decimal,
    sold_tickets: u32,
    resource_manager: ResourceManager,
    ticket_resource_manager: ResourceManager,
    unlocked_amount: Decimal,
    extract_winners: bool,
    number_of_extracted_tickets: u32,
    refunds_vault: Vault,
    key_random: u32,
    random_badge_resource_manager: ResourceManager,
}

#[derive(ScryptoSbor, PartialEq)]
enum LaunchType {
    Quick(QuickLaunchDetails),
    Fair(FairLaunchDetails),
    Random(RandomLaunchDetails),
    AlreadyExistingCoin,
}

static MAX_TICKETS_PER_OPERATION: u32 = 50;
static MAX_CALLS_TO_RANDOM: u32 = 10;

#[blueprint]
mod pool {

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
        roles {
            proxy => updatable_by: [OWNER];
            hook => updatable_by: [OWNER];
        },
        methods {
            launch => restrict_to: [proxy];
            terminate_launch => restrict_to: [proxy];
            unlock => restrict_to: [proxy];
            buy => restrict_to: [proxy, hook];
            sell => restrict_to: [proxy, hook];
            set_liquidation_mode => restrict_to: [proxy];
            get_flash_loan => restrict_to: [proxy];
            return_flash_loan => restrict_to: [proxy];
            get_pool_info => PUBLIC;
            update_pool_fee_percentage => restrict_to: [proxy];
            burn => restrict_to: [proxy];
            buy_ticket => restrict_to: [proxy, hook];
            redeem_ticket => restrict_to: [proxy, hook];
            random_callback => PUBLIC;
            random_on_error => PUBLIC;
            add_liquidity => restrict_to: [proxy, hook];
            remove_liquidity => restrict_to: [proxy, hook];
        }
    }

    struct Pool {
        base_coin_vault: Vault,
        coin_vault: LoanSafeVault,
        mode: PoolMode,
        last_price: Decimal,
        buy_pool_fee_percentage: Decimal,
        sell_pool_fee_percentage: Decimal,
        flash_loan_pool_fee_percentage: Decimal,
        launch: LaunchType,

        // This is only needed by RandomLaunch but unfortunately I can't put it into RandomLaunchDetails
        // because KeyValueStore doesn't implement PartialEq (this would make match unusable on a
        // LaunchType)
        extracted_tickets: KeyValueStore<u64, bool>,
        total_lp: Decimal,
        total_users_lp: Decimal,
        lp_resource_manager: ResourceManager,
        last_lp_id: u64,
        base_coins_to_lp_providers: Decimal,
    }

    impl Pool {

        fn start_resource_manager_creation(
            coin_symbol: String,
            coin_name: String,
            coin_icon_url: String,
            coin_description: String,
            coin_info_url: String,
            coin_creator_badge_rule: AccessRuleNode,
        ) -> InProgressResourceBuilder<FungibleResourceType> {
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::Fixed(AccessRule::Protected(coin_creator_badge_rule.clone())))
            .deposit_roles(deposit_roles!(
                depositor => rule!(allow_all);
                depositor_updater => rule!(deny_all);
            ))
            .withdraw_roles(withdraw_roles!(
                withdrawer => rule!(allow_all);
                withdrawer_updater => rule!(deny_all);
            ))
            .recall_roles(recall_roles!(
                recaller => rule!(deny_all);
                recaller_updater => rule!(deny_all);
            ))
            .freeze_roles(freeze_roles!(
                freezer => rule!(deny_all);
                freezer_updater => rule!(deny_all);
            ))
            .divisibility(DIVISIBILITY_MAXIMUM);

            match coin_info_url.len() {
                0 => 
                    resource_manager.metadata(metadata!(
                        roles {
                            metadata_setter => AccessRule::Protected(coin_creator_badge_rule.clone());
                            metadata_setter_updater => AccessRule::Protected(coin_creator_badge_rule.clone());
                            metadata_locker => AccessRule::Protected(coin_creator_badge_rule.clone());
                            metadata_locker_updater => AccessRule::Protected(coin_creator_badge_rule);
                        },
                        init {
                            "symbol" => coin_symbol, locked;
                            "name" => coin_name, locked;
                            "icon_url" => MetadataValue::Url(UncheckedUrl::of(coin_icon_url)), updatable;
                            "description" => coin_description, updatable;
                        }
                    )),
                _ => 
                    resource_manager.metadata(metadata!(
                        roles {
                            metadata_setter => AccessRule::Protected(coin_creator_badge_rule.clone());
                            metadata_setter_updater => AccessRule::Protected(coin_creator_badge_rule.clone());
                            metadata_locker => AccessRule::Protected(coin_creator_badge_rule.clone());
                            metadata_locker_updater => AccessRule::Protected(coin_creator_badge_rule);
                        },
                        init {
                            "symbol" => coin_symbol, locked;
                            "name" => coin_name, locked;
                            "icon_url" => MetadataValue::Url(UncheckedUrl::of(coin_icon_url)), updatable;
                            "description" => coin_description, updatable;
                            "info_url" => MetadataValue::Url(UncheckedUrl::of(coin_info_url)), updatable;
                        }
                    )),
            }
        }

        fn lp_resource_manager(
            coin_name: String,
            coin_icon_url: UncheckedUrl,
            coin_creator_badge_rule: AccessRuleNode,
            owner_badge_address: ResourceAddress,
            component_address: ComponentAddress,
        ) -> ResourceManager {
            ResourceBuilder::new_integer_non_fungible::<LPData>(
                OwnerRole::Fixed(AccessRule::Protected(coin_creator_badge_rule.clone()))
            )
            .deposit_roles(deposit_roles!(
                depositor => rule!(allow_all);
                depositor_updater => rule!(deny_all);
            ))
            .withdraw_roles(withdraw_roles!(
                withdrawer => rule!(allow_all);
                withdrawer_updater => rule!(deny_all);
            ))
            .recall_roles(recall_roles!(
                recaller => rule!(deny_all);
                recaller_updater => rule!(deny_all);
            ))
            .freeze_roles(freeze_roles!(
                freezer => rule!(deny_all);
                freezer_updater => rule!(deny_all);
            ))
            .burn_roles(burn_roles!(
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(require(owner_badge_address));
            ))
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(owner_badge_address));
            ))
            .metadata(metadata!(
                roles {
                    metadata_setter => AccessRule::Protected(coin_creator_badge_rule.clone());
                    metadata_setter_updater => AccessRule::Protected(coin_creator_badge_rule.clone());
                    metadata_locker => AccessRule::Protected(coin_creator_badge_rule.clone());
                    metadata_locker_updater => AccessRule::Protected(coin_creator_badge_rule);
                },
                init {
                    "name" => format!("LP {}", coin_name), locked;
                    "icon_url" => MetadataValue::Url(coin_icon_url), updatable;
                }
            ))
            .create_with_no_initial_supply()
        }

        pub fn new_fair_launch(
            owner_badge_address: ResourceAddress,
            proxy_badge_address: ResourceAddress,
            hook_badge_address: ResourceAddress,
            coin_symbol: String,
            coin_name: String,
            coin_icon_url: String,
            coin_description: String,
            coin_info_url: String,
            launch_price: Decimal,
            creator_locked_percentage: Decimal,
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee_percentage: Decimal,
            coin_creator_badge_rule: AccessRuleNode,
            base_coin_address: ResourceAddress,
        ) -> (
            Global<Pool>,
            ResourceAddress,
            ResourceAddress,
        ) {
            let (address_reservation, component_address) = Runtime::allocate_component_address(Pool::blueprint_id());

            let resource_manager = Pool::start_resource_manager_creation(
                coin_symbol,
                coin_name.clone(),
                coin_icon_url.clone(),
                coin_description,
                coin_info_url,
                coin_creator_badge_rule.clone(),
            )
            .burn_roles(burn_roles!(
                burner => AccessRule::Protected(coin_creator_badge_rule.clone());
                burner_updater => AccessRule::Protected(coin_creator_badge_rule.clone());
            ))
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(global_caller(component_address)));
            ))
            .create_with_no_initial_supply();

            let lp_resource_manager = Pool::lp_resource_manager(
                coin_name,
                UncheckedUrl::of(coin_icon_url),
                coin_creator_badge_rule,
                owner_badge_address,
                component_address,
            );

            let pool = Self {
                base_coin_vault: Vault::new(base_coin_address),
                coin_vault: LoanSafeVault::new(resource_manager.address()),
                mode: PoolMode::WaitingForLaunch,
                last_price: launch_price,
                buy_pool_fee_percentage: buy_pool_fee_percentage,
                sell_pool_fee_percentage: sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage: flash_loan_pool_fee_percentage,
                launch: LaunchType::Fair(
                    FairLaunchDetails {
                        end_launch_time: 0,
                        creator_locked_percentage: creator_locked_percentage,
                        locked_vault: Vault::new(resource_manager.address()),
                        unlocking_time: 0,
                        initial_locked_amount: Decimal::ZERO,
                        unlocked_amount: Decimal::ZERO,
                        resource_manager: resource_manager,
                    }
                ),
                extracted_tickets: KeyValueStore::new(),
                lp_resource_manager : lp_resource_manager,
                total_lp: Decimal::ZERO,
                total_users_lp: Decimal::ZERO,
                last_lp_id: 0,
                base_coins_to_lp_providers: Decimal::ZERO,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
                hook => rule!(require(hook_badge_address));
            ))
            .with_address(address_reservation)
            .globalize();

            (pool, resource_manager.address(), lp_resource_manager.address())
        }

        pub fn launch(
            &mut self,
            end_launch_time: i64,
            unlocking_time: i64,
        ) -> (
            PoolMode,
            HookArgument,
            AnyPoolEvent,
        ) {
            assert!(
                self.mode == PoolMode::WaitingForLaunch,
                "Not allowed in this mode",
            );

            self.mode = PoolMode::Launching;

            match self.launch {
                LaunchType::Fair(ref mut fair_launch) => {
                    fair_launch.end_launch_time = end_launch_time;
                    fair_launch.unlocking_time = unlocking_time;

                    (
                        PoolMode::Launching,
                        HookArgument {
                            //component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostFairLaunch,
                            amount: None,
                            mode: self.mode,
                            price: Some(self.last_price),
                            ids: vec![],
                        },
                        AnyPoolEvent::FairLaunchStartEvent(
                            FairLaunchStartEvent {
                                resource_address: fair_launch.resource_manager.address(),
                                price: self.last_price,
                                creator_locked_percentage: fair_launch.creator_locked_percentage,
                                end_launch_time: end_launch_time,
                                unlocking_time: unlocking_time,
                                buy_pool_fee_percentage: self.buy_pool_fee_percentage,
                                sell_pool_fee_percentage: self.sell_pool_fee_percentage,
                                flash_loan_pool_fee_percentage: self.flash_loan_pool_fee_percentage,
                            }
                        )
                    )
                },
                LaunchType::Random(ref mut random_launch) => {
                    random_launch.end_launch_time = end_launch_time;
                    random_launch.unlocking_time = unlocking_time;

                    (
                        PoolMode::Launching,
                        HookArgument {
                            //component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostRandomLaunch,
                            amount: None,
                            mode: self.mode,
                            price: Some(self.last_price),
                            ids: vec![],
                        },
                        AnyPoolEvent::RandomLaunchStartEvent(
                            RandomLaunchStartEvent {
                                resource_address: random_launch.resource_manager.address(),
                                ticket_price: random_launch.ticket_price,
                                winning_tickets: random_launch.winning_tickets,
                                coins_per_winning_ticket: random_launch.coins_per_winning_ticket,
                                end_launch_time: end_launch_time,
                                unlocking_time: unlocking_time,
                                buy_pool_fee_percentage: self.buy_pool_fee_percentage,
                                sell_pool_fee_percentage: self.sell_pool_fee_percentage,
                                flash_loan_pool_fee_percentage: self.flash_loan_pool_fee_percentage,
                            }
                        )
                    )
                },
                _ => Runtime::panic("Not allowed for this launch type".to_string()),
            }
        }

        pub fn terminate_launch(&mut self) -> (
            Option<Bucket>,
            Option<PoolMode>,
            Option<HookArgument>,
            Option<AnyPoolEvent>,
        ) {
            match self.launch {
                LaunchType::Fair(ref mut fair_launch) => {
                    assert!(
                        self.mode == PoolMode::Launching,
                        "Not allowed in this mode",
                    );

                    let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
                    assert!(
                        now >= fair_launch.end_launch_time,
                        "Too soon",
                    );
                    fair_launch.end_launch_time = now;

                    self.mode = PoolMode::Normal;

                    let base_coin_bucket = self.base_coin_vault.take_advanced(
                        self.base_coin_vault.amount() * (100 - self.buy_pool_fee_percentage) / 100,
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    );
                    let base_coin_bucket_amount = base_coin_bucket.amount();

                    fair_launch.initial_locked_amount = fair_launch.resource_manager.total_supply().unwrap() *
                        fair_launch.creator_locked_percentage / (dec!(100) - fair_launch.creator_locked_percentage);
                    fair_launch.locked_vault.put(fair_launch.resource_manager.mint(fair_launch.initial_locked_amount));

                    fair_launch.resource_manager.set_mintable(rule!(deny_all));
                    fair_launch.resource_manager.lock_mintable();

                    let supply = fair_launch.resource_manager.total_supply();

                    self.total_lp = self.coin_vault.amount();

                    (
                        Some(base_coin_bucket),
                        Some(PoolMode::Normal),
                        Some(
                            HookArgument {
                                //component: Runtime::global_address().into(),
                                coin_address: self.coin_vault.resource_address(),
                                operation: HookableOperation::PostTerminateFairLaunch,
                                amount: supply,
                                mode: PoolMode::Normal,
                                price: Some(self.last_price),
                                ids: vec![],
                            }
                        ),
                        Some(
                            AnyPoolEvent::FairLaunchEndEvent(
                                FairLaunchEndEvent {
                                    resource_address: fair_launch.resource_manager.address(),
                                    creator_proceeds: base_coin_bucket_amount,
                                    creator_locked_allocation: fair_launch.locked_vault.amount(),
                                    supply: supply.unwrap(),
                                    coins_in_pool: self.coin_vault.amount(),
                                }
                            )
                        )
                    )
                },
                LaunchType::Random(ref mut random_launch) => {
                    match self.mode {
                        PoolMode::Launching => {
                            let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
                            assert!(
                                now >= random_launch.end_launch_time,
                                "Too soon",
                            );
                            random_launch.end_launch_time = now;

                            random_launch.winning_tickets = min(random_launch.winning_tickets, random_launch.sold_tickets);

                            if random_launch.winning_tickets == random_launch.sold_tickets {
                                random_launch.extract_winners = false;

                                self.terminate_random_launch()
                            } else {
                                self.mode = PoolMode::TerminatingLaunch;

                                self.prepare_tickets_extraction()
                            }
                        },
                        PoolMode::TerminatingLaunch => {
                            if random_launch.extract_winners && random_launch.number_of_extracted_tickets < random_launch.winning_tickets ||
                               !random_launch.extract_winners && random_launch.sold_tickets - random_launch.winning_tickets < random_launch.number_of_extracted_tickets {
                                self.prepare_tickets_extraction()
                            } else {
                                random_launch.refunds_vault.put(
                                    self.base_coin_vault.take_advanced(
                                        (random_launch.sold_tickets - random_launch.winning_tickets) * random_launch.ticket_price,
                                        WithdrawStrategy::Rounded(RoundingMode::AwayFromZero),
                                    )
                                );

                                self.terminate_random_launch()
                            }
                        },
                        _ => Runtime::panic("Not allowed in this mode".to_string()),
                    }
                },
                _ => Runtime::panic("Not allowed for this launch type".to_string()),
            }
        }

        fn terminate_random_launch(&mut self) -> (
            Option<Bucket>,
            Option<PoolMode>,
            Option<HookArgument>,
            Option<AnyPoolEvent>,
        ) {
            self.mode = PoolMode::Normal;

            match self.launch {
                LaunchType::Random(ref mut random_launch) => {
                    let amount = random_launch.coins_per_winning_ticket * (random_launch.winning_tickets + 2);
                    let mut coin_bucket = random_launch.resource_manager.mint(amount);
                    random_launch.locked_vault.put(
                        coin_bucket.take(random_launch.coins_per_winning_ticket)
                    );
                    self.coin_vault.put(
                        coin_bucket.take(random_launch.coins_per_winning_ticket)
                    );
                    random_launch.winners_vault.put(coin_bucket);

                    random_launch.resource_manager.set_mintable(rule!(deny_all));
                    random_launch.resource_manager.lock_mintable();

                    let supply = random_launch.resource_manager.total_supply();

                    let base_coin_bucket = self.base_coin_vault.take_advanced(
                        random_launch.winning_tickets * random_launch.ticket_price,
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    );
                    let base_coin_bucket_amount = base_coin_bucket.amount();

                    self.last_price = self.base_coin_vault.amount() / random_launch.coins_per_winning_ticket;

                    self.total_lp = self.coin_vault.amount();

                    (
                        Some(base_coin_bucket),
                        Some(PoolMode::Normal),
                        Some(
                            HookArgument {
                                //component: Runtime::global_address().into(),
                                coin_address: self.coin_vault.resource_address(),
                                operation: HookableOperation::PostTerminateRandomLaunch,
                                amount: supply,
                                mode: PoolMode::Normal,
                                price: Some(self.last_price),
                                ids: vec![],
                            }
                        ),
                        Some(
                            AnyPoolEvent::RandomLaunchEndEvent(
                                RandomLaunchEndEvent {
                                    resource_address: random_launch.resource_manager.address(),
                                    creator_proceeds: base_coin_bucket_amount,
                                    creator_locked_allocation: random_launch.locked_vault.amount(),
                                    supply: supply.unwrap(),
                                    coins_in_pool: self.coin_vault.amount(),
                                }
                            )
                        )
                    )
                },
                _ => Runtime::panic("Should not happen".to_string()),
            }
        }

        fn prepare_tickets_extraction(&mut self) -> (
            Option<Bucket>,
            Option<PoolMode>,
            Option<HookArgument>,
            Option<AnyPoolEvent>,
        ) {
            let mut calls_to_random: u32;
            let remainder: u32;

            match self.launch {
                LaunchType::Random(ref mut random_launch) => {
                    if random_launch.winning_tickets < random_launch.sold_tickets / 2 {
                        random_launch.extract_winners = true;

                        calls_to_random = (random_launch.winning_tickets - random_launch.number_of_extracted_tickets) / MAX_TICKETS_PER_OPERATION;
                        remainder = (random_launch.winning_tickets - random_launch.number_of_extracted_tickets) % MAX_TICKETS_PER_OPERATION;
                    } else {
                        random_launch.extract_winners = false;

                        calls_to_random = (random_launch.sold_tickets - random_launch.winning_tickets - random_launch.number_of_extracted_tickets) / MAX_TICKETS_PER_OPERATION;
                        remainder = (random_launch.sold_tickets - random_launch.winning_tickets - random_launch.number_of_extracted_tickets) % MAX_TICKETS_PER_OPERATION;
                    }
                    if remainder > 0 {
                        calls_to_random += 1;
                    }
                    calls_to_random = min(calls_to_random, MAX_CALLS_TO_RANDOM);

                    let mut random_badge_bucket = random_launch.random_badge_resource_manager.mint(Decimal::try_from(calls_to_random).unwrap());
                    while random_badge_bucket.amount() >= Decimal::ONE {
                        RNG.request_random(
                            Runtime::global_address(),
                            "random_callback".to_string(),
                            "random_on_error".to_string(),
                            random_launch.key_random,
                            Some(random_badge_bucket.take(Decimal::ONE).as_fungible()),
                            10u8,
                        );

                        random_launch.key_random += 1;
                    }

                    // It's mandatory to burn the bucket even if it's empty
                    random_badge_bucket.burn();

                    (None, None, None, None)
                },
                _ => Runtime::panic("Should not happen".to_string()),
            }
        }

        pub fn unlock(
            &mut self,
            amount: Option<Decimal>,
        ) -> Bucket {
            assert!(
                self.mode == PoolMode::Normal,
                "Not allowed in this mode",
            );

            match self.launch {
                LaunchType::Fair(ref mut fair_launch) => {
                    let now = min(Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch, fair_launch.unlocking_time);
                    let unlockable_amount =
                        fair_launch.initial_locked_amount *
                        (now - fair_launch.end_launch_time) / (fair_launch.unlocking_time - fair_launch.end_launch_time) -
                        fair_launch.unlocked_amount;
                    let amount_to_unlock = min(
                        fair_launch.locked_vault.amount(),
                        match amount {
                            None => unlockable_amount,
                            Some(amount) => min(unlockable_amount, amount),
                        }
                    );

                    fair_launch.unlocked_amount += amount_to_unlock;

                    fair_launch.locked_vault.take(amount_to_unlock)
                },
                LaunchType::Random(ref mut random_launch) => {
                    let now = min(Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch, random_launch.unlocking_time);
                    let unlockable_amount =
                        random_launch.coins_per_winning_ticket *
                        (now - random_launch.end_launch_time) / (random_launch.unlocking_time - random_launch.end_launch_time) -
                        random_launch.unlocked_amount;
                    let amount_to_unlock = min(
                        random_launch.locked_vault.amount(),
                        match amount {
                            None => unlockable_amount,
                            Some(amount) => min(unlockable_amount, amount),
                        }
                    );

                    random_launch.unlocked_amount += amount_to_unlock;

                    random_launch.locked_vault.take(amount_to_unlock)
                },
                _ => Runtime::panic("Not allowed for this launch type".to_string()),
            }
        }

        pub fn new(
            owner_badge_address: ResourceAddress,
            proxy_badge_address: ResourceAddress,
            hook_badge_address: ResourceAddress,
            base_coin_address: ResourceAddress,
            coin_address: ResourceAddress,
            coin_name: String,
            coin_icon_url: UncheckedUrl,
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee_percentage: Decimal,
            coin_creator_badge_rule: AccessRuleNode,
        ) -> (
            Global<Pool>,
            ResourceAddress,
        ) {
            let (address_reservation, component_address) = Runtime::allocate_component_address(Pool::blueprint_id());

            let lp_resource_manager = Pool::lp_resource_manager(
                coin_name,
                coin_icon_url,
                coin_creator_badge_rule,
                owner_badge_address,
                component_address,
            );

            let pool = Self {
                base_coin_vault: Vault::new(base_coin_address),
                coin_vault: LoanSafeVault::new(coin_address),
                mode: PoolMode::Uninitialised,
                last_price: Decimal::ONE,
                buy_pool_fee_percentage: buy_pool_fee_percentage,
                sell_pool_fee_percentage: sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage: flash_loan_pool_fee_percentage,
                launch: LaunchType::AlreadyExistingCoin,
                extracted_tickets: KeyValueStore::new(),
                total_lp: Decimal::ZERO,
                total_users_lp: Decimal::ZERO,
                lp_resource_manager: lp_resource_manager,
                last_lp_id: 0,
                base_coins_to_lp_providers: Decimal::ZERO,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
                hook => rule!(require(hook_badge_address));
            ))
            .with_address(address_reservation)
            .globalize();

            (pool, lp_resource_manager.address())
        }

        pub fn new_quick_launch(
            owner_badge_address: ResourceAddress,
            proxy_badge_address: ResourceAddress,
            hook_badge_address: ResourceAddress,
            base_coin_bucket: Bucket,
            coin_symbol: String,
            coin_name: String,
            coin_icon_url: String,
            coin_description: String,
            coin_info_url: String,
            coin_supply: Decimal,
            coin_price: Decimal,
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee_percentage: Decimal,
            coin_creator_badge_rule: AccessRuleNode,
        ) -> (
            Global<Pool>,
            Bucket,
            HookArgument,
            AnyPoolEvent,
            ResourceAddress,
        ) {
            let (address_reservation, component_address) = Runtime::allocate_component_address(Pool::blueprint_id());

            let mut coin_bucket = Pool::start_resource_manager_creation(
                coin_symbol,
                coin_name.clone(),
                coin_icon_url.clone(),
                coin_description,
                coin_info_url,
                coin_creator_badge_rule.clone()
            )
            .burn_roles(burn_roles!(
                burner => AccessRule::Protected(
                    AccessRuleNode::AnyOf(
                        vec![
                            coin_creator_badge_rule.clone(),
                            AccessRuleNode::ProofRule(
                                ProofRule::Require(
                                    global_caller(component_address)
                                )
                            )
                        ]
                    )
                );
                burner_updater => AccessRule::Protected(coin_creator_badge_rule.clone());
            ))
            .mint_roles(mint_roles!(
                minter => rule!(deny_all);
                minter_updater => rule!(deny_all);
            ))
            .mint_initial_supply(coin_supply);

            let coin_address = coin_bucket.resource_address();

            let creator_amount = base_coin_bucket.amount() / coin_price;
            assert!(
                coin_supply >= dec!(2) * creator_amount,
                "Supply is too low",
            );
            let creator_coin_bucket = coin_bucket.take(creator_amount);

            let ignored_coins = coin_bucket.amount() - base_coin_bucket.amount() / coin_price;

            let total_lp = coin_bucket.amount() - ignored_coins;

            let lp_resource_manager = Pool::lp_resource_manager(
                coin_name,
                UncheckedUrl::of(coin_icon_url),
                coin_creator_badge_rule,
                owner_badge_address,
                component_address,
            );

            let pool = Self {
                base_coin_vault: Vault::with_bucket(base_coin_bucket),
                coin_vault: LoanSafeVault::with_bucket(coin_bucket.into()),
                mode: PoolMode::Normal,
                last_price: coin_price,
                buy_pool_fee_percentage: buy_pool_fee_percentage,
                sell_pool_fee_percentage: sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage: flash_loan_pool_fee_percentage,
                launch: LaunchType::Quick(
                    QuickLaunchDetails {
                        ignored_coins: ignored_coins,
                    }
                ),
                extracted_tickets: KeyValueStore::new(),
                total_lp: total_lp,
                total_users_lp: Decimal::ZERO,
                lp_resource_manager: lp_resource_manager,
                last_lp_id: 0,
                base_coins_to_lp_providers: Decimal::ZERO,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
                hook => rule!(require(hook_badge_address));
            ))
            .with_address(address_reservation)
            .globalize();

            (
                pool,
                creator_coin_bucket.into(),
                HookArgument {
                    //component: pool,
                    coin_address: coin_address,
                    operation: HookableOperation::PostQuickLaunch,
                    amount: Some(coin_supply),
                    mode: PoolMode::Normal,
                    price: Some(coin_price),
                    ids: vec![],
                },
                AnyPoolEvent::QuickLaunchEvent(
                    QuickLaunchEvent {
                        resource_address: coin_address,
                        price: coin_price,
                        coins_in_pool: coin_supply - creator_amount,
                        creator_allocation: creator_amount,
                        buy_pool_fee_percentage: buy_pool_fee_percentage,
                        sell_pool_fee_percentage: sell_pool_fee_percentage,
                        flash_loan_pool_fee_percentage: flash_loan_pool_fee_percentage,
                    }
                ),
                lp_resource_manager.address(),
            )
        }

        pub fn new_random_launch(
            owner_badge_address: ResourceAddress,
            proxy_badge_address: ResourceAddress,
            hook_badge_address: ResourceAddress,
            coin_symbol: String,
            coin_name: String,
            coin_icon_url: String,
            coin_description: String,
            coin_info_url: String,
            ticket_price: Decimal,
            winning_tickets: u32,
            coins_per_winning_ticket: Decimal,
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee_percentage: Decimal,
            coin_creator_badge_rule: AccessRuleNode,
            base_coin_address: ResourceAddress,
        ) -> (
            Global<Pool>,
            ResourceAddress,
            ResourceAddress,
        ) {
            let (address_reservation, component_address) = Runtime::allocate_component_address(Pool::blueprint_id());

            let ticket_resource_manager = ResourceBuilder::new_integer_non_fungible::<TicketData>(
                OwnerRole::Fixed(AccessRule::Protected(coin_creator_badge_rule.clone()))
            )
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(global_caller(component_address)));
            ))
            .burn_roles(burn_roles!(
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(deny_all);
            ))
            .deposit_roles(deposit_roles!(
                depositor => rule!(allow_all);
                depositor_updater => rule!(deny_all);
            ))
            .withdraw_roles(withdraw_roles!(
                withdrawer => rule!(allow_all);
                withdrawer_updater => rule!(deny_all);
            ))
            .recall_roles(recall_roles!(
                recaller => rule!(deny_all);
                recaller_updater => rule!(deny_all);
            ))
            .freeze_roles(freeze_roles!(
                freezer => rule!(deny_all);
                freezer_updater => rule!(deny_all);
            ))
            .metadata(metadata!(
                roles {
                    metadata_setter => AccessRule::Protected(coin_creator_badge_rule.clone());
                    metadata_setter_updater => AccessRule::Protected(coin_creator_badge_rule.clone());
                    metadata_locker => AccessRule::Protected(coin_creator_badge_rule.clone());
                    metadata_locker_updater => AccessRule::Protected(coin_creator_badge_rule.clone());
                },
                init {
                    "name" => format!("Ticket for the launch of {}", coin_name), updatable;
                    "icon_url" => MetadataValue::Url(UncheckedUrl::of(coin_icon_url.clone())), updatable;
                    "description" => coin_description.clone(), updatable;
                }
            ))
            .create_with_no_initial_supply();

            let random_badge_resource_manager = ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(require(owner_badge_address))))
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(deny_all);
            ))
            .burn_roles(burn_roles!(
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(deny_all);
            ))
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => format!("Random badge"), updatable;
                }
            ))
            .create_with_no_initial_supply();

            let resource_manager = Pool::start_resource_manager_creation(
                coin_symbol,
                coin_name.clone(),
                coin_icon_url.clone(),
                coin_description,
                coin_info_url,
                coin_creator_badge_rule.clone(),
            )
            .burn_roles(burn_roles!(
                burner => AccessRule::Protected(
                    AccessRuleNode::AnyOf(
                        vec![
                            coin_creator_badge_rule.clone(),
                            AccessRuleNode::ProofRule(
                                ProofRule::Require(
                                    global_caller(component_address)
                                )
                            )
                        ]
                    )
                );
                burner_updater => AccessRule::Protected(coin_creator_badge_rule.clone());
            ))
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(global_caller(component_address)));
            ))
            .create_with_no_initial_supply();

            let lp_resource_manager = Pool::lp_resource_manager(
                coin_name,
                UncheckedUrl::of(coin_icon_url),
                coin_creator_badge_rule,
                owner_badge_address,
                component_address,
            );

            let pool = Pool {
                base_coin_vault: Vault::new(base_coin_address),
                coin_vault: LoanSafeVault::new(resource_manager.address()),
                mode: PoolMode::WaitingForLaunch,
                last_price: ticket_price / coins_per_winning_ticket,
                buy_pool_fee_percentage: buy_pool_fee_percentage,
                sell_pool_fee_percentage: sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage: flash_loan_pool_fee_percentage,
                launch: LaunchType::Random(
                    RandomLaunchDetails {
                        end_launch_time: 0,
                        winners_vault: Vault::new(resource_manager.address()),
                        locked_vault: Vault::new(resource_manager.address()),
                        unlocking_time: 0,
                        ticket_price: ticket_price,
                        winning_tickets: winning_tickets,
                        coins_per_winning_ticket: coins_per_winning_ticket,
                        sold_tickets: 0,
                        resource_manager: resource_manager,
                        ticket_resource_manager: ticket_resource_manager,
                        unlocked_amount: Decimal::ZERO,
                        extract_winners: true,
                        number_of_extracted_tickets: 0,
                        refunds_vault: Vault::new(base_coin_address),
                        key_random: 0,
                        random_badge_resource_manager: random_badge_resource_manager,
                    }
                ),
                extracted_tickets: KeyValueStore::new(),
                total_lp: Decimal::ZERO,
                total_users_lp: Decimal::ZERO,
                lp_resource_manager: lp_resource_manager,
                last_lp_id: 1,
                base_coins_to_lp_providers: Decimal::ZERO,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
                hook => rule!(require(hook_badge_address));
            ))
            .with_address(address_reservation)
            .globalize();

            (pool, resource_manager.address(), lp_resource_manager.address())
        }

        pub fn buy(
            &mut self,
            base_coin_bucket: Bucket,
        ) -> (
            Bucket,
            HookArgument,
            AnyPoolEvent,
        ) {
            let fee = base_coin_bucket.amount() * self.buy_pool_fee_percentage / dec!(100);

            match self.mode {
                PoolMode::Normal => {
                    let constant_product = PreciseDecimal::from(self.base_coin_vault.amount()) * PreciseDecimal::from(self.coins_in_pool());

                    let coins_in_pool_new = (
                        constant_product /
                        PreciseDecimal::from(self.base_coin_vault.amount() + base_coin_bucket.amount() - fee)
                    )
                    .checked_truncate(RoundingMode::ToZero)
                    .unwrap();

                    let coin_amount_bought = self.coins_in_pool() - coins_in_pool_new;

                    self.last_price = base_coin_bucket.amount() / coin_amount_bought;

                    self.base_coin_vault.put(base_coin_bucket);

                    self.update_ignored_coins();

                    (
                        self.coin_vault.take(coin_amount_bought),
                        HookArgument {
                            //component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostBuy,
                            amount: Some(coin_amount_bought),
                            mode: PoolMode::Normal,
                            price: Some(self.last_price),
                            ids: vec![],
                        },
                        AnyPoolEvent::BuyEvent(
                            BuyEvent {
                                resource_address: self.coin_vault.resource_address(),
                                mode: PoolMode::Normal,
                                amount: coin_amount_bought,
                                price: self.last_price,
                                coins_in_pool: self.coin_vault.amount(),
                                fee_paid_to_the_pool: fee,
                            }
                        )
                    )
                },
                PoolMode::Launching => match self.launch {
                    LaunchType::Fair(ref mut fair_launch) => {
                        let mut coin_bucket = fair_launch.resource_manager.mint(
                            base_coin_bucket.amount() / self.last_price
                        );

                        self.coin_vault.put(
                            coin_bucket.take(
                                fee / self.last_price
                            )
                        );

                        self.base_coin_vault.put(base_coin_bucket);

                        let coin_bucket_amount = coin_bucket.amount();

                        (
                            coin_bucket,
                            HookArgument {
                                //component: Runtime::global_address().into(),
                                coin_address: self.coin_vault.resource_address(),
                                operation: HookableOperation::PostBuy,
                                amount: Some(coin_bucket_amount),
                                mode: PoolMode::Launching,
                                price: Some(self.last_price),
                                ids: vec![],
                            },
                            AnyPoolEvent::BuyEvent(
                                BuyEvent {
                                    resource_address: self.coin_vault.resource_address(),
                                    mode: PoolMode::Launching,
                                    amount: coin_bucket_amount,
                                    price: self.last_price,
                                    coins_in_pool: self.coin_vault.amount(),
                                    fee_paid_to_the_pool: fee,
                                }
                            )
                        )
                    },
                    LaunchType::Random(_) => Runtime::panic("Use buy_ticket instead".to_string()),
                    _ => Runtime::panic("Should not happen".to_string()),
                },
                _ => Runtime::panic("Not allowed in this mode".to_string()),
            }
        }

        pub fn sell(
            &mut self,
            coin_bucket: Bucket,
        ) -> (
            Bucket,
            HookArgument,
            AnyPoolEvent,
        ) {
            match self.mode {
                PoolMode::Normal => {
                    let constant_product = PreciseDecimal::from(self.base_coin_vault.amount()) * PreciseDecimal::from(self.coins_in_pool());

                    let coin_bucket_amount = coin_bucket.amount();

                    let base_coins_in_vault_new = (
                        constant_product / 
                        PreciseDecimal::from(coin_bucket_amount + self.coins_in_pool())
                    )
                    .checked_truncate(RoundingMode::ToZero)
                    .unwrap();

                    let bought_base_coins = self.base_coin_vault.amount() - base_coins_in_vault_new;
                    let fee_amount = bought_base_coins * self.sell_pool_fee_percentage / dec!(100);
                    let base_coin_bucket = self.base_coin_vault.take_advanced(
                        bought_base_coins - fee_amount,
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    );

                    self.last_price = base_coin_bucket.amount() / coin_bucket_amount;

                    self.coin_vault.put(coin_bucket);

                    self.update_ignored_coins();

                    (
                        base_coin_bucket,
                        HookArgument {
                            //component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostSell,
                            amount: Some(coin_bucket_amount),
                            mode: PoolMode::Normal,
                            price: Some(self.last_price),
                            ids: vec![],
                        },
                        AnyPoolEvent::SellEvent(
                            SellEvent {
                                resource_address: self.coin_vault.resource_address(),
                                mode: PoolMode::Normal,
                                amount: coin_bucket_amount,
                                price: self.last_price,
                                coins_in_pool: self.coin_vault.amount(),
                                fee_paid_to_the_pool: fee_amount,
                            }
                        )
                    )
                },
                PoolMode::Liquidation => {
                    let coin_bucket_amount = coin_bucket.amount();

                    self.coin_vault.put(coin_bucket);

                    (
                        self.base_coin_vault.take_advanced(
                            coin_bucket_amount * self.last_price,
                            WithdrawStrategy::Rounded(RoundingMode::ToZero),
                        ),
                        HookArgument {
                            //component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostSell,
                            amount: Some(coin_bucket_amount),
                            mode: PoolMode::Liquidation,
                            price: Some(self.last_price),
                            ids: vec![],
                        },
                        AnyPoolEvent::SellEvent(
                            SellEvent {
                                resource_address: self.coin_vault.resource_address(),
                                mode: PoolMode::Liquidation,
                                amount: coin_bucket_amount,
                                price: self.last_price,
                                coins_in_pool: self.coin_vault.amount(),
                                fee_paid_to_the_pool: Decimal::ZERO,
                            }
                        )
                    )
                },
                _ => Runtime::panic("Not allowed in this mode".to_string()),
            }
        }

        pub fn set_liquidation_mode(&mut self) -> (
            PoolMode,
            AnyPoolEvent,
        ) {
            assert!(
                self.mode == PoolMode::Normal ||
                self.mode == PoolMode::Launching ||
                self.mode == PoolMode::TerminatingLaunch,
                "Not allowed in this mode",
            );

            self.mode = PoolMode::Liquidation;

            let coin_resource_manager = ResourceManager::from_address(
                self.coin_vault.resource_address()
            );
            let coin_supply = coin_resource_manager.total_supply().unwrap();

            // This is the number of coins needed to repay LP providers.
            // The factor 2 exist because we have to repay both coins and base coins provided.
            // total_users_lp / total_lp represents the share of the coin in the pool that belogs to
            // LP providers,
            let coin_equivalent_lp: PreciseDecimal = 2 * PreciseDecimal::from(self.coins_in_pool()) * self.total_users_lp / self.total_lp;

            let coin_circulating_supply: PreciseDecimal = match &self.launch {
                LaunchType::Random(random_launch) =>
                    coin_supply +
                    coin_equivalent_lp -
                    random_launch.locked_vault.amount() -
                    self.coin_vault.amount(),
                LaunchType::Fair(fair_launch) =>
                    coin_supply +
                    coin_equivalent_lp -
                    fair_launch.locked_vault.amount() -
                    self.coin_vault.amount(),
                _ =>
                    coin_supply +
                    coin_equivalent_lp -
                    self.coin_vault.amount(),
            };

            // We have to repay the coin circulating supply with the base coins in the pool
            self.last_price = (self.base_coin_vault.amount() / coin_circulating_supply)
                .checked_truncate(RoundingMode::ToZero).unwrap();

            self.base_coins_to_lp_providers = (coin_equivalent_lp * self.base_coin_vault.amount() / coin_circulating_supply)
                .checked_truncate(RoundingMode::ToZero).unwrap();

            (
                PoolMode::Liquidation,
                AnyPoolEvent::LiquidationEvent(
                    LiquidationEvent {
                        resource_address: self.coin_vault.resource_address(),
                        price: self.last_price,
                    }
                )
            )
        }

        pub fn get_flash_loan(
            &mut self,
            amount: Decimal,
        ) -> (
            Bucket, // bucket of coins
            Decimal, // price
        ) {
            (self.coin_vault.get_loan(amount), self.last_price)
        }

        pub fn return_flash_loan(
            &mut self,
            base_coin_bucket: Bucket,
            coin_bucket: Bucket,
            price: Decimal,
        ) -> (
            HookArgument,
            AnyPoolEvent,
        ) {
            assert!(
                self.mode == PoolMode::Normal,
                "Not allowed in this mode",
            );
            assert!(
                base_coin_bucket.amount() >= coin_bucket.amount() * price * self.flash_loan_pool_fee_percentage / dec!(100),
                "Insufficient fee paid to the pool",
            );

            let coin_bucket_amount = coin_bucket.amount();
            let base_coin_bucket_amount = base_coin_bucket.amount();

            self.base_coin_vault.put(base_coin_bucket);
            self.coin_vault.return_loan(coin_bucket);

            self.update_ignored_coins();

            (
                HookArgument { 
                    //component: Runtime::global_address().into(),
                    coin_address: self.coin_vault.resource_address(),
                    operation: HookableOperation::PostReturnFlashLoan,
                    amount: Some(coin_bucket_amount),
                    mode: PoolMode::Normal,
                    price: Some(price),
                    ids: vec![],
                },
                AnyPoolEvent::FlashLoanEvent(
                    FlashLoanEvent {
                        resource_address: self.coin_vault.resource_address(),
                        amount: coin_bucket_amount,
                        fee_paid_to_the_pool: base_coin_bucket_amount,
                    }
                )
            )
        }

        pub fn get_pool_info(&self) -> PoolInfo {
            let coin_amount = self.coins_in_pool();

            // Not launched pools have zero LP
            let coin_lp_ratio: Decimal;
            if self.total_lp == Decimal::ZERO {
                coin_lp_ratio = Decimal::ONE;
            } else {
                coin_lp_ratio = coin_amount / self.total_lp;
            }

            PoolInfo {
                component: Runtime::global_address().into(),
                base_coin_amount: self.base_coin_vault.amount(),
                coin_amount: coin_amount,
                last_price: self.last_price,
                total_buy_fee_percentage: self.buy_pool_fee_percentage,
                total_sell_fee_percentage: self.sell_pool_fee_percentage,
                total_flash_loan_fee_percentage: self.flash_loan_pool_fee_percentage,
                pool_mode: self.mode,
                lp_resource_address: self.lp_resource_manager.address(),
                coin_lp_ratio: coin_lp_ratio,
                end_launch_time: match &self.launch {
                    LaunchType::Fair(fair_launch) => Some(fair_launch.end_launch_time),
                    LaunchType::Random(random_launch) => Some(random_launch.end_launch_time),
                    _ => None,
                },
                unlocking_time: match &self.launch {
                    LaunchType::Fair(fair_launch) => Some(fair_launch.unlocking_time),
                    LaunchType::Random(random_launch) => Some(random_launch.unlocking_time),
                    _ => None,
                },
                initial_locked_amount: match &self.launch {
                    LaunchType::Fair(fair_launch) => Some(fair_launch.initial_locked_amount),
                    LaunchType::Random(random_launch) => Some(random_launch.coins_per_winning_ticket),
                    _ => None,
                },
                unlocked_amount: match &self.launch {
                    LaunchType::Fair(fair_launch) => Some(fair_launch.unlocked_amount),
                    LaunchType::Random(random_launch) => Some(random_launch.unlocked_amount),
                    _ => None,
                },
                ticket_price: match &self.launch {
                    LaunchType::Random(random_launch) => Some(random_launch.ticket_price),
                    _ => None,
                },
                winning_tickets: match &self.launch {
                    LaunchType::Random(random_launch) => Some(random_launch.winning_tickets),
                    _ => None,
                },
                coins_per_winning_ticket: match &self.launch {
                    LaunchType::Random(random_launch) => Some(random_launch.coins_per_winning_ticket),
                    _ => None,
                },
                // These informations will be added by the proxy
                flash_loan_nft_resource_address: None,
                hooks_badge_resource_address: None,
                read_only_hooks_badge_resource_address: None,
            }
        }

        pub fn update_pool_fee_percentage(
            &mut self,
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee_percentage: Decimal,
        ) -> AnyPoolEvent {
            assert!(
                self.mode == PoolMode::WaitingForLaunch ||
                self.mode == PoolMode::TerminatingLaunch ||
                self.mode == PoolMode::Normal,
                "Not allowed in this mode",
            );

            assert!(
                buy_pool_fee_percentage <= self.buy_pool_fee_percentage &&
                sell_pool_fee_percentage <= self.sell_pool_fee_percentage &&
                flash_loan_pool_fee_percentage <= self.flash_loan_pool_fee_percentage,
                "You can't increase pool fees",
            );

            assert!(
                buy_pool_fee_percentage < self.buy_pool_fee_percentage ||
                sell_pool_fee_percentage < self.sell_pool_fee_percentage ||
                flash_loan_pool_fee_percentage < self.flash_loan_pool_fee_percentage,
                "No changes made",
            );

            self.buy_pool_fee_percentage = buy_pool_fee_percentage;
            self.sell_pool_fee_percentage = sell_pool_fee_percentage;
            self.flash_loan_pool_fee_percentage = flash_loan_pool_fee_percentage;

            AnyPoolEvent::FeeUpdateEvent(
                FeeUpdateEvent {
                    resource_address: self.coin_vault.resource_address(),
                    buy_pool_fee_percentage: buy_pool_fee_percentage,
                    sell_pool_fee_percentage: sell_pool_fee_percentage,
                    flash_loan_pool_fee_percentage: sell_pool_fee_percentage,
                }
            )
        }

        pub fn burn(
            &mut self,
            mut amount: Decimal,
        ) -> AnyPoolEvent {
            assert!(
                self.mode == PoolMode::Normal,
                "Not allowed in this mode",
            );

            amount = match self.launch {
                LaunchType::Quick(ref mut quick_launch) => {
                    let amount = min(
                        amount,
                        quick_launch.ignored_coins,
                    );
                    quick_launch.ignored_coins -= amount;

                    amount
                },
                _ => Runtime::panic("Not allowed for this launch type".to_string()),
            };

            assert!(
                amount > Decimal::ZERO,
                "No coins to burn",
            );

            self.coin_vault.take(amount).burn();

            AnyPoolEvent::BurnEvent(
                BurnEvent {
                    resource_address: self.coin_vault.resource_address(),
                    amount: amount,
                }
            )
        }

        pub fn buy_ticket(
            &mut self,
            amount: u32,
            base_coin_bucket: Bucket,
        ) -> (
            Bucket,
            HookArgument,
            AnyPoolEvent,
        ) {
            assert!(
                self.mode == PoolMode::Launching,
                "Not allowed in this mode",
            );
            assert!(
                amount <= MAX_TICKETS_PER_OPERATION,
                "It is not allowed to buy more than {} tickets in a single operation",
                MAX_TICKETS_PER_OPERATION,
            );
        
            match self.launch {
                LaunchType::Random(ref mut random_launch) => {
                    let fee = base_coin_bucket.amount() * self.buy_pool_fee_percentage / 100;
                    let available_base_coin_amount = base_coin_bucket.amount() - fee;
                    assert!(
                        available_base_coin_amount >= Decimal::try_from(amount).unwrap() * random_launch.ticket_price,
                        "Not enough cois to buy that amount of tickets",
                    );

                    let mut ticket_bucket = Bucket::new(random_launch.ticket_resource_manager.address());
                    let mut ids: Vec<u64> = vec![];
                    for i in 0..amount {
                        let ticket_number = random_launch.sold_tickets + i;
                        ids.push(ticket_number.into());

                        ticket_bucket.put(
                            random_launch.ticket_resource_manager.mint_non_fungible(
                                &NonFungibleLocalId::integer(ticket_number.into()),
                                TicketData {
                                    coin_resource_address: self.coin_vault.resource_address(),
                                    buy_date: Clock::current_time_rounded_to_seconds(),
                                },
                            )
                        );
                    }
                    random_launch.sold_tickets += amount;

                    self.base_coin_vault.put(base_coin_bucket);

                    (
                        ticket_bucket,
                        HookArgument { 
                            //component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostBuyTicket,
                            amount: Some(Decimal::try_from(amount).unwrap()),
                            mode: PoolMode::Launching,
                            price: Some(self.last_price),
                            ids: ids,
                        },
                        AnyPoolEvent::BuyTicketEvent(
                            BuyTicketEvent {
                                resource_address: self.coin_vault.resource_address(),
                                amount: amount,
                                price: random_launch.ticket_price,
                                ticket_resource_address: random_launch.ticket_resource_manager.address(),
                                sold_tickets: random_launch.sold_tickets,
                                fee_paid_to_the_pool: fee,
                            }
                        )
                    )
                },
                _ => Runtime::panic("Not allowed for this launch type".to_string()),
            }
        }

        pub fn random_callback(
            &mut self,
            _key: u32,
            badge: FungibleBucket,
            random_seed: Vec<u8>
        ) {
            match self.launch {
                LaunchType::Random(ref mut random_launch) => {
                    assert!(
                        badge.resource_address() == random_launch.random_badge_resource_manager.address() &&
                        badge.amount() == Decimal::ONE,
                        "Wrong badge",
                    );

                    badge.burn();

                    let tickets_to_extract = min(
                        MAX_TICKETS_PER_OPERATION,
                        match random_launch.extract_winners {
                            true => random_launch.winning_tickets,
                            false => random_launch.sold_tickets - random_launch.winning_tickets,
                        } - random_launch.number_of_extracted_tickets,
                    );

                    // Fail quietly if there's nothing left to do
                    if self.mode != PoolMode::TerminatingLaunch ||
                        tickets_to_extract == 0 {
                        return;
                    }

                    let mut random: Random = Random::new(&random_seed);
                    for _i in 0..tickets_to_extract {
                        let mut ticket_id = random.in_range::<u64>(0, random_launch.sold_tickets.into());
                        while self.extracted_tickets.get(&ticket_id).is_some() {
                            ticket_id = random.in_range::<u64>(0, random_launch.sold_tickets.into());
                        }
                        self.extracted_tickets.insert(ticket_id, random_launch.extract_winners);
                    }

                    random_launch.number_of_extracted_tickets += tickets_to_extract;

                },
                _ => Runtime::panic("Not allowed for this launch type".to_string()),
            }
        }

        pub fn random_on_error(
            &self,
            _key: u32,
            badge: FungibleBucket
        ) {
            match &self.launch {
                LaunchType::Random(random_launch) => {
                    assert!(
                        badge.resource_address() == random_launch.random_badge_resource_manager.address() &&
                        badge.amount() == Decimal::ONE,
                        "Wrong badge",
                    );

                    badge.burn();
                },
                _ => Runtime::panic("Not allowed for this launch type".to_string()),
            }
        }

        pub fn redeem_ticket(
            &mut self,
            ticket_bucket: Bucket,
        ) -> (
            Bucket, // base coin bucket
            Option<Bucket>, // coin bucket
            Option<HookArgument>,
            Option<HookArgument>,
        ) {
            assert!(
                self.mode == PoolMode::Normal || self.mode == PoolMode::Liquidation,
                "Not allowed in this mode"
            );

            match self.launch {
                LaunchType::Random(ref mut random_launch) => {
                    assert!(
                        ticket_bucket.resource_address() == random_launch.ticket_resource_manager.address(),
                        "Unknown ticket",
                    );

                    match self.mode {
                        PoolMode::Normal => {
                            let mut base_coin_bucket = Bucket::new(self.base_coin_vault.resource_address());
                            let mut coin_bucket = Bucket::new(self.coin_vault.resource_address());

                            let mut losers: Vec<u64> = vec![];
                            let mut winners: Vec<u64> = vec![];

                            for ticket_id in ticket_bucket.as_non_fungible().non_fungible_local_ids().iter() {
                                match &ticket_id {
                                    NonFungibleLocalId::Integer(ticket_id) => {
                                        let extracted = self.extracted_tickets.get(&ticket_id.value()).is_some();
                                        if extracted && random_launch.extract_winners || !extracted && !random_launch.extract_winners {
                                            coin_bucket.put(random_launch.winners_vault.take(random_launch.coins_per_winning_ticket));
                                            winners.push(ticket_id.value());
                                        } else {
                                            base_coin_bucket.put(random_launch.refunds_vault.take(random_launch.ticket_price));
                                            losers.push(ticket_id.value());
                                        }
                                    },
                                    _ => Runtime::panic("WTF".to_string()),
                                }
                            }

                            ticket_bucket.burn();

                            (
                                base_coin_bucket,
                                Some(coin_bucket),
                                match losers.len() {
                                    0 => None,
                                    _ => Some(
                                        HookArgument { 
                                            //component: Runtime::global_address().into(),
                                            coin_address: self.coin_vault.resource_address(),
                                            operation: HookableOperation::PostRedeemLousingTicket,
                                            amount: Some(Decimal::try_from(losers.len()).unwrap()),
                                            mode: PoolMode::Normal,
                                            price: Some(self.last_price),
                                            ids: losers,
                                        }
                                    ),
                                },
                                match winners.len() {
                                    0 => None,
                                    _ => Some(
                                        HookArgument { 
                                            //component: Runtime::global_address().into(),
                                            coin_address: self.coin_vault.resource_address(),
                                            operation: HookableOperation::PostRedeemWinningTicket,
                                            amount: Some(Decimal::try_from(winners.len()).unwrap()),
                                            mode: PoolMode::Normal,
                                            price: Some(self.last_price),
                                            ids: winners,
                                        }
                                    ),
                                },
                            )
                        },
                        PoolMode::Liquidation => {
                            let number_of_tickets = ticket_bucket.amount();

                            // In liquidation mode all tickets are considered losers
                            let mut losers: Vec<u64> = vec![];

                            for ticket_id in ticket_bucket.as_non_fungible().non_fungible_local_ids().iter() {
                                match &ticket_id {
                                    NonFungibleLocalId::Integer(ticket_id) => losers.push(ticket_id.value()),
                                    _ => Runtime::panic("WTF".to_string()),
                                }
                            }

                            ticket_bucket.burn();

                            (
                                self.base_coin_vault.take_advanced(
                                    self.last_price * number_of_tickets,
                                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                                ),
                                None,
                                Some(
                                    HookArgument { 
                                        //component: Runtime::global_address().into(),
                                        coin_address: self.coin_vault.resource_address(),
                                        operation: HookableOperation::PostRedeemLousingTicket,
                                        amount: Some(number_of_tickets),
                                        mode: PoolMode::Liquidation,
                                        price: Some(self.last_price),
                                        ids: losers,
                                    } 
                                ),
                                None,
                            )
                        },
                        _ => Runtime::panic("Not allowed in this mode".to_string()),
                    }
                },
                _ => Runtime::panic("Not allowed for this launch type".to_string()),
            }
        }

        pub fn add_liquidity(
            &mut self,
            base_coin_bucket: Bucket,
            mut coin_bucket: Bucket,
        ) -> (
            Bucket,
            Option<Bucket>,
            HookArgument,
            AnyPoolEvent,
            Option<PoolMode>,
        ) {
            assert!(
                coin_bucket.amount() > Decimal::ZERO && base_coin_bucket.amount() > Decimal::ZERO,
                "Zero amount not allowed",
            );


            let coins_in_vault = PreciseDecimal::from(self.coins_in_pool());
            let base_coin_amount = PreciseDecimal::from(base_coin_bucket.amount());
            let mut coin_amount = PreciseDecimal::from(coin_bucket.amount());
           
            let (lp, return_bucket, mode) = match self.mode {
                PoolMode::Uninitialised => {
                    // If the pool is empty (AlreadyExistingCoin launch type) initialise the price by using
                    // the coin ratio received.
                    self.last_price = (base_coin_amount / coin_amount).checked_truncate(RoundingMode::ToZero).unwrap();

                    self.mode = PoolMode::Normal;

                    (
                        coin_amount.checked_truncate(RoundingMode::ToZero).unwrap(),
                        None,
                        Some(PoolMode::Normal),
                    )
                },
                PoolMode::Normal => {
                    // If the pool is already initialised, the user is supposed to provide coins and base coins in the
                    // same ratio as those already in the pool.
                    let expected_coin_amount = base_coin_amount * coins_in_vault /
                        PreciseDecimal::from(self.base_coin_vault.amount());

                    // In case the user provided too many base coins for the provided coins the pool just accept
                    // them (pump the price!)
                    // In case the user provided too few base coins the pool returns the excess coins.
                    let return_bucket = coin_bucket.take(
                        max(
                            (coin_amount - expected_coin_amount).checked_truncate(RoundingMode::ToZero).unwrap(),
                            Decimal::ZERO,
                        )
                    );
                    coin_amount = PreciseDecimal::from(coin_bucket.amount());

                    let lp = (coin_amount * (PreciseDecimal::from(self.total_lp) / PreciseDecimal::from(coins_in_vault)))
                        .checked_truncate(RoundingMode::ToZero).unwrap();

                    (
                        lp,
                        Some(return_bucket),
                        None
                    )
                },
                _ => Runtime::panic("Not allowed in this mode".to_string()),
            };

            self.total_lp += lp;
            self.total_users_lp += lp;

            self.last_lp_id += 1;
            let lp_bucket = self.lp_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.last_lp_id),
                LPData {
                    deposited_coins: coin_bucket.amount(),
                    deposited_base_coins: base_coin_bucket.amount(),
                    lp_share: lp,
                    date: Clock::current_time_rounded_to_seconds(),
                    coin_resource_address: coin_bucket.resource_address(),
                }
            );

            self.base_coin_vault.put(base_coin_bucket);
            self.coin_vault.put(coin_bucket);

            self.update_ignored_coins();

            (
                lp_bucket,
                return_bucket,
                HookArgument {
                    //component: Runtime::global_address().into(),
                    coin_address: self.coin_vault.resource_address(),
                    operation: HookableOperation::PostAddLiquidity,
                    amount: Some(coin_amount.checked_truncate(RoundingMode::ToZero).unwrap()),
                    mode: PoolMode::Normal,
                    price: Some(self.last_price),
                    ids: vec![self.last_lp_id],
                },
                AnyPoolEvent::AddLiquidityEvent(
                    AddLiquidityEvent {
                        resource_address: self.coin_vault.resource_address(),
                        amount: coin_amount.checked_truncate(RoundingMode::ToZero).unwrap(),
                    }
                ),
                mode,
            )
        }

        pub fn remove_liquidity(
            &mut self,
            lp_bucket: Bucket,
        ) -> (
            Bucket,
            Option<Bucket>,
            HookArgument,
            AnyPoolEvent,
        ) {
            assert!(
                lp_bucket.resource_address() == self.lp_resource_manager.address(),
                "Unknown LP token",
            );

            let mut lp_share = Decimal::ZERO;
            let mut ids: Vec<u64> = vec![];
            for lp_id in lp_bucket.as_non_fungible().non_fungible_local_ids().iter() {
                match &lp_id {
                    NonFungibleLocalId::Integer(lp_id) => {
                        ids.push(lp_id.value());
                    },
                    _ => Runtime::panic("WTF".to_string()),
                }

                lp_share += self.lp_resource_manager.get_non_fungible_data::<LPData>(&lp_id).lp_share;
            }
            let user_share = PreciseDecimal::from(lp_share) / PreciseDecimal::from(self.total_lp);

            let (base_coin_bucket, coin_bucket, amount) = match &self.mode {
                PoolMode::Normal => {
                    let amount = (user_share * self.coin_vault.amount())
                    .checked_truncate(RoundingMode::ToZero).unwrap();

                    (
                        self.base_coin_vault.take_advanced(
                            (user_share * self.base_coin_vault.amount()).checked_truncate(RoundingMode::ToZero).unwrap(),
                            WithdrawStrategy::Rounded(RoundingMode::ToZero),
                        ),
                        Some(self.coin_vault.take(amount)),
                        amount,
                    )
                },
                PoolMode::Liquidation => (
                    self.base_coin_vault.take_advanced(
                        self.base_coins_to_lp_providers * (lp_share / self.total_users_lp),
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    ),
                    None,
                    Decimal::ZERO,
                ),
                _ => Runtime::panic("Not allowed in this mode".to_string()),
            };

            lp_bucket.burn();

            self.total_lp -= lp_share;
            self.total_users_lp -= lp_share;

            // Needed?
            self.update_ignored_coins();

            (
                base_coin_bucket,
                coin_bucket,
                HookArgument {
                    //component: Runtime::global_address().into(),
                    coin_address: self.coin_vault.resource_address(),
                    operation: HookableOperation::PostRemoveLiquidity,
                    amount: Some(amount),
                    mode: self.mode,
                    price: Some(self.last_price),
                    ids: ids,
                },
                AnyPoolEvent::RemoveLiquidityEvent(
                    RemoveLiquidityEvent {
                        resource_address: self.coin_vault.resource_address(),
                        amount: amount,
                    }
                ),
            )
        }

        // Returns the non ignored number of coins in the pool
        fn coins_in_pool(&self) -> Decimal {
            match &self.launch {
                LaunchType::Quick(quick_launch) =>
                    self.coin_vault.amount() - quick_launch.ignored_coins,
                _ => self.coin_vault.amount(),
            }
        }

        fn update_ignored_coins(&mut self) {
            match self.launch {
                LaunchType::Quick(ref mut quick_launch) =>
                    if quick_launch.ignored_coins > Decimal::ZERO {
                        quick_launch.ignored_coins = max(
                            self.coin_vault.amount() - self.base_coin_vault.amount() / self.last_price,
                            Decimal::ZERO,
                        );
                    }
                 _ => {}
            }
        }
    }
}
