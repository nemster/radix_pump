use scrypto::prelude::*;
use scrypto_interface::*;
use crate::pool::pool::*;

#[derive(Debug, ScryptoSbor, PartialEq, Clone, Copy)]
pub enum PoolMode {
    WaitingForLaunch,
    Launching,
    TerminatingLaunch,
    Normal,
    Liquidation,
    Uninitialised,
}

#[derive(ScryptoSbor)]
pub struct PoolInfo {
    pub component: Global<Pool>,
    pub base_coin_amount: Decimal,
    pub coin_amount: Decimal,
    pub last_price: Decimal,
    pub total_buy_fee_percentage: Decimal,
    pub total_sell_fee_percentage: Decimal,
    pub total_flash_loan_fee_percentage: Decimal,
    pub pool_mode: PoolMode,
    pub lp_resource_address: ResourceAddress,
    pub coin_lp_ratio: Decimal,
    pub end_launch_time: Option<i64>,
    pub unlocking_time: Option<i64>,
    pub initial_locked_amount: Option<Decimal>,
    pub unlocked_amount: Option<Decimal>,
    pub ticket_price: Option<Decimal>,
    pub winning_tickets: Option<u32>,
    pub coins_per_winning_ticket: Option<Decimal>,
    pub flash_loan_nft_resource_address: Option<ResourceAddress>,
    pub hooks_badge_resource_address: Option<ResourceAddress>,
    pub read_only_hooks_badge_resource_address: Option<ResourceAddress>,
}

#[derive(Debug, ScryptoSbor, NonFungibleData)]
pub struct CreatorData {
    pub id: u64,
    pub coin_resource_address: ResourceAddress,
    pub coin_name: String,
    pub coin_symbol: String,
    pub creation_date: Instant,
    #[mutable]
    pub pool_mode: PoolMode,
}

#[derive(Debug, ScryptoSbor, PartialEq, Clone, Copy)]
pub enum HookableOperation {
    PostFairLaunch,
    PostTerminateFairLaunch,
    PostQuickLaunch,
    PostRandomLaunch,
    PostTerminateRandomLaunch,
    PostBuy,
    PostSell,
    PostReturnFlashLoan,
    PostBuyTicket,
    PostRedeemWinningTicket,
    PostRedeemLousingTicket,
    PostAddLiquidity,
    PostRemoveLiquidity,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct FairLaunchStartEvent {
    pub resource_address: ResourceAddress,
    pub price: Decimal,
    pub creator_locked_percentage: Decimal,
    pub end_launch_time: i64,
    pub unlocking_time: i64,
    pub buy_pool_fee_percentage: Decimal,
    pub sell_pool_fee_percentage: Decimal,
    pub flash_loan_pool_fee_percentage: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct FairLaunchEndEvent {
    pub resource_address: ResourceAddress,
    pub creator_proceeds: Decimal,
    pub creator_locked_allocation: Decimal,
    pub supply: Decimal,
    pub coins_in_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct QuickLaunchEvent {
    pub resource_address: ResourceAddress,
    pub price: Decimal,
    pub coins_in_pool: Decimal,
    pub creator_allocation: Decimal,
    pub buy_pool_fee_percentage: Decimal,
    pub sell_pool_fee_percentage: Decimal,
    pub flash_loan_pool_fee_percentage: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct RandomLaunchStartEvent {
    pub resource_address: ResourceAddress,
    pub ticket_price: Decimal,
    pub winning_tickets: u32,
    pub coins_per_winning_ticket: Decimal,
    pub end_launch_time: i64,
    pub unlocking_time: i64,
    pub buy_pool_fee_percentage: Decimal,
    pub sell_pool_fee_percentage: Decimal,
    pub flash_loan_pool_fee_percentage: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct RandomLaunchEndEvent {
    pub resource_address: ResourceAddress,
    pub creator_proceeds: Decimal,
    pub creator_locked_allocation: Decimal,
    pub supply: Decimal,
    pub coins_in_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct BuyEvent {
    pub resource_address: ResourceAddress,
    pub mode: PoolMode,
    pub amount: Decimal,
    pub price: Decimal,
    pub coins_in_pool: Decimal,
    pub fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct SellEvent {
    pub resource_address: ResourceAddress,
    pub mode: PoolMode,
    pub amount: Decimal,
    pub price: Decimal,
    pub coins_in_pool: Decimal,
    pub fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct LiquidationEvent {
    pub resource_address: ResourceAddress,
    pub price: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct FlashLoanEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
    pub fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct BuyTicketEvent {
    pub resource_address: ResourceAddress,
    pub amount: u32,
    pub price: Decimal,
    pub ticket_resource_address: ResourceAddress,
    pub sold_tickets: u32,
    pub fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct FeeUpdateEvent {
    pub resource_address: ResourceAddress,
    pub buy_pool_fee_percentage: Decimal,
    pub sell_pool_fee_percentage: Decimal,
    pub flash_loan_pool_fee_percentage: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct BurnEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct AddLiquidityEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone)]
pub struct RemoveLiquidityEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(ScryptoSbor, Clone)]
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

#[derive(Debug, ScryptoSbor, NonFungibleData)]
pub struct TicketData {
    pub coin_resource_address: ResourceAddress,
    pub buy_date: Instant,
}

#[derive(Debug, ScryptoSbor, NonFungibleData)]
pub struct LPData {
    pub deposited_coins: Decimal,
    pub deposited_base_coins: Decimal,
    pub lp_share: Decimal,
    pub date: Instant,
    pub coin_resource_address: ResourceAddress,
}

#[derive(ScryptoSbor, Clone)]
pub struct HookArgument {
//TODO: why this doesn't work?    pub component: Global<Pool>,
    pub coin_address: ResourceAddress,
    pub operation: HookableOperation,
    pub amount: Option<Decimal>,
    pub mode: PoolMode,
    pub price: Option<Decimal>,
    pub ids: Vec<u64>,
}

// Hooks can be executed in three different rounds (0, 1 or 2)

// Round 0 hooks can recursively trigger more round 1 and 2 hooks calls while interacting with a Pool.
// A round 0 hook will never trigger the execution of another round 0 hook.

// Round 1 hook are executed after all of the round 0 hooks are done.
// If a round 1 hook returns one or more HookArgument, it is ignored: recursion is not happening.

// Round 2 hooks are executed once round 1 is completed and are not allowed to perform any state changing
// operation on the Pools.
pub type HookExecutionRound = usize;

define_interface! {
    Hook impl [ScryptoStub, Trait, ScryptoTestStub] {

        // Hook component instantiation is not performed by RadixPump; you should take care of it.
        // A hook component instantiation function should have a ResourceAddress parameter to set
        // the badge that will be used by the proxy; you can know this ResourceAddress by querying
        // the get_pool_info() method on the RadixPump component.
        // - hooks_badge_resource_address for rounds 0 and 1 hooks, it can be used to
        //   interact with any Pool component.
        // - read_only_hooks_badge_resource_address for round 2 hooks

        fn hook(
            &mut self,

            // This struct contains information about what caused the hook to be called
            argument: HookArgument,

            // This badge has two reasons:
            // - ensure the hook that RadixPump is calling it
            // - authenticate when calling a Pool method
            hook_badge_bucket: FungibleBucket,
        ) -> (
            // Return back the hook_badge_bucket
            FungibleBucket,

            // Any coin the hook wants to send to the user
            Option<Bucket>,

            // If the hook called a Pool method and it returned an event struct, return it to the
            // caller so it can be emitted
            Vec<AnyPoolEvent>,

            // A round 0 hook can also return informations about new hooks to be called
            Vec<HookArgument>,
        );

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
