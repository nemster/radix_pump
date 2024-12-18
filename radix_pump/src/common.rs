use scrypto::prelude::*;
use scrypto_interface::*;

// Internal state of a pool component
#[derive(Debug, ScryptoSbor, PartialEq, Clone, Copy)]
pub enum PoolMode {
    WaitingForLaunch,  // FairLaunch or RandomLaunch not started yet
    Launching,         // FairLaunch or RandomLaunch started
    TerminatingLaunch, // RandomLaunch extracting winners
    Normal,            // Normal operation
    Liquidation,       // Liquidation mode
    Uninitialised,     // Pool created for a pre existing coin without adding liquidity
}

// Info about the state of a pool
// The get_pool_info methods of both RadixPump and Pool components return this struct, but the one
// returned by the Pool component is missing some information
#[derive(ScryptoSbor)]
pub struct PoolInfo {
    // Pool component address
    pub component: RadixPumpPoolInterfaceScryptoStub,

    // Amount of base coins in the pool vault
    pub base_coin_amount: Decimal,

    // Amount of non ignored coins in the pool vault
    pub coin_amount: Decimal,

    // The price of the last buy or sell operation
    pub last_price: Decimal,

    // The current price based on the vault amount ratio
    pub price: Decimal,

    // Amount of non ingnored or locked coins
    pub circulating_supply: Decimal,

    // When calling the Pool get_pool_info method, these are the pool fees
    // When calling the RadixPump get_pool_info method, these are the total fees (owner/integrator
    // + pool)
    pub total_buy_fee_percentage: Decimal,
    pub total_sell_fee_percentage: Decimal,
    pub total_flash_loan_fee: Decimal,

    // Pool mode (see above)
    pub pool_mode: PoolMode,

    // Resource address of the liquidity non fungible token
    // The non fungible data of this token is the struct LPData
    pub lp_resource_address: ResourceAddress,

    // Non ignored coins in pool / base coins in pool
    pub coin_lp_ratio: Decimal,

    // Timings for FairLaunch and RandomLaunch coins
    pub end_launch_time: Option<i64>,
    pub unlocking_time: Option<i64>,

    // Amount of coins in the creator allocation when launch terminates (FairLaunch and
    // RandomLaunch only)
    pub initial_locked_amount: Option<Decimal>,

    // Creator allocation withdrawed so far (FairLaunch and RandomLaunch only)
    pub unlocked_amount: Option<Decimal>,

    // Price of a ticket for coins extraction (RandomLaunch only)
    pub ticket_price: Option<Decimal>,

    // Number of winning ticket that will be extracted (RandomLaunch only)
    pub winning_tickets: Option<u32>,

    // How many coins a winning ticket will receive (RandomLaunch only)
    pub coins_per_winning_ticket: Option<Decimal>,

    // When calling the Pool get_pool_info method, None
    // When calling the RadixPump get_pool_info method, the resource address of the transient NFT
    // used to guarantee the flash loan return
    // This is the same for all of the pools
    pub flash_loan_nft_resource_address: Option<ResourceAddress>,

    // When calling the Pool get_pool_info method, None
    // When calling the RadixPump get_pool_info method, the resource address of the badge that
    // RadixPump passes to round 0 and 1 hooks and that the hooks can use to
    // authenticate against a Pool
    // This is the same for all of the pools
    pub hooks_badge_resource_address: Option<ResourceAddress>,

    // When calling the Pool get_pool_info method, None
    // When calling the RadixPump get_pool_info method, the resource address of the coin creator
    // badge
    // This is the same for all of the pools
    // The non fungible data of this token is the struct CreatorData
    pub creator_badge_resource_address: Option<ResourceAddress>,
}

// Non fungible data for the creator badges
#[derive(Debug, ScryptoSbor, NonFungibleData)]
pub struct CreatorData {
    pub coin_resource_address: ResourceAddress,
    pub coin_name: String,
    pub coin_symbol: String,
    pub creation_date: Instant,
    pub lp_token_address: ResourceAddress,
    pub key_image_url: Url,
    #[mutable]
    pub pool_mode: PoolMode,
}

#[derive(Debug, ScryptoSbor, PartialEq)]
pub enum TaskStatus {
    OK,
    LowGas,
    HookUnregistered,
}

// Non fungible data for scheduled operations
#[derive(Debug, ScryptoSbor, NonFungibleData)]
pub struct TimerBadge {
    #[mutable]
    pub minute: String,
    #[mutable]
    pub hour: String,
    #[mutable]
    pub day_of_month: String,
    #[mutable]
    pub month: String,
    #[mutable]
    pub day_of_week: String,
    #[mutable]
    pub random_delay: u32,
    #[mutable]
    pub status: TaskStatus,
    pub hook: String,
    pub coin_address: ResourceAddress,
}

// List of pool operations a hook can be attached to
#[derive(Debug, ScryptoSbor, PartialEq, Clone, Copy)]
pub enum HookableOperation {
    FairLaunch,             // launch method
    TerminateFairLaunch,    // terminate_launch method
    QuickLaunch,            // new_quick_launch method
    RandomLaunch,           // launch method
    TerminateRandomLaunch,  // terminate_launch method (last invocation)
    Buy,                    // buy method
    Sell,                   // sell method
    ReturnFlashLoan,        // return_flash_loan method
    BuyTicket,              // buy_ticket method
    RedeemWinningTicket,    // redeem_ticket method
    RedeemLosingTicket,     // redeem ticket method
    AddLiquidity,           // add_liquidity method
    RemoveLiquidity,        // remove_liquidity method
    Timer,                  // used by the Timer component
}

// Event created by a pool launch method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct FairLaunchStartEvent {
    pub resource_address: ResourceAddress,
    pub price: Decimal,
    pub creator_locked_percentage: Decimal,
    pub end_launch_time: i64,
    pub unlocking_time: i64,
    pub buy_pool_fee_percentage: Decimal,
    pub sell_pool_fee_percentage: Decimal,
    pub flash_loan_pool_fee: Decimal,
}

// Event created by a pool terminate_launch method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct FairLaunchEndEvent {
    pub resource_address: ResourceAddress,
    pub creator_proceeds: Decimal,
    pub creator_locked_allocation: Decimal,
    pub supply: Decimal,
    pub coins_in_pool: Decimal,
}

// Event created by a pool new_quick_launch method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct QuickLaunchEvent {
    pub resource_address: ResourceAddress,
    pub price: Decimal,
    pub coins_in_pool: Decimal,
    pub creator_allocation: Decimal,
    pub buy_pool_fee_percentage: Decimal,
    pub sell_pool_fee_percentage: Decimal,
    pub flash_loan_pool_fee: Decimal,
}

// Event created by a pool launch method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct RandomLaunchStartEvent {
    pub resource_address: ResourceAddress,
    pub ticket_price: Decimal,
    pub winning_tickets: u32,
    pub coins_per_winning_ticket: Decimal,
    pub end_launch_time: i64,
    pub unlocking_time: i64,
    pub buy_pool_fee_percentage: Decimal,
    pub sell_pool_fee_percentage: Decimal,
    pub flash_loan_pool_fee: Decimal,
}

// Event created by a pool terminate_launch method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct RandomLaunchEndEvent {
    pub resource_address: ResourceAddress,
    pub creator_proceeds: Decimal,
    pub creator_locked_allocation: Decimal,
    pub supply: Decimal,
    pub coins_in_pool: Decimal,
}

// Event created by a pool buy method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct BuyEvent {
    pub resource_address: ResourceAddress,
    pub mode: PoolMode,
    pub amount: Decimal,
    pub price: Decimal,
    pub coins_in_pool: Decimal,
    pub fee_paid_to_the_pool: Decimal,
    pub integrator_id: u64,
    pub circulating_supply: Decimal,
}

// Event created by a pool sell method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct SellEvent {
    pub resource_address: ResourceAddress,
    pub mode: PoolMode,
    pub amount: Decimal,
    pub price: Decimal,
    pub coins_in_pool: Decimal,
    pub fee_paid_to_the_pool: Decimal,
    pub integrator_id: u64,
    pub circulating_supply: Decimal,
}

// Event created by a pool set_liquidation_mode method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct LiquidationEvent {
    pub resource_address: ResourceAddress,
    pub price: Decimal,
}

// Event created by a pool return_flash_loan method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct FlashLoanEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
    pub fee_paid_to_the_pool: Decimal,
    pub integrator_id: u64,
}

// Event created by a pool buy_ticket method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct BuyTicketEvent {
    pub resource_address: ResourceAddress,
    pub amount: u32,
    pub price: Decimal,
    pub ticket_resource_address: ResourceAddress,
    pub sold_tickets: u32,
    pub fee_paid_to_the_pool: Decimal,
}

// Event created by a pool update_pool_fees method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct FeeUpdateEvent {
    pub resource_address: ResourceAddress,
    pub buy_pool_fee_percentage: Decimal,
    pub sell_pool_fee_percentage: Decimal,
    pub flash_loan_pool_fee: Decimal,
}

// Event created by a pool burn method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct BurnEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

// Event created by a pool add_liquidity method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct AddLiquidityEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
    pub lp_id: u64,
    pub coins_in_pool: Decimal,
}

// Event created by a pool remove_liquidity method
#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct RemoveLiquidityEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
    pub coins_in_pool: Decimal,
}

// A wrapper for any event a pool can generate
#[derive(ScryptoSbor, Clone, Copy)]
pub enum AnyPoolEvent {
    FairLaunchStartEvent(FairLaunchStartEvent),
    FairLaunchEndEvent(FairLaunchEndEvent),
    QuickLaunchEvent(QuickLaunchEvent),
    RandomLaunchStartEvent(RandomLaunchStartEvent),
    RandomLaunchEndEvent(RandomLaunchEndEvent),
    BuyEvent(BuyEvent),
    SellEvent(SellEvent),
    LiquidationEvent(LiquidationEvent),
    FlashLoanEvent(FlashLoanEvent),
    BuyTicketEvent(BuyTicketEvent),
    FeeUpdateEvent(FeeUpdateEvent),
    BurnEvent(BurnEvent),
    AddLiquidityEvent(AddLiquidityEvent),
    RemoveLiquidityEvent(RemoveLiquidityEvent),
}

// Non fungible data for the ticket NFT used in RandomLaunch
#[derive(Debug, ScryptoSbor, NonFungibleData)]
pub struct TicketData {
    pub coin_resource_address: ResourceAddress,
    pub buy_date: Instant,
}

// Non fungible data for the liquidity tokens of a pool
#[derive(Debug, ScryptoSbor, NonFungibleData)]
pub struct LPData {
    pub deposited_coins: Decimal,
    pub deposited_base_coins: Decimal,
    pub lp_share: Decimal,
    pub date: Instant,
    pub coin_resource_address: ResourceAddress,
}

// This is a brief description of an opertation done on a pool, this argument is passed to a hook
// to let it know why it was invoked
#[derive(ScryptoSbor, Clone)]
pub struct HookArgument {
    pub component: RadixPumpPoolInterfaceScryptoStub,
    pub coin_address: ResourceAddress,
    pub operation: HookableOperation,

    // The meaning of the amount field depends on the operation:
    // if Buy it is the bought amount of coins
    // if Sell it is the sold amount of coins
    // if BuyTicket it is the number of bought tickets (integer)
    // if RedeemLosingTicket it is the number of losing tickets reedemed (integer)
    // if RedeemWinningTicket it is the number of winning tickets reedemed (integer)
    // if AddLiquidity it is the amount of coins added to the pool
    // if RemoveLiquidity it is the amount of coins withdrawn from the pool
    // if FairLaunch or RandomLaunch it is None
    // if TerminateFairLaunch or QuickLaunch or TerminateRandomLaunch it is the total supply of the coin
    // if ReturnFlashLoan it is the amount of coins returned
    pub amount: Option<Decimal>,

    pub mode: PoolMode,
    pub price: Decimal,

    // The meaning of the ids field depends on the operation:
    // if BuyTicket it is the list of ids of the bought tickets
    // if RedeemLosingTicket it is the list of ids of the redeemed losing tickets
    // if RedeemWinningTicket it is the list of ids of the redeemed winning tickets
    // if AddLiquidity it is the id of the minted liquidity token
    // if RemoveLiquidity it is the list of the ids of the burned liquidity tokens
    // if Timer it is the id of the TimerBadge
    // in any other case it is just an empty array
    pub ids: Vec<u64>,
}

/* Hooks can be executed in three different rounds (0, 1 or 2)

   Round 0 hooks can recursively trigger more round 1 and 2 hooks calls while interacting with a Pool.
   A round 0 hook will never trigger the execution of another round 0 hook.

   Round 1 hook are executed after all of the round 0 hooks are done.
   If a round 1 hook returns one or more HookArgument, it is ignored: recursion is not happening.

  Round 2 hooks are executed once round 1 is completed and are not allowed to perform any state changing
  operation on the Pools.
*/
pub type HookExecutionRound = usize;

// Scripto-interface for all of the hook blueprints
define_interface! {
    Hook impl [ScryptoStub, Trait, ScryptoTestStub] {

        // This is the method called by RadixPump when a relevant operation is performed
        // RadixPump uses his proxy badge when calling this method, you must ask for the address
        // of the proxy badge in the hook instantiation and restrict this method with it.
        fn hook(
            &mut self,

            // This struct contains information about what caused the hook to be called
            argument: HookArgument,

            // This badge is only passed to round 0 and 1 hooks, it can be used to call Poll
            // methods
            hook_badge_bucket: Option<FungibleBucket>,
        ) -> (
            // Return back the hook_badge_bucket
            Option<FungibleBucket>,

            // Any coin the hook wants to send to the user
            Option<Bucket>,

            // If the hook called a Pool method and it returned an event struct, return it to the
            // caller so it can be emitted
            Vec<AnyPoolEvent>,

            // A round 0 hook can also return informations about new hooks to be called
            Vec<HookArgument>,
        );

        // This method is called during the hook registration, it must provide information about
        // the execution of this hook
        fn get_hook_info(&self) -> (

            // Which round wants this hook be executed?
            // Any number bigger than 2 will cause an exception when the hook is registered,
            HookExecutionRound,

            // Wheter other hooks can trigger or not the execution of this hook.
            // For round 0 hooks this must be false.
            bool,
        );
    }
}

// Scrypto-interface for the Pool components
define_interface! {
    RadixPumpPool impl [ScryptoStub, Trait, ScryptoTestStub] {

// THE FOLLOWING METHOD REQUIRES NO AUTHENTICATION, IT CAN BE CALLED BY ANYONE

        // Return detailed information about the pool status
        fn get_pool_info(&self) -> PoolInfo;

// THE FOLLOWING METHODS CAN ONLY BE CALLED BY ROUND 0 AND 1 HOOKS AND BY THE RadixPump COMPONENT

        // Call this method to buy coins with base coins
        fn buy(
            &mut self,
            base_coin_bucket: FungibleBucket,
        ) -> (
            FungibleBucket,
            HookArgument,
            AnyPoolEvent,
        );

        // Call this method to sell coins for base coins
        fn sell(
            &mut self,
            coin_bucket: FungibleBucket,
        ) -> (
            FungibleBucket,
            HookArgument,
            AnyPoolEvent,
        );

        // Call this method to buy tickets for a reandom launched coin launch phase
        fn buy_ticket(
            &mut self,
            amount: u32,
            base_coin_bucket: FungibleBucket,
        ) -> (
            Bucket,
            NonFungibleBucket,
            HookArgument,
            AnyPoolEvent,
        );

        // Users who bought ticket for a random launched coin can call this method to get their
        // coins (if winners) or a refund (if losers)
        fn redeem_ticket(
            &mut self,
            ticket_bucket: Bucket,
        ) -> (
            FungibleBucket, // base coin bucket
            Option<FungibleBucket>, // coin bucket
            Option<HookArgument>,
            Option<HookArgument>,
        );

        // Users can add liquidity to the pool by calling this method.
        // Both base coins and coins must be provided
        fn add_liquidity(
            &mut self,
            base_coin_bucket: Bucket,
            coin_bucket: Bucket,
        ) -> (  
            NonFungibleBucket,
            Option<Bucket>,
            HookArgument, 
            AnyPoolEvent,
            Option<PoolMode>,
        );

        // Users can invoke this method to get back the proviously added liquidity
        fn remove_liquidity(
            &mut self,
            lp_bucket: Bucket,
        ) -> (  
            FungibleBucket, 
            Option<FungibleBucket>,
            HookArgument,
            AnyPoolEvent,
        );

// THE FOLLOWING METHODS CAN ONLY BE CALLED BY THE RadixPump COMPONENT

        // The creator of a fair or random launched coin can use this method to start the launch
        // phase
        fn launch(
            &mut self, 
            end_launch_time: i64,
            unlocking_time: i64,
        ) -> (
            PoolMode,
            HookArgument,
            AnyPoolEvent,
        );

        // The creator of a fair or random launched coin can use this method to terminate the launch
        // phase            
        // In case of a fair launch the mode goes from Launching to Normal in just one step
        // In case of a random launch there are 4 possibilities:
        // - Launching -> Normal (if sold tickets <= winning tickets, everybody won)
        // - Launching -> TerminatingLaunch (request random data to the RandomComponent)
        // - TerminatingLaunch -> TerminatingLaunch (request more random data to the RandomComponent)
        // - TerminatingLaunch -> Normal (tickets extraction completed)
        fn terminate_launch(&mut self) -> (
            Option<FungibleBucket>,
            Option<PoolMode>,
            Option<HookArgument>,
            Option<AnyPoolEvent>,
        );

        // The creator of a fair or random launched coin can use this method to get (part of) his
        // allocation
        // The creator allocation is time locked and has an unlock schedule linear with time
        fn unlock(
            &mut self,
            amount: Option<Decimal>,
        ) -> FungibleBucket;

        // Call this method to put the pool in Liquidation mode (authentication is in RadixPump
        // component)
        // The goal of the liquidation mode is to get all of the coins into the pool and all of the
        // base coins out of the pool
        fn set_liquidation_mode(&mut self) -> (
            PoolMode,
            AnyPoolEvent,
        );

        // Get a flash loan. It is RadixPump responsibility to ensure the loan will be returned
        fn get_flash_loan(
            &mut self,
            amount: Decimal,
        ) -> FungibleBucket;

        // Return a previoulsy received flash loan
        fn return_flash_loan(
            &mut self,
            base_coin_bucket: Bucket,
            coin_bucket: Bucket,
        ) -> (
            HookArgument,
            AnyPoolEvent,
        );

        // The coin creator can use this method to update poll fees, user authentication is managed by RadixPump
        fn update_pool_fees(
            &mut self,
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee: Decimal,
        ) -> AnyPoolEvent;

        // The creator of a quick launched coin can use this method to burn excess coins in the
        // pool. User authentication is managed bu RadixPump
        fn burn(
            &mut self,
            amount: Decimal,
        ) -> AnyPoolEvent;
    }
}
