use scrypto::prelude::*;
use scrypto::prelude::rust::cmp::*;
use random::Random;
use crate::common::*;
use crate::loan_safe_vault::*;
use scrypto_interface::*;

/* Each Pool component manages the launch of a different coin.
 *
   Different launch strategies are possible:
   - QuickLaunch: the coin creator has to provide the base coins to initialize the pool.
     He can decide the supply of the coin but he receives only a limited allocation depending on the base coin deposit
     and the launch price.
   - FairLaunch: during an initial launch phase users buy coins at a fixed price and doing so they initialize the
     pool.
     The supply is known when the launch phase ends and no more coins can be minted.
     The creator can decide his own allocation before launch but his coins are time locked.
     The creator also receives the launch sale proceeds (fees excluded).
   - RandomLaunch: during an initial launch phase users buy tickets.
     At the end of the launch phase there's an extraction of the winnings tickets.
     Winning tickets will receive a share of coins while losers get a refund.
     The coin creator receives the equivalent of a winning ticket but his allocation is time locked.
     The creator also receives the launch sale proceeds (fees excluded).
   It is also possible to create a pool for an already existing coin, this way there will be no launch phase.

   Depending on the launch type and his history a Pool can be in a number of different modes of operation.
   - Quick launched coins start directly in the Normal mode
   - Fair and Random launched coins start in the WaitingForLaunch mode
   - Pools for externally created coins start in the Uninitialised mode
   These are all of the possible modes:
   - WaitingForLaunch: FairLaunch or RandomLaunch not started yet
   - Launchin: FairLaunch or RandomLaunch started
   - TerminatingLaunch: RandomLaunch extracting winners
   - Normal: Normal operation
   - Liquidation: Liquidation mode
   - Uninitialised: Pool created for a pre existing coin without adding liquidity
*/

// Additional state for QuickLaunched pools
#[derive(Debug, ScryptoSbor, PartialEq)]
struct QuickLaunchDetails {

    // When a coin is quick launched the whole coin supply (except the creator allocation) is in the pool while
    // the number of base coins deposited by the creator only matches the price his allocation.
    // So, to keep the desired price, a number of coins in the pool are just ignored when appling
    // the constant product formula.
    // The number of ignored coins can decrease over time as users buy the coin or the creator
    // burns them.
    ignored_coins: Decimal,
}

// Additional state for FairLaunched pools
#[derive(Debug, ScryptoSbor, PartialEq)]
struct FairLaunchDetails {

    // When the launch of this coin terminated (or when is supposed to terminate)
    // The creator can terminate the launch after this date, not before.
    end_launch_time: i64,

    // Creator allocation percentage (locked at the end of the launch phase)
    // The supply of the coin is not known until the end of the launch phase so the creator
    // allocation can only be expressed as a percentage
    creator_locked_percentage: Decimal,

    // Vault containing the creator allocation (time locked)
    locked_vault: Vault,

    // When the creator allocation will be fully unlocked
    unlocking_time: i64,

    // Creator allocation
    // The supply of the coin is unknown until the end of the launch phase so the creator
    // allocation is unknown at start
    initial_locked_amount: Decimal,

    // Part of the creator allocation that has been already withdrawn
    unlocked_amount: Decimal,

    // Resource manager to mint the coins when they are bought during the launch phase
    resource_manager: ResourceManager,
}

// Additional state for RandomLaunched pools
#[derive(ScryptoSbor, PartialEq)]
struct RandomLaunchDetails {

    // When the launch of this coin terminated (or when is supposed to terminate)
    // The creator can terminate the launch after this date, not before.
    end_launch_time: i64,

    // The coins waiting to be withdrawn from the winners are doposited here
    winners_vault: Vault,

    // Vault containing the creator allocation (time locked)
    locked_vault: Vault,

    // When the creator allocation will be fully unlocked
    unlocking_time: i64,

    // Price of a ticket for taking part in the launch phase extraction (includes fee)
    ticket_price: Decimal,

    // The number of winners
    winning_tickets: u32,

    // How many coins a winning ticket will receive
    coins_per_winning_ticket: Decimal,

    // How many tickets have been sold
    sold_tickets: u32,

    // Resource manager to mint the coins when the launch phase ends
    resource_manager: ResourceManager,

    // Resource manager to mint the tickets when they are bought during the launch phase
    ticket_resource_manager: ResourceManager,

    // Part of the creator allocation that has been already withdrawn
    unlocked_amount: Decimal,

    // Whether to extract winners or losers
    // If the number of winners is less than half of the sold tickets it's cheaper to extract
    // winners, if it's bigger it's cheaper to extract losers
    extract_winners: bool,

    // If the number of tickets to extract is bigger than MAX_TICKETS_PER_OPERATION the extraction
    // happens in multiple steps, this is why we need to keep track of the number of tickets extracted
    // so far
    number_of_extracted_tickets: u32,

    // Losing tickets get a refund of the paid base coins fee excluded.
    // This is the Vault where these base coins are stored
    refunds_vault: Vault,

    // Each call to the RandomComponent must have a unique id, this is incremented time after time
    key_random: u32,

    // Resource manager to mint badges used for authenticating calls from the RandomComponent
    // The interaction with the RandomComponent is asynchronous: we send it a badge and expect to
    // receive it back in a new call containig the random data
    random_badge_resource_manager: ResourceManager,

    // For a correct refund management we need to know the buy fee percentage that was applied
    // during launch if it has changed later
    buy_fee_during_launch: Decimal,
}

// This enum can contain different structs (QuickLaunchDetails, FairLaunchDetails or RandomLaunchDetails) to
// store information that are needed only for a specfic launch type.
#[derive(ScryptoSbor, PartialEq)]
enum LaunchType {
    Quick(QuickLaunchDetails),
    Fair(FairLaunchDetails),
    Random(RandomLaunchDetails),
    AlreadyExistingCoin,
}

// Limits on the usage of the external random data source
static MAX_TICKETS_PER_OPERATION: u32 = 50;
static MAX_CALLS_TO_RANDOM: u32 = 10;

// Some common error message
static MODE_NOT_ALLOWED: &str = "Not allowed in this mode";
static TYPE_NOT_ALLOWED: &str = "Not allowed for this launch type";
static SHOULD_NOT_HAPPEN: &str = "Should not happen";

#[blueprint_with_traits]
#[types(
    u64,
    bool,
    LPData,
    TicketData,
)]
mod pool {

    // Package and component address of the RandomComponent used in RandomLaunch as source of
    // randomness
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
            get_pool_info => PUBLIC;

            buy => restrict_to: [proxy, hook];
            sell => restrict_to: [proxy, hook];
            buy_ticket => restrict_to: [proxy, hook];
            redeem_ticket => restrict_to: [proxy, hook];
            add_liquidity => restrict_to: [proxy, hook];
            remove_liquidity => restrict_to: [proxy, hook];

            launch => restrict_to: [proxy];
            terminate_launch => restrict_to: [proxy];
            unlock => restrict_to: [proxy];
            set_liquidation_mode => restrict_to: [proxy];
            get_flash_loan => restrict_to: [proxy];
            return_flash_loan => restrict_to: [proxy];
            update_pool_fees => restrict_to: [proxy];
            burn => restrict_to: [proxy];

            random_callback => PUBLIC;
            random_on_error => PUBLIC;
        }
    }

    struct Pool {
        // Vaults for keeping base coins and coins
        base_coin_vault: Vault,
        coin_vault: LoanSafeVault,

        // Current pool mode
        mode: PoolMode,

        // Price of the last operation
        last_price: Decimal,

        // Pool fees
        buy_pool_fee_percentage: Decimal,
        sell_pool_fee_percentage: Decimal,
        flash_loan_pool_fee: Decimal,

        // Launch type with variants
        launch: LaunchType,

        // This is only needed by RandomLaunch but unfortunately I can't put it into RandomLaunchDetails
        // because KeyValueStore doesn't implement PartialEq (this would make match unusable on a
        // LaunchType)
        extracted_tickets: KeyValueStore<u64, bool>,

        // Total LP represents the total liquidity in the pool
        total_lp: Decimal,

        // A pool can contain some liquidity that is not owned by anyone, this is the part of
        // total_lp added by users
        total_users_lp: Decimal,

        // Resource manager for minting LP tokens
        lp_resource_manager: ResourceManager,

        // Id of the last liquidity token minted
        last_lp_id: u64,

        // This variable is only used in liquidation mode to keep track of the coins belonging to the
        // liquidity providers
        base_coins_to_lp_providers: Decimal,
    }

    impl RadixPumpPoolInterfaceTrait for Pool {

// THE FOLLOWING METHOD REQUIRES NO AUTHENTICATION, IT CAN BE CALLED BY ANYONE

        // Return detailed information about the pool status
        fn get_pool_info(&self) -> PoolInfo {

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
                total_flash_loan_fee: self.flash_loan_pool_fee,
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
                creator_badge_resource_address: None,
            }
        }

// THE FOLLOWING METHODS CAN ONLY BE CALLED BY ROUND 0 AND 1 HOOKS AND BY THE RadixPump COMPONENT

        // Call this method to buy coins with base coins
        fn buy(
            &mut self,

            // Base coins
            base_coin_bucket: Bucket,
        ) -> (
            Bucket, // Coins
            HookArgument, // Short description of the operation happened, to be used by hooks
            AnyPoolEvent, // BuyEvent
        ) {
            // Compute the fees owed to the pool
            let fee = base_coin_bucket.amount() * self.buy_pool_fee_percentage / dec!(100);

            match self.mode {
                PoolMode::Normal => {

                    // In Normal mode use the constant product formula to get the number of coins
                    // bought
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

                    // In case of quick launched coins we need to update the number of ignored
                    // coins at each movement
                    self.update_ignored_coins();

                    (
                        self.coin_vault.take(coin_amount_bought),

                        // Create the HookArgument that RadixPump will use to call hooks
                        HookArgument {
                            component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostBuy,
                            amount: Some(coin_amount_bought),
                            mode: PoolMode::Normal,
                            price: self.last_price,
                            ids: vec![],
                        },

                        // Create the event but let RadixPump emit it
                        AnyPoolEvent::BuyEvent(
                            BuyEvent {
                                resource_address: self.coin_vault.resource_address(),
                                mode: PoolMode::Normal,
                                amount: coin_amount_bought,
                                price: self.last_price,
                                coins_in_pool: self.coin_vault.amount(),
                                fee_paid_to_the_pool: fee,
                                integrator_id: 0, // This will be set by RadixPump
                            }
                        )
                    )
                },
                PoolMode::Launching => match self.launch {
                    LaunchType::Fair(ref mut fair_launch) => {

                        // During the launch phase of a fair launched coin, coins are freshly minted
                        // and the price is constant
                        let mut coin_bucket = fair_launch.resource_manager.mint(
                            base_coin_bucket.amount() / self.last_price
                        );

                        // The part of the minted coins matching the fee price is put back in the
                        // pool; this way the pool will be correctly initialised at current price
                        // at the end of the launch phase
                        self.coin_vault.put(
                            coin_bucket.take(
                                fee / self.last_price
                            )
                        );

                        // Put temporary all of the base coins in the pool vault; at the end of the
                        // launch phase the base coins (fees excluded) will be given to the coin
                        // creator
                        self.base_coin_vault.put(base_coin_bucket);

                        let coin_bucket_amount = coin_bucket.amount();

                        (
                            coin_bucket,

                            // Create the HookArgument that RadixPump will use to call hooks
                            HookArgument {
                                component: Runtime::global_address().into(),
                                coin_address: self.coin_vault.resource_address(),
                                operation: HookableOperation::PostBuy,
                                amount: Some(coin_bucket_amount),
                                mode: PoolMode::Launching,
                                price: self.last_price,
                                ids: vec![],
                            },

                            // Create the event but let RadixPump emit it
                            AnyPoolEvent::BuyEvent(
                                BuyEvent {
                                    resource_address: self.coin_vault.resource_address(),
                                    mode: PoolMode::Launching,
                                    amount: coin_bucket_amount,
                                    price: self.last_price,
                                    coins_in_pool: self.coin_vault.amount(),
                                    fee_paid_to_the_pool: fee,
                                    integrator_id: 0, // This will be set by RadixPump
                                }
                            )
                        )
                    },

                    // To take part in a random launch you have to buy a ticket, not the coin
                    // itself
                    LaunchType::Random(_) => Runtime::panic("Use buy_ticket instead".to_string()),

                    // Only RandomLaunch and FairLaunch have a Launching phase
                    _ => Runtime::panic(SHOULD_NOT_HAPPEN.to_string()),
                },

                // Not allowed in WaitingForLaunch, TerminatingLaunch, Liquidation and Uninitialised
                // modes
                _ => Runtime::panic(MODE_NOT_ALLOWED.to_string()),
            }
        }

        // Call this method to sell coins for base coins
        fn sell(
            &mut self,

            // Coins to sell
            coin_bucket: Bucket,
        ) -> (
            Bucket, // Base coins
            HookArgument, // Short description of the operation happened, to be used by hooks
            AnyPoolEvent, // SellEvent
        ) {
            match self.mode {
                PoolMode::Normal => {

                    // In Normal mode use the constant product formula to get the amount of base
                    // coins bought
                    let constant_product = PreciseDecimal::from(self.base_coin_vault.amount()) * PreciseDecimal::from(self.coins_in_pool());
                    let coin_bucket_amount = coin_bucket.amount();
                    let base_coins_in_vault_new = (
                        constant_product / 
                        PreciseDecimal::from(coin_bucket_amount + self.coins_in_pool())
                    )
                    .checked_truncate(RoundingMode::ToZero)
                    .unwrap();
                    let bought_base_coins = self.base_coin_vault.amount() - base_coins_in_vault_new;

                    // Take the fee from the base coin bucket
                    let fee_amount = bought_base_coins * self.sell_pool_fee_percentage / dec!(100);
                    let base_coin_bucket = self.base_coin_vault.take_advanced(
                        bought_base_coins - fee_amount,
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    );

                    self.last_price = base_coin_bucket.amount() / coin_bucket_amount;

                    self.coin_vault.put(coin_bucket);

                    // In case of quick launched coins we need to update the number of ignored
                    // coins at each movement
                    self.update_ignored_coins();

                    (
                        base_coin_bucket,

                        // Create the HookArgument that RadixPump will use to call hooks
                        HookArgument {
                            component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostSell,
                            amount: Some(coin_bucket_amount),
                            mode: PoolMode::Normal,
                            price: self.last_price,
                            ids: vec![],
                        },

                        // Create the event but let RadixPump emit it
                        AnyPoolEvent::SellEvent(
                            SellEvent {
                                resource_address: self.coin_vault.resource_address(),
                                mode: PoolMode::Normal,
                                amount: coin_bucket_amount,
                                price: self.last_price,
                                coins_in_pool: self.coin_vault.amount(),
                                fee_paid_to_the_pool: fee_amount,
                                integrator_id: 0, // This will be set by RadixPump
                            }
                        )
                    )
                },
                PoolMode::Liquidation => {
                    let coin_bucket_amount = coin_bucket.amount();

                    self.coin_vault.put(coin_bucket);

                    (
                        // In Liquidation the price is constant and no fees are paid to the pool
                        self.base_coin_vault.take_advanced(
                            coin_bucket_amount * self.last_price,
                            WithdrawStrategy::Rounded(RoundingMode::ToZero),
                        ),

                        // Create the HookArgument that RadixPump will use to call hooks
                        HookArgument {
                            component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostSell,
                            amount: Some(coin_bucket_amount),
                            mode: PoolMode::Liquidation,
                            price: self.last_price,
                            ids: vec![],
                        },

                        // Create the event but let RadixPump emit it
                        AnyPoolEvent::SellEvent(
                            SellEvent {
                                resource_address: self.coin_vault.resource_address(),
                                mode: PoolMode::Liquidation,
                                amount: coin_bucket_amount,
                                price: self.last_price,
                                coins_in_pool: self.coin_vault.amount(),
                                fee_paid_to_the_pool: Decimal::ZERO,
                                integrator_id: 0,
                            }
                        )
                    )
                },
                _ => Runtime::panic(MODE_NOT_ALLOWED.to_string()),
            }
        }

        // Call this method to buy tickets for a reandom launched coin launch phase
        fn buy_ticket(
            &mut self,

            // Number of tickets to buy
            amount: u32,

            // Base coins to buy the tickets
            base_coin_bucket: Bucket,
        ) -> (
            Bucket, // Base coins
            HookArgument, // Short description of the operation happened, to be used by hooks
            AnyPoolEvent, // BuyTicketEvent
        ) {
            assert!(
                self.mode == PoolMode::Launching,
                "Not allowed in this mode",
            );
            assert!(
                amount <= MAX_TICKETS_PER_OPERATION,
                "It is not permitted to buy more than {} tickets in a single operation",
                MAX_TICKETS_PER_OPERATION,
            );
        
            match self.launch {
                LaunchType::Random(ref mut random_launch) => {
                    assert!(
                        base_coin_bucket.amount() >= Decimal::try_from(amount).unwrap() * random_launch.ticket_price,
                        "Not enough cois to buy that amount of tickets",
                    );

                    let fee = base_coin_bucket.amount() * self.buy_pool_fee_percentage / 100;

                    let mut ticket_bucket = Bucket::new(random_launch.ticket_resource_manager.address());
                    let mut ids: Vec<u64> = vec![];
                    let now = Clock::current_time_rounded_to_seconds();

                    // Mint the tickets one by one and put them in ticket_bucket
                    for i in 0..amount {
                        let ticket_number = random_launch.sold_tickets + i;
                        ids.push(ticket_number.into());

                        ticket_bucket.put(
                            random_launch.ticket_resource_manager.mint_non_fungible(
                                &NonFungibleLocalId::integer(ticket_number.into()),
                                TicketData {
                                    coin_resource_address: self.coin_vault.resource_address(),
                                    buy_date: now,
                                },
                            )
                        );
                    }
                    random_launch.sold_tickets += amount;

                    self.base_coin_vault.put(base_coin_bucket);

                    (
                        ticket_bucket,

                        // Create the HookArgument that RadixPump will use to call hooks
                        HookArgument { 
                            component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostBuyTicket,
                            amount: Some(Decimal::try_from(amount).unwrap()),
                            mode: PoolMode::Launching,
                            price: self.last_price,
                            ids: ids,
                        },

                        // Create the event but let RadixPump emit it
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

                // Only random launches sell tickets
                _ => Runtime::panic(TYPE_NOT_ALLOWED.to_string()),
            }
        }

        // Users who bought ticket for a random launched coin can call this method to get their
        // coins (if winners) or a refund (if losers)
        fn redeem_ticket(
            &mut self,

            // Tickets to redeem. It is possible to redeem any number of tickets
            ticket_bucket: Bucket,
        ) -> (
            Bucket, // base coin bucket (can be empty)
            Option<Bucket>, // coin bucket
            Option<HookArgument>, // Winning tickets and losing ones can trigger different hooks
            Option<HookArgument>, // with different arguments
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

                            // Create two vectors for losing and winning ticket ids
                            let mut losers: Vec<u64> = vec![];
                            let mut winners: Vec<u64> = vec![];

                            // For each ticket in the bucket
                            for ticket_id in ticket_bucket.as_non_fungible().non_fungible_local_ids().iter() {
                                match &ticket_id {
                                    NonFungibleLocalId::Integer(ticket_id) => {

                                        // Has the ticket been extracted?
                                        let extracted = self.extracted_tickets.get(&ticket_id.value()).is_some();

                                        // Put the id of the ticket in the winners or in the losers vector
                                        if extracted && random_launch.extract_winners || !extracted && !random_launch.extract_winners {
                                            winners.push(ticket_id.value());
                                        } else {
                                            losers.push(ticket_id.value());
                                        }
                                    },
                                    _ => Runtime::panic(SHOULD_NOT_HAPPEN.to_string()),
                                }
                            }

                            ticket_bucket.burn();

                            // Put in base_coin_bucket the refund (fees excluded) for the losing tickets
                            base_coin_bucket.put(
                                random_launch.refunds_vault.take_advanced(
                                    random_launch.ticket_price * losers.len() * (100 - random_launch.buy_fee_during_launch) / 100,
                                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                                )
                            );

                            // Put in coin_bucket all of the won coins
                            coin_bucket.put(
                                random_launch.winners_vault.take(
                                    random_launch.coins_per_winning_ticket * winners.len()
                                )
                            );

                            (
                                base_coin_bucket,
                                Some(coin_bucket),
                                match losers.len() {
                                    0 => None,
                                    _ => Some(
                                        // Create the HookArgument for the losing tickets
                                        HookArgument { 
                                            component: Runtime::global_address().into(),
                                            coin_address: self.coin_vault.resource_address(),
                                            operation: HookableOperation::PostRedeemLosingTicket,
                                            amount: Some(Decimal::try_from(losers.len()).unwrap()),
                                            mode: PoolMode::Normal,
                                            price: self.last_price,
                                            ids: losers,
                                        }
                                    ),
                                },
                                match winners.len() {
                                    0 => None,
                                    _ => Some(
                                        // Create the HookArgument for the winning tickets
                                        HookArgument { 
                                            component: Runtime::global_address().into(),
                                            coin_address: self.coin_vault.resource_address(),
                                            operation: HookableOperation::PostRedeemWinningTicket,
                                            amount: Some(Decimal::try_from(winners.len()).unwrap()),
                                            mode: PoolMode::Normal,
                                            price: self.last_price,
                                            ids: winners,
                                        }
                                    ),
                                },
                            )
                        },
                        PoolMode::Liquidation => {
                            let number_of_tickets = ticket_bucket.amount();

                            // In liquidation mode all tickets are considered losers, there are no
                            // winners
                            let mut losers: Vec<u64> = vec![];

                            // Fill the list of ticket ids
                            for ticket_id in ticket_bucket.as_non_fungible().non_fungible_local_ids().iter() {
                                match &ticket_id {
                                    NonFungibleLocalId::Integer(ticket_id) => losers.push(ticket_id.value()),
                                    _ => Runtime::panic(SHOULD_NOT_HAPPEN.to_string()),
                                }
                            }

                            ticket_bucket.burn();

                            (
                                // Refund the fees too
                                self.base_coin_vault.take_advanced(
                                    self.last_price * number_of_tickets,
                                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                                ),
                                None, // No coin bucket
                                Some(
                                    // Create the HookArgument for the losing tickets
                                    HookArgument { 
                                        component: Runtime::global_address().into(),
                                        coin_address: self.coin_vault.resource_address(),
                                        operation: HookableOperation::PostRedeemLosingTicket,
                                        amount: Some(number_of_tickets),
                                        mode: PoolMode::Liquidation,
                                        price: self.last_price,
                                        ids: losers,
                                    } 
                                ),
                                None,
                            )
                        },

                        // WaitingForLaunch, Launching, TerminatingLaunch and Uninitialised modes
                        // not allowed
                        _ => Runtime::panic(MODE_NOT_ALLOWED.to_string()),
                    }
                },

                // Only random launches sell tickets
                _ => Runtime::panic(TYPE_NOT_ALLOWED.to_string()),
            }
        }

        // Users can add liquidity to the pool by calling this method.
        // Both base coins and coins must be provided
        fn add_liquidity(
            &mut self,

            // Base coins
            base_coin_bucket: Bucket,

            // Coins
            mut coin_bucket: Bucket,
        ) -> (
            Bucket, // Liquidity token representing the deposited coins
            Option<Bucket>, // Eventual excess coins are returned (eventual excess base coins are
                            // not returned!)
            HookArgument, // Short description of the operation happened, to be used by hooks
            AnyPoolEvent, // AddLiquidityEvent
            Option<PoolMode>, // If the mode of the pool has changed, tell it to the RadixPump
                              // component
        ) {
            assert!(
                coin_bucket.amount() > Decimal::ZERO && base_coin_bucket.amount() > Decimal::ZERO,
                "Zero amount not allowed",
            );

            let coins_in_vault = PreciseDecimal::from(self.coins_in_pool());
            let base_coin_amount = PreciseDecimal::from(base_coin_bucket.amount());
            let mut coin_amount = PreciseDecimal::from(coin_bucket.amount());
          
            // lp here is a measure of the liquidity amount provided
            // The vaule has no special meaning while lp/total_lp is the share of the liquidity added to the user
            let (lp, return_bucket, mode) = match self.mode {
                PoolMode::Uninitialised => {
                    // If the pool is empty (AlreadyExistingCoin launch type) initialise the price by using
                    // the coin ratio received.
                    self.last_price = (base_coin_amount / coin_amount).checked_truncate(RoundingMode::ToZero).unwrap();

                    // Now the pool is correctly initialised
                    self.mode = PoolMode::Normal;

                    (
                        // For the first deposit, just consider lp equal to the number of coins
                        coin_amount.checked_truncate(RoundingMode::ToZero).unwrap(),
                        None,
                        Some(PoolMode::Normal), // Tell RadixPump about the new mode
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

                    // Compute the share of added liquidity
                    let lp = (coin_amount * (PreciseDecimal::from(self.total_lp) / PreciseDecimal::from(coins_in_vault)))
                        .checked_truncate(RoundingMode::ToZero).unwrap();

                    (
                        lp,
                        Some(return_bucket),
                        None
                    )
                },

                // Not allowed in WaitingForLaunch, Launching, TerminatingLaunch and Liquidation
                // modes
                _ => Runtime::panic(MODE_NOT_ALLOWED.to_string()),
            };

            self.total_lp += lp;
            self.total_users_lp += lp;

            // Mint the LP token
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

            // In case of quick launched coins we need to update the number of ignored
            // coins at each movement
            self.update_ignored_coins();

            (
                lp_bucket, // LP token

                return_bucket, // Eventual excess coins provided

                // Short description of the operation happened, to be used by hooks
                HookArgument {
                    component: Runtime::global_address().into(),
                    coin_address: self.coin_vault.resource_address(),
                    operation: HookableOperation::PostAddLiquidity,
                    amount: Some(coin_amount.checked_truncate(RoundingMode::ToZero).unwrap()),
                    mode: PoolMode::Normal,
                    price: self.last_price,
                    ids: vec![self.last_lp_id],
                },

                // Create the event but let RadixPump emit it
                AnyPoolEvent::AddLiquidityEvent(
                    AddLiquidityEvent {
                        resource_address: self.coin_vault.resource_address(),
                        amount: coin_amount.checked_truncate(RoundingMode::ToZero).unwrap(),
                        lp_id: self.last_lp_id,
                    }
                ),
                mode, // Tell RadixPump about the new mode (if changed)
            )
        }

        // Users can invoke this method to get back the proviously added liquidity
        fn remove_liquidity(
            &mut self,

            // LP tokens. It is possible to provide multiple LP tokens in a single operation
            lp_bucket: Bucket,
        ) -> (
            Bucket, // Base coins
            Option<Bucket>, // Coins
            HookArgument, // Short description of the operation happened, to be used by hooks
            AnyPoolEvent, // RemoveLiquidityEvent
        ) {
            assert!(
                lp_bucket.resource_address() == self.lp_resource_manager.address(),
                "Unknown LP token",
            );

            let mut lp_share = Decimal::ZERO;
            let mut ids: Vec<u64> = vec![];

            // Compute the total lp_share of the given LP tokens and build the ids list
            for lp_id in lp_bucket.as_non_fungible().non_fungible_local_ids().iter() {
                match &lp_id {
                    NonFungibleLocalId::Integer(lp_id) => {
                        ids.push(lp_id.value());
                    },
                    _ => Runtime::panic(SHOULD_NOT_HAPPEN.to_string()),
                }

                lp_share += self.lp_resource_manager.get_non_fungible_data::<LPData>(&lp_id).lp_share;
            }
            let user_share = PreciseDecimal::from(lp_share) / PreciseDecimal::from(self.total_lp);

            let (base_coin_bucket, coin_bucket, amount) = match &self.mode {
                PoolMode::Normal => {

                    // In Normal mode take a user_share ratio of both the coins and the base coins
                    // out of the vaults
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

                    // In Liquidation mode only take base coins out of the vault
                    self.base_coin_vault.take_advanced(
                        self.base_coins_to_lp_providers * (lp_share / self.total_users_lp),
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    ),
                    None,
                    Decimal::ZERO,
                ),
                _ => Runtime::panic(MODE_NOT_ALLOWED.to_string()),
            };

            lp_bucket.burn();

            self.total_lp -= lp_share;
            self.total_users_lp -= lp_share;

            // Needed?
            self.update_ignored_coins();

            (
                base_coin_bucket,
                coin_bucket,

                // Create the HookArgument that RadixPump will use to call hooks
                HookArgument {
                    component: Runtime::global_address().into(),
                    coin_address: self.coin_vault.resource_address(),
                    operation: HookableOperation::PostRemoveLiquidity,
                    amount: Some(amount),
                    mode: self.mode,
                    price: self.last_price,
                    ids: ids,
                },

                // Create the event but let RadixPump emit it
                AnyPoolEvent::RemoveLiquidityEvent(
                    RemoveLiquidityEvent {
                        resource_address: self.coin_vault.resource_address(),
                        amount: amount,
                    }
                ),
            )
        }

// THE FOLLOWING METHODS CAN ONLY BE CALLED BY THE RadixPump COMPONENT

        // The creator of a fair or random launched coin can use this method to start the launch
        // phase
        fn launch(
            &mut self,

            // Earliest time the launch will end
            end_launch_time: i64,

            // Time of the total creator allocation unlock
            unlocking_time: i64,
        ) -> (
            PoolMode, // Inform RadixPump about the WaitingForLaunch -> Launching mode change
            HookArgument, // Short description of the operation happened, to be used by hooks
            AnyPoolEvent, // FairLaunchStartEvent or RandomLaunchStartEvent
        ) {
            assert!(
                self.mode == PoolMode::WaitingForLaunch,
                "Not allowed in this mode",
            );
            self.mode = PoolMode::Launching;

            match self.launch {
                LaunchType::Fair(ref mut fair_launch) => {

                    // Set the timings
                    fair_launch.end_launch_time = end_launch_time;
                    fair_launch.unlocking_time = unlocking_time;

                    (
                        PoolMode::Launching,

                        // Create the HookArgument that RadixPump will use to call hooks
                        HookArgument {
                            component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostFairLaunch,
                            amount: None,
                            mode: self.mode,
                            price: self.last_price,
                            ids: vec![],
                        },

                        // Create the event but let RadixPump emit it
                        AnyPoolEvent::FairLaunchStartEvent(
                            FairLaunchStartEvent {
                                resource_address: fair_launch.resource_manager.address(),
                                price: self.last_price,
                                creator_locked_percentage: fair_launch.creator_locked_percentage,
                                end_launch_time: end_launch_time,
                                unlocking_time: unlocking_time,
                                buy_pool_fee_percentage: self.buy_pool_fee_percentage,
                                sell_pool_fee_percentage: self.sell_pool_fee_percentage,
                                flash_loan_pool_fee: self.flash_loan_pool_fee,
                            }
                        )
                    )
                },
                LaunchType::Random(ref mut random_launch) => {

                    // Set the timings
                    random_launch.end_launch_time = end_launch_time;
                    random_launch.unlocking_time = unlocking_time;

                    (
                        PoolMode::Launching,

                        // Create the HookArgument that RadixPump will use to call hooks
                        HookArgument {
                            component: Runtime::global_address().into(),
                            coin_address: self.coin_vault.resource_address(),
                            operation: HookableOperation::PostRandomLaunch,
                            amount: None,
                            mode: self.mode,
                            price: self.last_price,
                            ids: vec![],
                        },

                        // Create the event but let RadixPump emit it
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
                                flash_loan_pool_fee: self.flash_loan_pool_fee,
                            }
                        )
                    )
                },
                _ => Runtime::panic(TYPE_NOT_ALLOWED.to_string()),
            }
        }

        // The creator of a fair or random launched coin can use this method to terminate the launch
        // phase
        // In case of a fair launch the mode goes from Launching to Normal in just one step
        // In case of a random launch there are 4 possibilities:
        // - Launching -> Normal (if sold tickets <= winning tickets, everybody won)
        // - Launching -> TerminatingLaunch (request random data to the RandomComponent)
        // - TerminatingLaunch -> TerminatingLaunch (request more random data to the RandomComponent)
        // - TerminatingLaunch -> Normal (tickets extraction completed)
        fn terminate_launch(&mut self) -> (
            Option<Bucket>, // Proceeds of the sale (base coins)
            Option<PoolMode>, // Inform RadixPump if a mode change happened
            Option<HookArgument>, // Short description of the operation happened, to be used by hooks
            Option<AnyPoolEvent>, // FairLaunchEndEvent, RandomLaunchEndEvent or None
        ) {
            match self.launch {
                LaunchType::Fair(ref mut fair_launch) => {
                    assert!(
                        self.mode == PoolMode::Launching,
                        "Not allowed in this mode",
                    );
                    self.mode = PoolMode::Normal;

                    let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
                    assert!(
                        now >= fair_launch.end_launch_time,
                        "Too soon",
                    );
                    fair_launch.end_launch_time = now;

                    // Get the proceeds of the sale (fee excluded)
                    let base_coin_bucket = self.base_coin_vault.take_advanced(
                        self.base_coin_vault.amount() * (100 - self.buy_pool_fee_percentage) / 100,
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    );
                    let base_coin_bucket_amount = base_coin_bucket.amount();

                    // Mint the locked allocation of coins for the creator
                    fair_launch.initial_locked_amount = fair_launch.resource_manager.total_supply().unwrap() *
                        fair_launch.creator_locked_percentage / (dec!(100) - fair_launch.creator_locked_percentage);
                    fair_launch.locked_vault.put(fair_launch.resource_manager.mint(fair_launch.initial_locked_amount));

                    // Disable mint forever
                    fair_launch.resource_manager.set_mintable(rule!(deny_all));
                    fair_launch.resource_manager.lock_mintable();

                    let supply = fair_launch.resource_manager.total_supply();

                    // The pool now contains:
                    // - the fees of the sale (base coins)
                    // - a coin amount matching the price of the base coins
                    // The price is still the same as it was during the launch phase
                    // This initial liquidity is not owned by anyone, it belongs to the pool itself
                    self.total_lp = self.coin_vault.amount();

                    (
                        Some(base_coin_bucket),
                        Some(PoolMode::Normal),
                        Some(

                            // Create the HookArgument that RadixPump will use to call hooks
                            HookArgument {
                                component: Runtime::global_address().into(),
                                coin_address: self.coin_vault.resource_address(),
                                operation: HookableOperation::PostTerminateFairLaunch,
                                amount: supply,
                                mode: PoolMode::Normal,
                                price: self.last_price,
                                ids: vec![],
                            }
                        ),
                        Some(

                            // Create the event but let RadixPump emit it
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

                            // We will need this information when handling refunds
                            random_launch.buy_fee_during_launch = self.buy_pool_fee_percentage;

                            // If the numer of ticket is smaller or equal than the winning tickets,
                            // everybody won, no extraction happens
                            random_launch.winning_tickets = min(random_launch.winning_tickets, random_launch.sold_tickets);
                            if random_launch.winning_tickets == random_launch.sold_tickets {
                                random_launch.extract_winners = false;

                                self.terminate_random_launch()
                            } else {
                                self.mode = PoolMode::TerminatingLaunch;

                                // Otherwise ask the RandomComponent some randomness
                                self.prepare_tickets_extraction();

                                (None, None, None, None)
                            }
                        },
                        PoolMode::TerminatingLaunch => {

                            // Did we already extract enough tickets?
                            if random_launch.extract_winners && random_launch.number_of_extracted_tickets < random_launch.winning_tickets ||
                               !random_launch.extract_winners && random_launch.sold_tickets - random_launch.winning_tickets < random_launch.number_of_extracted_tickets {

                                // If not, ask the RandomComponent more randomness
                                self.prepare_tickets_extraction();

                                (None, None, None, None)
                            } else {

                                // If yes, take the base coins paid by the losers (fee excluded)
                                // and move them in the refunds vault
                                random_launch.refunds_vault.put(
                                    self.base_coin_vault.take_advanced(
                                        (random_launch.sold_tickets - random_launch.winning_tickets) * random_launch.ticket_price * ((100 - random_launch.buy_fee_during_launch) / 100),
                                        WithdrawStrategy::Rounded(RoundingMode::AwayFromZero),
                                    )
                                );

                                self.terminate_random_launch()
                            }
                        },
                        _ => Runtime::panic(MODE_NOT_ALLOWED.to_string()),
                    }
                },
                _ => Runtime::panic(TYPE_NOT_ALLOWED.to_string()),
            }
        }

        // The creator of a fair or random launched coin can use this method to get (part of) his
        // allocation
        // The creator allocation is time locked and has an unlock schedule linear with time
        fn unlock(
            &mut self,

            // The maximum amount to withdraw (None = all available coins)
            amount: Option<Decimal>,
        ) -> 
            Bucket // Unlocked coins
        {
            assert!(
                self.mode == PoolMode::Normal,
                "Not allowed in this mode",
            );

            match self.launch {
                LaunchType::Fair(ref mut fair_launch) => {

                    // How much is it possible to unlock now?
                    let now = min(Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch, fair_launch.unlocking_time);
                    let unlockable_amount =
                        fair_launch.initial_locked_amount *
                        (now - fair_launch.end_launch_time) / (fair_launch.unlocking_time - fair_launch.end_launch_time) -
                        fair_launch.unlocked_amount;

                    // Does the user want to unlock everything available or just a part of it?
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

                    // How much is it possible to unlock now?
                    let now = min(Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch, random_launch.unlocking_time);
                    let unlockable_amount =
                        random_launch.coins_per_winning_ticket *
                        (now - random_launch.end_launch_time) / (random_launch.unlocking_time - random_launch.end_launch_time) -
                        random_launch.unlocked_amount;

                    // Does the user want to unlock everything available or just a part of it?
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
                _ => Runtime::panic(TYPE_NOT_ALLOWED.to_string()),
            }
        }

        // Call this method to put the pool in Liquidation mode (authentication is in RadixPump
        // component)
        // The goal of the liquidation mode is to get all of the coins into the pool and all of the
        // base coins out of the pool
        fn set_liquidation_mode(&mut self) -> (
            PoolMode, // Inform RadixPump that the mode has changed to Liquidation
            AnyPoolEvent, // LiquidationEvent
        ) {
            assert!(
                self.mode == PoolMode::Normal ||
                self.mode == PoolMode::Launching ||
                self.mode == PoolMode::TerminatingLaunch,
                "Not allowed in this mode",
            );
            self.mode = PoolMode::Liquidation;

            // Get the total supply of the coin
            let coin_resource_manager = ResourceManager::from_address(
                self.coin_vault.resource_address()
            );
            let coin_supply = coin_resource_manager.total_supply().unwrap();

            // This is the number of coins needed to repay LP providers.
            // The factor 2 exist because we have to repay both coins and base coins provided.
            // total_users_lp / total_lp represents the share of the coin in the pool that belogs to
            // LP providers,
            let coin_equivalent_lp: PreciseDecimal = 2 * PreciseDecimal::from(self.coins_in_pool()) * self.total_users_lp / self.total_lp;

            // coin_circulating_supply is the amount of coins that are eligible for a refund, this
            // includes LP tokens and random tickets
            // If there's some creator allocation in the locked_vault it is excluded from the
            // calculation: the creator will never be able to withdraw it
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

            // We have to repay the coin circulating supply with the base coins in the pool, this
            // is the new immutable price of the coins
            self.last_price = (self.base_coin_vault.amount() / coin_circulating_supply)
                .checked_truncate(RoundingMode::ToZero).unwrap();

            // This is the total amount of base coins that will go to liquidity providers
            self.base_coins_to_lp_providers = (coin_equivalent_lp * self.base_coin_vault.amount() / coin_circulating_supply)
                .checked_truncate(RoundingMode::ToZero).unwrap();

            (
                // Tell RadixPump the mode has changed
                PoolMode::Liquidation,

                // Create the event but let RadixPump emit it
                AnyPoolEvent::LiquidationEvent(
                    LiquidationEvent {
                        resource_address: self.coin_vault.resource_address(),
                        price: self.last_price,
                    }
                )
            )
        }

        // Get a flash loan. It is RadixPump responsibility to ensure the loan will be returned
        fn get_flash_loan(
            &mut self,

            // The requested amount of coins
            amount: Decimal,
        ) -> Bucket {

            // Use the get_loan method instead of take, so the output of amount() doesn't change
            self.coin_vault.get_loan(amount)
        }

        // Return a previoulsy received flash loan
        fn return_flash_loan(
            &mut self,

            // Fee paid to the pool (base coins)
            base_coin_bucket: Bucket,

            // Coins to return
            coin_bucket: Bucket,
        ) -> (
            HookArgument, // Short description of the operation happened, to be used by hooks
            AnyPoolEvent, // FlashLoanEvent
        ) {
            assert!(
                self.mode == PoolMode::Normal,
                "Not allowed in this mode",
            );
            assert!(
                base_coin_bucket.amount() >= self.flash_loan_pool_fee,
                "Insufficient fee paid to the pool",
            );

            // Both RadixPump and LoanSafeVault verify that the returned amount is correct, no need to 
            // do it again here
            let coin_bucket_amount = coin_bucket.amount();
            let base_coin_bucket_amount = base_coin_bucket.amount();

            self.base_coin_vault.put(base_coin_bucket);
            self.coin_vault.return_loan(coin_bucket);

            // In case of quick launched coins we need to update the number of ignored
            // coins at each movement
            self.update_ignored_coins();

            (
                // Create the HookArgument that RadixPump will use to call hooks
                HookArgument { 
                    component: Runtime::global_address().into(),
                    coin_address: self.coin_vault.resource_address(),
                    operation: HookableOperation::PostReturnFlashLoan,
                    amount: Some(coin_bucket_amount),
                    mode: PoolMode::Normal,
                    price: self.last_price,
                    ids: vec![],
                },

                // Create the event but let RadixPump emit it
                AnyPoolEvent::FlashLoanEvent(
                    FlashLoanEvent {
                        resource_address: self.coin_vault.resource_address(),
                        amount: coin_bucket_amount,
                        fee_paid_to_the_pool: base_coin_bucket_amount,
                        integrator_id: 0, // RadixPump will set this
                    }
                )
            )
        }

        // The coin creator can use this method to update poll fees, user authentication is managed by RadixPump
        fn update_pool_fees(
            &mut self,

            // The new fees to set
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee: Decimal,

        ) -> AnyPoolEvent // FeeUpdateEvent
        {

            // It is not fair to change fees during launch
            // Fees make no sense during Liquidation phase, no one pays them
            assert!(
                self.mode == PoolMode::WaitingForLaunch ||
                self.mode == PoolMode::TerminatingLaunch ||
                self.mode == PoolMode::Normal,
                "Not allowed in this mode",
            );

            assert!(
                buy_pool_fee_percentage <= self.buy_pool_fee_percentage &&
                sell_pool_fee_percentage <= self.sell_pool_fee_percentage,
                "You can't increase pool percentage fees",
            );

            assert!(
                buy_pool_fee_percentage < self.buy_pool_fee_percentage ||
                sell_pool_fee_percentage < self.sell_pool_fee_percentage ||
                flash_loan_pool_fee != self.flash_loan_pool_fee,
                "No changes made",
            );

            self.buy_pool_fee_percentage = buy_pool_fee_percentage;
            self.sell_pool_fee_percentage = sell_pool_fee_percentage;
            self.flash_loan_pool_fee = flash_loan_pool_fee;

            // Just create the event, let RadixPump emit it
            AnyPoolEvent::FeeUpdateEvent(
                FeeUpdateEvent {
                    resource_address: self.coin_vault.resource_address(),
                    buy_pool_fee_percentage: buy_pool_fee_percentage,
                    sell_pool_fee_percentage: sell_pool_fee_percentage,
                    flash_loan_pool_fee: flash_loan_pool_fee,
                }
            )
        }

        // The creator of a quick launched coin can use this method to burn excess coins in the
        // pool. User authentication is managed bu RadixPump
        fn burn(
            &mut self,

            // Maximum amount of coins to burn
            mut amount: Decimal,

        ) -> AnyPoolEvent // BurnEvent
        {
            assert!(
                self.mode == PoolMode::Normal,
                "Not allowed in this mode",
            );

            amount = match self.launch {
                LaunchType::Quick(ref mut quick_launch) => {

                    // The burned amount can't be bigger then current ingnored_coins
                    let amount = min(
                        amount,
                        quick_launch.ignored_coins,
                    );

                    // Reduce ignored_coins
                    quick_launch.ignored_coins -= amount;

                    amount
                },
                _ => Runtime::panic(TYPE_NOT_ALLOWED.to_string()),
            };

            assert!(
                amount > Decimal::ZERO,
                "No coins to burn",
            );
            self.coin_vault.take(amount).burn();

            // Create the event, let RadixPump emit it
            AnyPoolEvent::BurnEvent(
                BurnEvent {
                    resource_address: self.coin_vault.resource_address(),
                    amount: amount,
                }
            )
        }

    }

    impl Pool {

// CONSTRUCTORS. ONLY THE RadixPump COMPONENT IS SUPPOSED TO CALL THEM

        // Instantiate a pool that will manage a fair launch.
        // During the initial launch phase users buy coins at a fixed price and doing so they initialize the
        // pool.
        // The supply will be known when the launch phase ends and no more coins can be minted.
        // The creator can decide his own allocation percentage before launch but his coins are time locked.
        // The creator also receives the launch sale proceeds (fees excluded).
        // The creator will have to call the launch method to start the launch phase.
        pub fn new_fair_launch(

            // Component owner badge address
            owner_badge_address: ResourceAddress,

            // The badge RadixPump will use for authentication
            proxy_badge_address: ResourceAddress,

            // The badge round 0 and 1 hooks will use for authentication
            hook_badge_address: ResourceAddress,

            // Metadata for the coin to create
            coin_symbol: String,
            coin_name: String,
            coin_icon_url: String,
            coin_description: String,
            coin_info_url: String,
            coin_social_url: Vec<String>,

            // Price of the coin during the launch phase
            launch_price: Decimal,

            // Creator allocation. This is expressed in percentage, the supply of the coin is
            // unknown until the end of the launch phase
            creator_locked_percentage: Decimal,

            // Fees for the pool
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee: Decimal,

            // AccessRuleNode identifying the coin creator badge, it will be the owner of the coin
            // and the LP tokens
            coin_creator_badge_rule: AccessRuleNode,

            // The base coin used to buy and sell the coin and to pay fees
            base_coin_address: ResourceAddress,

            // dApp definition account address to use in components and resources
            dapp_definition: ComponentAddress,
        ) -> (
            ComponentAddress, // The new Pool component address
            ResourceAddress, // The coin resource address
            ResourceAddress, // The LP token resource address
        ) {
            let (address_reservation, component_address) = Runtime::allocate_component_address(Pool::blueprint_id());

            // Create a resource manager to mint the coin during launch phase
            let resource_manager = Pool::start_resource_manager_creation(
                coin_symbol,
                coin_name.clone(),
                coin_icon_url.clone(),
                coin_description,
                coin_info_url,
                coin_social_url.clone(),
                coin_creator_badge_rule.clone(),
            )

            // Only the creator can burn coins but he is also allowed to change this setting
            .burn_roles(burn_roles!(
                burner => AccessRule::Protected(coin_creator_badge_rule.clone());
                burner_updater => AccessRule::Protected(coin_creator_badge_rule.clone());
            ))

            // During the launch phase coins will be minted as users buy them
            // At the end of the launch phase the mint will be locked
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(global_caller(component_address)));
            ))
            .create_with_no_initial_supply();

            // Create a resource manager to mint LP tokens
            let lp_resource_manager = Pool::lp_resource_manager(
                coin_name.clone(),
                UncheckedUrl::of(coin_icon_url),
                coin_creator_badge_rule,
                owner_badge_address,
                component_address,
                dapp_definition,
            );

            // Instantiate the Pool component
            Self {
                base_coin_vault: Vault::new(base_coin_address),
                coin_vault: LoanSafeVault::new(resource_manager.address()),
                mode: PoolMode::WaitingForLaunch,
                last_price: launch_price,
                buy_pool_fee_percentage: buy_pool_fee_percentage,
                sell_pool_fee_percentage: sell_pool_fee_percentage,
                flash_loan_pool_fee: flash_loan_pool_fee,
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
                extracted_tickets: KeyValueStore::new_with_registered_type(),
                lp_resource_manager : lp_resource_manager,
                total_lp: Decimal::ZERO,
                total_users_lp: Decimal::ZERO,
                last_lp_id: 0,
                base_coins_to_lp_providers: Decimal::ZERO,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))

            // Restrict Pool access to the RadixPump component and the hooks
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
                hook => rule!(require(hook_badge_address));
            ))
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "name" => format!("{} pool", coin_name), updatable;
                    "dapp_definition" => dapp_definition, updatable;
                }
            })
            .globalize();

            (component_address, resource_manager.address(), lp_resource_manager.address())
        }

        // This constructor creates an empty pool component that will manage an already existing
        // coin.
        // The user will have to add liquidity to make the pool usable.
        pub fn new(

            // Component owner badge address
            owner_badge_address: ResourceAddress,

            // The badge RadixPump will use for authentication
            proxy_badge_address: ResourceAddress,

            // The badge round 0 and 1 hooks will use for authentication
            hook_badge_address: ResourceAddress,

            // The base coin used to buy and sell the coin and to pay fees
            base_coin_address: ResourceAddress,

            // Information about the existing coin
            coin_address: ResourceAddress,
            coin_name: String,
            coin_icon_url: UncheckedUrl,

            // Fees for the pool
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee: Decimal,

            // AccessRuleNode identifying the coin creator badge, it will be the owner of the LP
            // tokens
            coin_creator_badge_rule: AccessRuleNode,

            // dApp definition account address to use in components and resources
            dapp_definition: ComponentAddress,
        ) -> (
            ComponentAddress, // The address of the created Pool component
            ResourceAddress, // The address of the LP tokens
        ) {
            let (address_reservation, component_address) = Runtime::allocate_component_address(Pool::blueprint_id());

            // Create a resource manager to mint the LP tokens
            let lp_resource_manager = Pool::lp_resource_manager(
                coin_name.clone(),
                coin_icon_url,
                coin_creator_badge_rule,
                owner_badge_address,
                component_address,
                dapp_definition,
            );

            // Instantiate the pool component
            Self {
                base_coin_vault: Vault::new(base_coin_address),
                coin_vault: LoanSafeVault::new(coin_address),
                mode: PoolMode::Uninitialised,
                last_price: Decimal::ONE,
                buy_pool_fee_percentage: buy_pool_fee_percentage,
                sell_pool_fee_percentage: sell_pool_fee_percentage,
                flash_loan_pool_fee: flash_loan_pool_fee,
                launch: LaunchType::AlreadyExistingCoin,
                extracted_tickets: KeyValueStore::new_with_registered_type(),
                total_lp: Decimal::ZERO,
                total_users_lp: Decimal::ZERO,
                lp_resource_manager: lp_resource_manager,
                last_lp_id: 0,
                base_coins_to_lp_providers: Decimal::ZERO,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))

            // Restrict Pool access to the RadixPump component and the hooks
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
                hook => rule!(require(hook_badge_address));
            ))
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "name" => format!("{} pool", coin_name), updatable;
                    "dapp_definition" => dapp_definition, updatable;
                }
            })
            .globalize();

            (component_address, lp_resource_manager.address())
        }

        // Instantiate a pool component to manage a quick launch
        // The coin creator has to provide the base coins to initialize the pool.
        // He can decide the supply of the coin but he receives only a limited allocation depending on the base coin
        // deposit and the launch price.
        // After creation the pool is in Normal mode: no further actions are required
        pub fn new_quick_launch(

            // Component owner badge address
            owner_badge_address: ResourceAddress,

            // The badge RadixPump will use for authentication
            proxy_badge_address: ResourceAddress,

            // The badge round 0 and 1 hooks will use for authentication
            hook_badge_address: ResourceAddress,

            // Base coins to initialize the pool
            base_coin_bucket: Bucket,

            // Metadata for the coin to create
            coin_symbol: String,
            coin_name: String,
            coin_icon_url: String,
            coin_description: String,
            coin_info_url: String,
            coin_social_url: Vec<String>,

            // Supply and initial price of the coin
            coin_supply: Decimal,
            coin_price: Decimal,

            // Pool fees
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee: Decimal,

            // AccessRuleNode identifying the coin creator badge, it will be the owner of the coin
            // and the LP tokens
            coin_creator_badge_rule: AccessRuleNode,
            
            // dApp definition account address to use in components and resources
            dapp_definition: ComponentAddress,
        ) -> (
            ComponentAddress, // The created pool component
            Bucket, // Creator allocation of the new coin
            HookArgument, // Short description of the operation happened, to be used by hooks
            AnyPoolEvent, // QuickLaunchEvent
            ResourceAddress, // Resource address of the LP tokens
        ) {
            let (address_reservation, component_address) = Runtime::allocate_component_address(Pool::blueprint_id());

            // Mint the total supply of the new coin
            let mut coin_bucket = Pool::start_resource_manager_creation(
                coin_symbol,
                coin_name.clone(),
                coin_icon_url.clone(),
                coin_description,
                coin_info_url,
                coin_social_url,
                coin_creator_badge_rule.clone()
            )

            // Both the creator and this component can burn coins, the creator can change this
            // behaviour
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

            // All of the supply is minted at launch
            .mint_roles(mint_roles!(
                minter => rule!(deny_all);
                minter_updater => rule!(deny_all);
            ))
            .mint_initial_supply(coin_supply);

            let coin_address = coin_bucket.resource_address();

            // The creator gets his coins at the same price he sets as initial price
            let creator_amount = base_coin_bucket.amount() / coin_price;
            assert!(
                coin_supply >= dec!(2) * creator_amount,
                "Supply is too low",
            );
            let creator_coin_bucket = coin_bucket.take(creator_amount);

            // Ignore some coins so that the base coin / coin ratio in the pool maches the price
            let ignored_coins = coin_bucket.amount() - base_coin_bucket.amount() / coin_price;

            // Set an initial value for total_lp as the number of non ignored coins in the pool
            let total_lp = coin_bucket.amount() - ignored_coins;

            // Create the resource manager to mint the LP tokens
            let lp_resource_manager = Pool::lp_resource_manager(
                coin_name.clone(),
                UncheckedUrl::of(coin_icon_url),
                coin_creator_badge_rule,
                owner_badge_address,
                component_address,
                dapp_definition,
            );

            // Instantiate the component
            Self {
                base_coin_vault: Vault::with_bucket(base_coin_bucket),
                coin_vault: LoanSafeVault::with_bucket(coin_bucket.into()),
                mode: PoolMode::Normal,
                last_price: coin_price,
                buy_pool_fee_percentage: buy_pool_fee_percentage,
                sell_pool_fee_percentage: sell_pool_fee_percentage,
                flash_loan_pool_fee: flash_loan_pool_fee,
                launch: LaunchType::Quick(
                    QuickLaunchDetails {
                        ignored_coins: ignored_coins,
                    }
                ),
                extracted_tickets: KeyValueStore::new_with_registered_type(),
                total_lp: total_lp,
                total_users_lp: Decimal::ZERO,
                lp_resource_manager: lp_resource_manager,
                last_lp_id: 0,
                base_coins_to_lp_providers: Decimal::ZERO,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))

            // Access to methods is restricted to the RadixPump component and the hooks it invokes
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
                hook => rule!(require(hook_badge_address));
            ))

            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "name" => format!("{} pool", coin_name), updatable;
                    "dapp_definition" => dapp_definition, updatable;
                }
            })
            .globalize();

            (
                component_address,
                creator_coin_bucket.into(),

                // Prepare the argument to call the hooks
                HookArgument {
                    component: Runtime::global_address().into(),
                    coin_address: coin_address,
                    operation: HookableOperation::PostQuickLaunch,
                    amount: Some(coin_supply),
                    mode: PoolMode::Normal,
                    price: coin_price,
                    ids: vec![],
                },

                // Prepare the event the RadixPump component will emit
                AnyPoolEvent::QuickLaunchEvent(
                    QuickLaunchEvent {
                        resource_address: coin_address,
                        price: coin_price,
                        coins_in_pool: coin_supply - creator_amount,
                        creator_allocation: creator_amount,
                        buy_pool_fee_percentage: buy_pool_fee_percentage,
                        sell_pool_fee_percentage: sell_pool_fee_percentage,
                        flash_loan_pool_fee: flash_loan_pool_fee,
                    }
                ),
                lp_resource_manager.address(),
            )
        }

        // Instantiate a pool component to manage a random launch
        // During an initial launch phase users buy tickets.
        // At the end of the launch phase there's an extraction of the winnings tickets.
        // Winning tickets will receive a share of coins while losers get a refund.
        // The coin creator receives the equivalent of a winning ticket but his allocation is time locked.
        // The creator also receives the launch sale proceeds (fees excluded).
        // The creator will have to call the launch method to start the launch phase.
        pub fn new_random_launch(

            // Component owner badge address
            owner_badge_address: ResourceAddress,

            // The badge RadixPump will use for authentication
            proxy_badge_address: ResourceAddress,

            // The badge round 0 and 1 hooks will use for authentication
            hook_badge_address: ResourceAddress,

            // Metadata for the coin to create
            coin_symbol: String,
            coin_name: String,
            coin_icon_url: String,
            coin_description: String,
            coin_info_url: String,
            coin_social_url: Vec<String>,

            // The price of a launch ticket (fee included)
            ticket_price: Decimal,

            // The number of winning tickets
            winning_tickets: u32,

            // How many coins will a winning ticket receive
            coins_per_winning_ticket: Decimal,

            // Pool fees
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee: Decimal,

            // AccessRuleNode identifying the coin creator badge, it will be the owner of the coin,
            // the tickets and the LP tokens
            coin_creator_badge_rule: AccessRuleNode,

            // The base coin used to buy and sell the coin and to pay fees
            base_coin_address: ResourceAddress,

            // dApp definition account address to use in components and resources
            dapp_definition: ComponentAddress,
        ) -> (
            ComponentAddress, // The created component address
            ResourceAddress, // The coin resource address
            ResourceAddress, // The LP tokens resource address
        ) {
            let (address_reservation, component_address) = Runtime::allocate_component_address(Pool::blueprint_id());

            // Create a resource manager to mint tickets
            let ticket_resource_manager = ResourceBuilder::new_integer_non_fungible_with_registered_type::<TicketData>(
                OwnerRole::Updatable(AccessRule::Protected(coin_creator_badge_rule.clone()))
            )
            // Only this component can mint tickets
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(global_caller(component_address)));
            ))
            // Only this component can burn tickets
            .burn_roles(burn_roles!(
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(deny_all);
            ))
            // Everyone can deposit and withdraw tickets and the creator can't change this
            .deposit_roles(deposit_roles!(
                depositor => rule!(allow_all);
                depositor_updater => rule!(deny_all);
            ))
            .withdraw_roles(withdraw_roles!(
                withdrawer => rule!(allow_all);
                withdrawer_updater => rule!(deny_all);
            ))
            // No one can recall or freeze tickets and the creator can't change this
            .recall_roles(recall_roles!(
                recaller => rule!(deny_all);
                recaller_updater => rule!(deny_all);
            ))
            .freeze_roles(freeze_roles!(
                freezer => rule!(deny_all);
                freezer_updater => rule!(deny_all);
            ))
            // The creator can update tickets metadata
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
                    "dapp_definition" => dapp_definition, updatable;
                }
            ))
            .create_with_no_initial_supply();

            // Create a resource manager for badges to authenticate responses of the RandomComponent
            let random_badge_resource_manager = ResourceBuilder::new_fungible(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            // Only this component can mint random badges
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(deny_all);
            ))
            // Only this component can burn random badges
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
                    "dapp_definition" => dapp_definition, updatable;
                }
            ))
            .create_with_no_initial_supply();

            // Create the resource manager the mint the coins at the end of the launch phase
            let resource_manager = Pool::start_resource_manager_creation(
                coin_symbol,
                coin_name.clone(),
                coin_icon_url.clone(),
                coin_description,
                coin_info_url,
                coin_social_url,
                coin_creator_badge_rule.clone(),
            )
            // Only the creator can burn coins but he can also change this behaviour
            .burn_roles(burn_roles!(
                burner => AccessRule::Protected(coin_creator_badge_rule.clone());
                burner_updater => AccessRule::Protected(coin_creator_badge_rule.clone());
            ))
            // Only this component can mint coins, the mint happens at the end of the launch phase,
            // after that minting will be locked
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(global_caller(component_address)));
            ))
            .create_with_no_initial_supply();

            // Create a resource manager to mint LP tokens
            let lp_resource_manager = Pool::lp_resource_manager(
                coin_name.clone(),
                UncheckedUrl::of(coin_icon_url),
                coin_creator_badge_rule,
                owner_badge_address,
                component_address,
                dapp_definition,
            );

            // Instantiate the pool component
            Self {
                base_coin_vault: Vault::new(base_coin_address),
                coin_vault: LoanSafeVault::new(resource_manager.address()),
                mode: PoolMode::WaitingForLaunch,
                last_price: ticket_price / coins_per_winning_ticket,
                buy_pool_fee_percentage: buy_pool_fee_percentage,
                sell_pool_fee_percentage: sell_pool_fee_percentage,
                flash_loan_pool_fee: flash_loan_pool_fee,
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
                        buy_fee_during_launch: buy_pool_fee_percentage,
                    }
                ),
                extracted_tickets: KeyValueStore::new_with_registered_type(),
                total_lp: Decimal::ZERO,
                total_users_lp: Decimal::ZERO,
                lp_resource_manager: lp_resource_manager,
                last_lp_id: 1,
                base_coins_to_lp_providers: Decimal::ZERO,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            // Only RadixPump and the hooks he calls can access this component methods
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
                hook => rule!(require(hook_badge_address));
            ))
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "name" => format!("{} pool", coin_name), updatable;
                    "dapp_definition" => dapp_definition, updatable;
                }
            })
            .globalize();

            (component_address, resource_manager.address(), lp_resource_manager.address())
        }

// ONLY THE RandomComponent IS SUPPOSED TO CALL THE FOLLOWING METHODS AUTHENTICATION IS HANDLED BY A BADGE, MINTED BY
// random_badge_resource_manager, THAT THIS COMPONENT PASSES TO THE RandomComponent, EXPECT TO RECEIVE BACK AND BURNS

        // When random data are neeeded, at the end o the random launch phase, the RandomComponent
        // is invoked passing it a badge.
        // The RandomComponent, asynchronously, call this method to provide the random seed and
        // returns the random badge
        pub fn random_callback(
            &mut self,

            // The key specified in the RandomComponent invokation, we don't need that
            _key: u32,

            // The badge provided to the RandomComponent to authenticate this callback
            badge: FungibleBucket,

            // The random data
            random_seed: Vec<u8>
        ) {
            match self.launch {
                LaunchType::Random(ref mut random_launch) => {

                    // Check the badge and burn it
                    assert!(
                        badge.resource_address() == random_launch.random_badge_resource_manager.address() &&
                        badge.amount() == Decimal::ONE,
                        "Wrong badge",
                    );
                    badge.burn();

                    // How many tickets are we going to extract?
                    // No more than MAX_TICKETS_PER_OPERATION to avoid hitting transaction cost
                    // limits
                    // We can either extract winners or losers, which number is smaller
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

                    // Initialise the Random library
                    let mut random: Random = Random::new(&random_seed);

                    // For each ticket to extract
                    for _i in 0..tickets_to_extract {

                        // Generate a random number from zero to the number of sold tickets
                        // excluded (ticket ids start from zero)
                        let mut ticket_id = random.in_range::<u64>(0, random_launch.sold_tickets.into());

                        // If the number has already been extracted, try again
                        while self.extracted_tickets.get(&ticket_id).is_some() {
                            ticket_id = random.in_range::<u64>(0, random_launch.sold_tickets.into());
                        }

                        // Add the extracted number to the KVS specifying if it's a winner or a
                        // loser
                        self.extracted_tickets.insert(ticket_id, random_launch.extract_winners);
                    }

                    // Update the count of extracted tokens
                    random_launch.number_of_extracted_tickets += tickets_to_extract;

                },
                _ => Runtime::panic(TYPE_NOT_ALLOWED.to_string()),
            }
        }

        // If a call to random_callback fails, RandomComponent calls this method to notify it.
        // This method does nothing except burning the random badge: at the next terminate_launch
        // invocation more call to the RandomComponent will be issued if we didn't extract enough
        // tickets
        pub fn random_on_error(
            &self,

            // The key specified in the RandomComponent invokation, we don't need that
            _key: u32,

            // The badge provided to the RandomComponent to authenticate calls to random_callback
            // (and to this method too)
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
                _ => Runtime::panic(TYPE_NOT_ALLOWED.to_string()),
            }
        }

// PRIVATE METHODS AND FUNCTIONS

        // This private function contains the common part of the coin resource manager creation used
        // by 3 pool constructors
        fn start_resource_manager_creation(

            // Metadata for the coin to create
            coin_symbol: String,
            coin_name: String,
            coin_icon_url: String,
            coin_description: String,
            coin_info_url: String,
            coin_social_url: Vec<String>,

            // AccessRuleNode identifying the coin creator badge, it will be the owner of the coin
            coin_creator_badge_rule: AccessRuleNode,

        ) -> InProgressResourceBuilder<FungibleResourceType> // The resource manager is still
                                                             // missing some details that wil be
                                                             // added by the invoker
        {
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::Updatable(AccessRule::Protected(coin_creator_badge_rule.clone())))

            // Everybody can deposit and withdraw coins and the creator can't change this
            .deposit_roles(deposit_roles!(
                depositor => rule!(allow_all);
                depositor_updater => rule!(deny_all);
            ))
            .withdraw_roles(withdraw_roles!(
                withdrawer => rule!(allow_all);
                withdrawer_updater => rule!(deny_all);
            ))

            // No one can recall or freeze coins and the creator can't change this
            .recall_roles(recall_roles!(
                recaller => rule!(deny_all);
                recaller_updater => rule!(deny_all);
            ))
            .freeze_roles(freeze_roles!(
                freezer => rule!(deny_all);
                freezer_updater => rule!(deny_all);
            ))
            .divisibility(DIVISIBILITY_MAXIMUM);

            // TODO: Any intelligent way to do this?
            match coin_social_url.len() {
                0 => match coin_info_url.len() {
                    0 => resource_manager.metadata(metadata!(
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
                    _ => resource_manager.metadata(metadata!(
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
                },
                _ => {
                    let mut url: Vec<UncheckedUrl> = vec![];
                    for string in coin_social_url.iter() {
                        url.push(UncheckedUrl::of(string));
                    }

                    match coin_info_url.len() {
                        0 => resource_manager.metadata(metadata!(
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
                                "social_url" => url, updatable;
                            }
                        )),
                        _ => resource_manager.metadata(metadata!(
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
                                "social_url" => url, updatable;
                            }
                        )),
                    }
                },
            }
        }

        // Create a resource manager for minting LP tokens
        fn lp_resource_manager(

            // Metadata for the LP tokens
            coin_name: String,
            coin_icon_url: UncheckedUrl,

            // AccessRuleNode identifying the coin creator badge, it will be the owner of the LP
            // tokens
            coin_creator_badge_rule: AccessRuleNode,

            // Component owner badge
            owner_badge_address: ResourceAddress,

            // The address of the pool componet
            component_address: ComponentAddress,

            // dApp definition account address to set int he metadata
            dapp_definition: ComponentAddress,

        ) -> ResourceManager // The resource manager to mint the LP tokens
        {
            ResourceBuilder::new_integer_non_fungible_with_registered_type::<LPData>(
                OwnerRole::Updatable(AccessRule::Protected(coin_creator_badge_rule.clone()))
            )
            // Everybody can deposit and withdraw LP tokens and the creator can't change that
            .deposit_roles(deposit_roles!(
                depositor => rule!(allow_all);
                depositor_updater => rule!(deny_all);
            ))
            .withdraw_roles(withdraw_roles!(
                withdrawer => rule!(allow_all);
                withdrawer_updater => rule!(deny_all);
            ))

            // No one can recall or freeze LP tokens and the creator can't change that
            .recall_roles(recall_roles!(
                recaller => rule!(deny_all);
                recaller_updater => rule!(deny_all);
            ))
            .freeze_roles(freeze_roles!(
                freezer => rule!(deny_all);
                freezer_updater => rule!(deny_all);
            ))

            // Only this component can mint and burn LP tokens
            .burn_roles(burn_roles!(
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(require(owner_badge_address));
            ))
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(owner_badge_address));
            ))

            // The creator can set LP tokens metadata
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
                    "dapp_definition" => dapp_definition, updatable;
                }
            ))
            .create_with_no_initial_supply()
        }

        // This method is called at the end of the launch phase of a random launched coin
        fn terminate_random_launch(&mut self) -> (
            Option<Bucket>, // Sale proceeds for the creator
            Option<PoolMode>, // Notify RadixPump that now the mode is Normal
            Option<HookArgument>, // Short description of the operation happened, to be used in hooks invocation
            Option<AnyPoolEvent>, // RandomLaunchEndEvent to be issue by the RadixPump component
        ) {
            self.mode = PoolMode::Normal;

            match self.launch {
                LaunchType::Random(ref mut random_launch) => {

                    // The number od coins to mint correspond to the number of winning ticket + 2: one
                    // winning ticket is granted to the creator, one winning ticket is used to
                    // initialize the pool
                    let amount = random_launch.coins_per_winning_ticket * (random_launch.winning_tickets + 2);
                    let mut coin_bucket = random_launch.resource_manager.mint(amount);

                    // Lock the coins for the creator
                    random_launch.locked_vault.put(
                        coin_bucket.take(random_launch.coins_per_winning_ticket)
                    );

                    // Put in the pool the equivalent of one winning ticket
                    self.coin_vault.put(
                        coin_bucket.take(random_launch.coins_per_winning_ticket)
                    );

                    // Put all of the remaining coins in the vault where winners can claim them
                    random_launch.winners_vault.put(coin_bucket);

                    // No more mints
                    random_launch.resource_manager.set_mintable(rule!(deny_all));
                    random_launch.resource_manager.lock_mintable();

                    let supply = random_launch.resource_manager.total_supply();

                    // Take the proceeds of the sale (fees excluded) out of the pool
                    let base_coin_bucket = self.base_coin_vault.take_advanced(
                        random_launch.winning_tickets * random_launch.ticket_price * ((100 - random_launch.buy_fee_during_launch) / 100),
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    );
                    let base_coin_bucket_amount = base_coin_bucket.amount();

                    // The pool now cointains:
                    // - fees (base coins) paid by all ticket buyers (both winners and losers)
                    // - the equivalent of one winning ticket (coins)
                    // The price can be higer or lower than the launch phase, no guarantees!
                    self.last_price = self.base_coin_vault.amount() / random_launch.coins_per_winning_ticket;

                    // Initialize total_lp
                    self.total_lp = self.coin_vault.amount();

                    (
                        Some(base_coin_bucket),
                        Some(PoolMode::Normal),
                        Some(

                            // Argument for the hooks to call
                            HookArgument {
                                component: Runtime::global_address().into(),
                                coin_address: self.coin_vault.resource_address(),
                                operation: HookableOperation::PostTerminateRandomLaunch,
                                amount: supply,
                                mode: PoolMode::Normal,
                                price: self.last_price,
                                ids: vec![],
                            }
                        ),
                        Some(

                            // The RandomLaunchEndEvent that RadixPump will emit
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
                _ => Runtime::panic(SHOULD_NOT_HAPPEN.to_string()),
            }
        }

        // This private method asks randomness to the RandomComponent
        fn prepare_tickets_extraction(&mut self) {
            let mut calls_to_random: u32;
            let remainder: u32;

            match self.launch {
                LaunchType::Random(ref mut random_launch) => {

                    // Each call to random_callback will extract a maximum of MAX_TICKETS_PER_OPERATION tickets
                    // We can call the RandomComponent multiple times to extract all of the tickets
                    // Is it cheaper to extract winners or losers?
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

                    // Limit the number of calls to the RandomComponent to avoid hitting transaction limits
                    // The user will have to execute terminate_launch again if calls_to_random is
                    // not big enough
                    calls_to_random = min(calls_to_random, MAX_CALLS_TO_RANDOM);

                    // Mint all the badges we need
                    let mut random_badge_bucket = random_launch.random_badge_resource_manager.mint(Decimal::try_from(calls_to_random).unwrap());

                    // Call the RandomComponent multiple times passing a random badge each time
                    while random_badge_bucket.amount() >= Decimal::ONE {
                        RNG.request_random(
                            Runtime::global_address(),
                            "random_callback".to_string(),
                            "random_on_error".to_string(),
                            random_launch.key_random,
                            Some(random_badge_bucket.take(Decimal::ONE).as_fungible()),
                            10u8,
                        );

                        // Each request has a unique id even if we don't neet it
                        random_launch.key_random += 1;
                    }

                    // It's mandatory to burn the bucket even if it's empty
                    random_badge_bucket.burn();
                },
                _ => Runtime::panic(SHOULD_NOT_HAPPEN.to_string()),
            }
        }

        // Returns the non ignored number of coins in the pool
        fn coins_in_pool(&self) -> Decimal {
            match &self.launch {
                LaunchType::Quick(quick_launch) =>
                    self.coin_vault.amount() - quick_launch.ignored_coins,
                    
                // For non quick launched coins there are no ignored coins
                _ => self.coin_vault.amount(),
            }
        }

        // Update the number of ignored coins according to the price and the number of base coins
        // in the pool
        fn update_ignored_coins(&mut self) {
            match self.launch {
                LaunchType::Quick(ref mut quick_launch) =>

                    // Once ignored_coins reaches zero, it will be no longer updated
                    if quick_launch.ignored_coins > Decimal::ZERO {
                        quick_launch.ignored_coins = max(
                            self.coin_vault.amount() - self.base_coin_vault.amount() / self.last_price,
                            Decimal::ZERO,
                        );
                    }

                // For non quick launched coins do nothing
                _ => {}
            }
        }
    }
}
