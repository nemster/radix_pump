use scrypto::prelude::*;
use scrypto::prelude::rust::cmp::*;
use random::Random;
use crate::common::*;
use crate::hook_helpers::*;

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FairLaunchStartEvent {
    resource_address: ResourceAddress,
    price: Decimal,
    creator_locked_percentage: Decimal,
    end_launch_time: i64,
    unlocking_time: i64,
    buy_pool_fee_percentage: Decimal,
    sell_pool_fee_percentage: Decimal,
    flash_loan_pool_fee_percentage: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FairLaunchEndEvent {
    resource_address: ResourceAddress,
    creator_proceeds: Decimal,
    creator_locked_allocation: Decimal,
    supply: Decimal,
    coins_in_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct QuickLaunchEvent {
    resource_address: ResourceAddress,
    price: Decimal,
    coins_in_pool: Decimal,
    creator_allocation: Decimal,
    buy_pool_fee_percentage: Decimal,
    sell_pool_fee_percentage: Decimal,
    flash_loan_pool_fee_percentage: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct RandomLaunchStartEvent {
    resource_address: ResourceAddress,
    ticket_price: Decimal,
    winning_tickets: u32,
    coins_per_winning_ticket: Decimal,
    end_launch_time: i64,
    unlocking_time: i64,
    buy_pool_fee_percentage: Decimal,
    sell_pool_fee_percentage: Decimal,
    flash_loan_pool_fee_percentage: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct BuyEvent {
    resource_address: ResourceAddress,
    mode: PoolMode,
    amount: Decimal,
    price: Decimal,
    coins_in_pool: Decimal,
    fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct SellEvent {
    resource_address: ResourceAddress,
    mode: PoolMode,
    amount: Decimal,
    price: Decimal,
    coins_in_pool: Decimal,
    fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct LiquidationEvent {
    resource_address: ResourceAddress,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FlashLoanEvent {
    resource_address: ResourceAddress,
    amount: Decimal,
    fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct BuyTicketEvent {
    coin_resource_address: ResourceAddress,
    amount: u32,
    price: Decimal,
    ticket_resource_address: ResourceAddress,
    sold_tickets: u32,
    fee_paid_to_the_pool: Decimal,
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
}

#[derive(ScryptoSbor, PartialEq)]
enum LaunchType {
    Quick,
    Fair(FairLaunchDetails),
    Random(RandomLaunchDetails),
}

#[derive(ScryptoSbor)]
pub struct Pool {
    base_coin_vault: Vault,
    coin_vault: Vault,
    mode: PoolMode,
    last_price: Decimal,
    buy_pool_fee_percentage: Decimal,
    sell_pool_fee_percentage: Decimal,
    flash_loan_pool_fee_percentage: Decimal,
    pub enabled_hooks: HooksPerOperation,
    launch: LaunchType,
    creator_id: u64,

    // This is only needed by RandomLaunch but unfortunately I can't put it into RandomLaunchDetails
    // because KeyValueStore doesn't implement PartialEq
    extracted_tickets: KeyValueStore<NonFungibleLocalId, bool>,
}

#[derive(Debug, ScryptoSbor, NonFungibleData)]
pub struct TicketData {
    pub coin_resource_address: ResourceAddress,
    buy_date: Instant,
}

static MAX_TICKETS_PER_OPERATION: u32 = 50;
static MAX_CALLS_TO_RANDOM: u32 = 10;

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

    pub fn new_fair_launch(
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
        creator_id: u64,
    ) -> (Pool, ResourceAddress) {
        let component_address = Runtime::global_address();

        let resource_manager = Pool::start_resource_manager_creation(
            coin_symbol,
            coin_name,
            coin_icon_url,
            coin_description,
            coin_info_url,
            coin_creator_badge_rule.clone(),
        )
        .burn_roles(burn_roles!(
            burner => AccessRule::Protected(coin_creator_badge_rule.clone());
            burner_updater => AccessRule::Protected(coin_creator_badge_rule);
        ))
        .mint_roles(mint_roles!(
            minter => rule!(require(global_caller(component_address)));
            minter_updater => rule!(require(global_caller(component_address)));
        ))
        .create_with_no_initial_supply();

        let pool = Pool {
            base_coin_vault: Vault::new(base_coin_address),
            coin_vault: Vault::new(resource_manager.address()),
            mode: PoolMode::WaitingForLaunch,
            last_price: launch_price,
            buy_pool_fee_percentage: buy_pool_fee_percentage,
            sell_pool_fee_percentage: sell_pool_fee_percentage,
            flash_loan_pool_fee_percentage: flash_loan_pool_fee_percentage,
            enabled_hooks: HooksPerOperation::new(),
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
            creator_id: creator_id,
            extracted_tickets: KeyValueStore::new(),
        };

        (pool, resource_manager.address())
    }

    pub fn launch(
        &mut self,
        end_launch_time: i64,
        unlocking_time: i64,
    ) -> Decimal {
        assert!(
            self.mode == PoolMode::WaitingForLaunch,
            "Not allowed in this mode",
        );

        self.mode = PoolMode::Launching;

        match self.launch {
            LaunchType::Fair(ref mut fair_launch) => {
                fair_launch.end_launch_time = end_launch_time;
                fair_launch.unlocking_time = unlocking_time;

                Runtime::emit_event(
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
                );
            },
            LaunchType::Random(ref mut random_launch) => {
                random_launch.end_launch_time = end_launch_time;
                random_launch.unlocking_time = unlocking_time;

                Runtime::emit_event(
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
                );
            },
            _ => Runtime::panic("Not allowed for this launch type".to_string()),
        };

        self.last_price
    }

    pub fn terminate_launch(&mut self) -> (
        Bucket,
        Decimal,
        Decimal,
        Option<HookableOperation>,
    ) {
        assert!(
            self.mode == PoolMode::Launching || self.mode == PoolMode::TerminatingLaunch,
            "Not allowed in this mode",
        );

        match self.launch {
            LaunchType::Fair(ref mut fair_launch) => {
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

                fair_launch.initial_locked_amount = fair_launch.resource_manager.total_supply().unwrap() *
                    fair_launch.creator_locked_percentage / (dec!(100) - fair_launch.creator_locked_percentage);
                fair_launch.locked_vault.put(fair_launch.resource_manager.mint(fair_launch.initial_locked_amount));

                fair_launch.resource_manager.set_mintable(rule!(deny_all));
                fair_launch.resource_manager.lock_mintable();

                let supply = fair_launch.resource_manager.total_supply().unwrap();

                Runtime::emit_event(
                    FairLaunchEndEvent {
                        resource_address: fair_launch.resource_manager.address(),
                        creator_proceeds: base_coin_bucket.amount(),
                        creator_locked_allocation: fair_launch.locked_vault.amount(),
                        supply: supply,
                        coins_in_pool: self.coin_vault.amount(),
                    }
                );

                (base_coin_bucket, self.last_price, supply, Some(HookableOperation::PostTerminateFairLaunch))
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
                    _ => Runtime::panic("Should not happen".to_string()),
                }
            },
            _ => Runtime::panic("Not allowed for this launch type".to_string()),
        }
    }

    fn terminate_random_launch(&mut self) -> (
        Bucket,
        Decimal,
        Decimal,
        Option<HookableOperation>,
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

                (
                    self.base_coin_vault.take_advanced(
                        random_launch.winning_tickets * random_launch.ticket_price,
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    ),
                    self.last_price,
                    amount,
                    Some(HookableOperation::PostTerminateRandomLaunch),
                )
            },
            _ => Runtime::panic("Should not happen".to_string()),
        }
    }

    fn prepare_tickets_extraction(&mut self) -> (
        Bucket,
        Decimal,
        Decimal,
        Option<HookableOperation>,
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

                (
                    random_launch.resource_manager.mint(Decimal::try_from(calls_to_random).unwrap()),
                    self.last_price,
                    Decimal::ZERO,
                    None
                )
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
            _ => Runtime::panic("Not allowed for this launch type".to_string()),
        }
    }

    // This function instantiates a new Pool and simulates a bought so that the creater gets new coins
    // at about the same price as early birds will do.
    pub fn new_quick_launch(
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
        creator_id: u64,
    ) -> (Pool, Bucket) {
        let mut coin_bucket = Pool::start_resource_manager_creation(
            coin_symbol,
            coin_name,
            coin_icon_url,
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
                                global_caller(Runtime::global_address())
                            )
                        )
                    ]
                )
            );
            burner_updater => AccessRule::Protected(coin_creator_badge_rule);
        ))
        .mint_roles(mint_roles!(
            minter => rule!(deny_all);
            minter_updater => rule!(deny_all);
        ))
        .mint_initial_supply(coin_supply);

        let creator_amount = base_coin_bucket.amount() / coin_price;
        assert!(
            coin_supply >= dec!(2) * creator_amount,
            "Supply is too low",
        );
        let creator_coin_bucket = coin_bucket.take(creator_amount);

        Runtime::emit_event(
            QuickLaunchEvent {
                resource_address: coin_bucket.resource_address(),
                price: coin_price,
                coins_in_pool: coin_bucket.amount(),
                creator_allocation: creator_amount,
                buy_pool_fee_percentage: buy_pool_fee_percentage,
                sell_pool_fee_percentage: sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage: flash_loan_pool_fee_percentage,
            }
        );

        let pool = Pool {
            base_coin_vault: Vault::with_bucket(base_coin_bucket),
            coin_vault: Vault::with_bucket(coin_bucket.into()),
            mode: PoolMode::Normal,
            last_price: coin_price,
            buy_pool_fee_percentage: buy_pool_fee_percentage,
            sell_pool_fee_percentage: sell_pool_fee_percentage,
            flash_loan_pool_fee_percentage: flash_loan_pool_fee_percentage,
            enabled_hooks: HooksPerOperation::new(),
            launch: LaunchType::Quick,
            creator_id: creator_id,
            extracted_tickets: KeyValueStore::new(),
        };

        (pool, creator_coin_bucket.into())
    }

    pub fn new_random_launch(
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
        creator_id: u64,
    ) -> (Pool, ResourceAddress) {
        let component_address = Runtime::global_address();

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

        let resource_manager = Pool::start_resource_manager_creation(
            coin_symbol,
            coin_name,
            coin_icon_url,
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
                                global_caller(Runtime::global_address())
                            )
                        )
                    ]
                )
            );
            burner_updater => AccessRule::Protected(coin_creator_badge_rule);
        ))
        .mint_roles(mint_roles!(
            minter => rule!(require(global_caller(component_address)));
            minter_updater => rule!(require(global_caller(component_address)));
        ))
        .create_with_no_initial_supply();

        let pool = Pool {
            base_coin_vault: Vault::new(base_coin_address),
            coin_vault: Vault::new(resource_manager.address()),
            mode: PoolMode::WaitingForLaunch,
            last_price: ticket_price / coins_per_winning_ticket,
            buy_pool_fee_percentage: buy_pool_fee_percentage,
            sell_pool_fee_percentage: sell_pool_fee_percentage,
            flash_loan_pool_fee_percentage: flash_loan_pool_fee_percentage,
            enabled_hooks: HooksPerOperation::new(),
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
                }
            ),
            creator_id: creator_id,
            extracted_tickets: KeyValueStore::new(),
        };

        (pool, resource_manager.address())
    }

    fn custom_costant_product(
        &self,
    ) -> (PreciseDecimal, PreciseDecimal) {
        let base_coin_amount = PreciseDecimal::from(self.base_coin_vault.amount());
        let coin_amount = PreciseDecimal::from(self.coin_vault.amount());

        match self.launch {
            LaunchType::Quick | LaunchType::Random(_) => {
                let expected_coin_amount = base_coin_amount / PreciseDecimal::from(self.last_price);

                (
                    min(expected_coin_amount, coin_amount) * base_coin_amount,
                    max(coin_amount - expected_coin_amount, PreciseDecimal::ZERO),
                )
            },
            _ => (coin_amount * base_coin_amount, PreciseDecimal::ZERO),
        }
    }

    pub fn buy(
        &mut self,
        base_coin_bucket: Bucket,
    ) -> (Bucket, Decimal, PoolMode) {
        let fee = base_coin_bucket.amount() * self.buy_pool_fee_percentage / dec!(100);

        let (coin_bucket, coins_in_pool) = match self.mode {
            PoolMode::Normal => {
                let (constant_product, ignored_coins) = self.custom_costant_product();

                let coins_in_pool = (
                    ignored_coins +
                    constant_product /
                    PreciseDecimal::from(self.base_coin_vault.amount() + base_coin_bucket.amount() - fee)
                )
                .checked_truncate(RoundingMode::ToZero)
                .unwrap();

                let coin_amount_bought = self.coin_vault.amount() - coins_in_pool;

                self.last_price = base_coin_bucket.amount() / coin_amount_bought;

                (self.coin_vault.take(coin_amount_bought), coins_in_pool)
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

                    (coin_bucket, self.coin_vault.amount())
                },
                LaunchType::Random(_) => Runtime::panic("Use buy_ticket instead".to_string()),
                _ => Runtime::panic("Not allowed for this launch type".to_string()),
            },
            _ => Runtime::panic("Not allowed in this mode".to_string()),
        };

        Runtime::emit_event(
            BuyEvent {
                resource_address: self.coin_vault.resource_address(),
                mode: self.mode,
                amount: coin_bucket.amount(),
                price: self.last_price,
                coins_in_pool: coins_in_pool,
                fee_paid_to_the_pool: fee,
            }
        );

        self.base_coin_vault.put(base_coin_bucket);

        (coin_bucket, self.last_price, self.mode)
    }

    pub fn sell(
        &mut self,
        coin_bucket: Bucket,
    ) -> (Bucket, Decimal, PoolMode) {

        let (base_coin_bucket, fee_amount) = match self.mode {
            PoolMode::Normal => {
                let (constant_product, ignored_coins) = self.custom_costant_product();

                let base_coins_in_vault = (
                    constant_product / 
                    (PreciseDecimal::from(coin_bucket.amount() + self.coin_vault.amount()) - ignored_coins)
                )
                .checked_truncate(RoundingMode::ToZero)
                .unwrap();

                let bought_base_coins = self.base_coin_vault.amount() - base_coins_in_vault;
                let fee_amount = bought_base_coins * self.sell_pool_fee_percentage / dec!(100);
                let base_coin_bucket = self.base_coin_vault.take_advanced(
                    bought_base_coins - fee_amount,
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                );

                (base_coin_bucket, fee_amount)
            },
            PoolMode::Liquidation => {
                let coin_supply = ResourceManager::from_address(
                    self.coin_vault.resource_address()
                ).total_supply().unwrap();

                let user_share = coin_bucket.amount() / (coin_supply - self.coin_vault.amount());

                (
                    self.base_coin_vault.take_advanced(
                        self.base_coin_vault.amount() * user_share,
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    ),
                    Decimal::ZERO,
                )
            },
            _ => Runtime::panic("Not allowed in this mode".to_string()),
        };

        self.last_price = base_coin_bucket.amount() / coin_bucket.amount();

        Runtime::emit_event(
            SellEvent {
                resource_address: self.coin_vault.resource_address(),
                amount: coin_bucket.amount(),
                price: self.last_price,
                coins_in_pool: self.coin_vault.amount() + coin_bucket.amount(),
                mode: self.mode,
                fee_paid_to_the_pool: fee_amount,
            }
        );

        self.coin_vault.put(coin_bucket);

        (base_coin_bucket, self.last_price, self.mode)
    }

    pub fn set_liquidation_mode(&mut self) -> u64 {
        assert!(
            self.mode == PoolMode::Normal || self.mode == PoolMode::Launching,
            "Not allowed in this mode",
        );

        Runtime::emit_event(
            LiquidationEvent {
                resource_address: self.coin_vault.resource_address(),
            }
        );

        self.mode = PoolMode::Liquidation;

        self.creator_id
    }

    pub fn get_flash_loan(
        &mut self,
        amount: Decimal,
    ) -> (Bucket, Decimal) {
        (self.coin_vault.take(amount), self.last_price)
    }

    pub fn return_flash_loan(
        &mut self,
        base_coin_bucket: Bucket,
        coin_bucket: Bucket,
        price: Decimal,
    ) {
        assert!(
            self.mode == PoolMode::Normal,
            "Not allowed in this mode",
        );

        assert!(
            base_coin_bucket.amount() >= coin_bucket.amount() * price * self.flash_loan_pool_fee_percentage / dec!(100),
            "Insufficient fee paid to the pool",
        );

        Runtime::emit_event(
            FlashLoanEvent {
                resource_address: coin_bucket.resource_address(),
                amount: coin_bucket.amount(),
                fee_paid_to_the_pool: base_coin_bucket.amount(),
            }
        );

        self.base_coin_vault.put(base_coin_bucket);
        self.coin_vault.put(coin_bucket);
    }

    pub fn get_pool_info(&self) -> (
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
        Option<Decimal>,
        Option<u32>,
        Option<Decimal>,
    ) {
        (
            self.base_coin_vault.amount(),
            self.coin_vault.amount(),
            self.last_price,
            self.buy_pool_fee_percentage,
            self.sell_pool_fee_percentage,
            self.flash_loan_pool_fee_percentage,
            self.mode,
            match &self.launch {
                LaunchType::Quick => None,
                LaunchType::Fair(fair_launch) => Some(fair_launch.end_launch_time),
                LaunchType::Random(random_launch) => Some(random_launch.end_launch_time),
            },
            match &self.launch {
                LaunchType::Quick => None,
                LaunchType::Fair(fair_launch) => Some(fair_launch.unlocking_time),
                LaunchType::Random(random_launch) => Some(random_launch.unlocking_time),
            },
            match &self.launch {
                LaunchType::Quick => None,
                LaunchType::Fair(fair_launch) => Some(fair_launch.initial_locked_amount),
                LaunchType::Random(random_launch) => Some(random_launch.coins_per_winning_ticket),
            },
            match &self.launch {
                LaunchType::Quick => None,
                LaunchType::Fair(fair_launch) => Some(fair_launch.unlocked_amount),
                LaunchType::Random(random_launch) => Some(random_launch.unlocked_amount),
            },
            match &self.launch {
                LaunchType::Quick | LaunchType::Fair(_) => None,
                LaunchType::Random(random_launch) => Some(random_launch.ticket_price),
            },
            match &self.launch {
                LaunchType::Quick | LaunchType::Fair(_) => None,
                LaunchType::Random(random_launch) => Some(random_launch.winning_tickets),
            },
            match &self.launch {
                LaunchType::Quick | LaunchType::Fair(_) => None,
                LaunchType::Random(random_launch) => Some(random_launch.coins_per_winning_ticket),
            },
        )
    }

    pub fn update_pool_fee_percentage(
        &mut self,
        buy_pool_fee_percentage: Decimal,
        sell_pool_fee_percentage: Decimal,
        flash_loan_pool_fee_percentage: Decimal,
    ) {
        assert!(
            self.mode == PoolMode::WaitingForLaunch || self.mode == PoolMode::Normal,
            "Not allowed in this mode",
        );

        self.buy_pool_fee_percentage = buy_pool_fee_percentage;
        self.sell_pool_fee_percentage = sell_pool_fee_percentage;
        self.flash_loan_pool_fee_percentage = flash_loan_pool_fee_percentage;
    }

    pub fn burn(
        &mut self,
        mut amount: Decimal,
    ) {
        match &self.launch {
            LaunchType::Quick => {
                assert!(
                    self.mode == PoolMode::Normal,
                    "Not allowed in this mode",
                );

                let (_, ignored_coins) = self.custom_costant_product();
                amount = min(amount, ignored_coins.checked_truncate(RoundingMode::ToZero).unwrap());

                assert!(
                    amount > Decimal::ZERO,
                    "No coins to burn",
                );

                self.coin_vault.take(amount).burn();
            },
            _ => Runtime::panic("Not allowed for this launch type".to_string()),
        }
    }

    pub fn buy_ticket(
        &mut self,
        amount: u32,
        base_coin_bucket: Bucket,
    ) -> (Bucket, Decimal) {
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
                for i in 0..amount {
                    let ticket_number = random_launch.sold_tickets + i;

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

                Runtime::emit_event(
                    BuyTicketEvent {
                        coin_resource_address: self.coin_vault.resource_address(),
                        amount: amount,
                        price: random_launch.ticket_price,
                        ticket_resource_address: random_launch.ticket_resource_manager.address(),
                        sold_tickets: random_launch.sold_tickets,
                        fee_paid_to_the_pool: fee,
                    }
                );

                (ticket_bucket, random_launch.ticket_price)
            },
            _ => Runtime::panic("Not allowed for this launch type".to_string()),
        }
    }

    pub fn extract_tickets(
        &mut self,
        random_seed: Vec<u8>
    ) {
        match self.launch {
            LaunchType::Random(ref mut random_launch) => {
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
                    let mut ticket_id = NonFungibleLocalId::integer(random.in_range::<u32>(0, random_launch.sold_tickets).into());
                    while self.extracted_tickets.get(&ticket_id).is_some() {
                        ticket_id = NonFungibleLocalId::integer(random.in_range::<u32>(0, random_launch.sold_tickets).into());
                    }
                    self.extracted_tickets.insert(ticket_id, random_launch.extract_winners);
                }

                random_launch.number_of_extracted_tickets += tickets_to_extract;

            },
            _ => Runtime::panic("Not allowed for this launch type".to_string()),
        }
    }

    pub fn redeem_ticket(
        &mut self,
        ticket_bucket: Bucket,
    ) -> (
        Bucket,
        Bucket,
        u32,
        u32,
        PoolMode,
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

                let mut base_coin_bucket = Bucket::new(self.base_coin_vault.resource_address());
                let mut coin_bucket = Bucket::new(self.coin_vault.resource_address());
                let mut win = 0u32;
                let mut lose = 0u32;

                for ticket_id in ticket_bucket.as_non_fungible().non_fungible_local_ids().iter() {
                    if self.extracted_tickets.get(ticket_id).is_some()  && random_launch.extract_winners {
                        coin_bucket.put(random_launch.winners_vault.take(random_launch.coins_per_winning_ticket));
                        win += 1;
                    } else {
                        base_coin_bucket.put(random_launch.refunds_vault.take(random_launch.ticket_price));
                        lose += 1;
                    }
                }

                ticket_bucket.burn();

                (base_coin_bucket, coin_bucket, lose, win, self.mode)
            }
            _ => Runtime::panic("Not allowed for this launch type".to_string()),
        }
    }
}
