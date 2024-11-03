use scrypto::prelude::*;
use scrypto_interface::*;

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
    pub component: RadixPumpPoolInterfaceScryptoStub,
    pub base_coin_amount: Decimal,
    pub coin_amount: Decimal,
    pub last_price: Decimal,
    pub total_buy_fee_percentage: Decimal,
    pub total_sell_fee_percentage: Decimal,
    pub total_flash_loan_fee: Decimal,
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
    pub coin_resource_address: ResourceAddress,
    pub coin_name: String,
    pub coin_symbol: String,
    pub creation_date: Instant,
    pub lp_token_address: ResourceAddress,
    pub key_image_url: Url,
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

#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct FairLaunchEndEvent {
    pub resource_address: ResourceAddress,
    pub creator_proceeds: Decimal,
    pub creator_locked_allocation: Decimal,
    pub supply: Decimal,
    pub coins_in_pool: Decimal,
}

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

#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct RandomLaunchEndEvent {
    pub resource_address: ResourceAddress,
    pub creator_proceeds: Decimal,
    pub creator_locked_allocation: Decimal,
    pub supply: Decimal,
    pub coins_in_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct BuyEvent {
    pub resource_address: ResourceAddress,
    pub mode: PoolMode,
    pub amount: Decimal,
    pub price: Decimal,
    pub coins_in_pool: Decimal,
    pub fee_paid_to_the_pool: Decimal,
    pub integrator_id: u64,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct SellEvent {
    pub resource_address: ResourceAddress,
    pub mode: PoolMode,
    pub amount: Decimal,
    pub price: Decimal,
    pub coins_in_pool: Decimal,
    pub fee_paid_to_the_pool: Decimal,
    pub integrator_id: u64,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct LiquidationEvent {
    pub resource_address: ResourceAddress,
    pub price: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct FlashLoanEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
    pub fee_paid_to_the_pool: Decimal,
    pub integrator_id: u64,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct BuyTicketEvent {
    pub resource_address: ResourceAddress,
    pub amount: u32,
    pub price: Decimal,
    pub ticket_resource_address: ResourceAddress,
    pub sold_tickets: u32,
    pub fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct FeeUpdateEvent {
    pub resource_address: ResourceAddress,
    pub buy_pool_fee_percentage: Decimal,
    pub sell_pool_fee_percentage: Decimal,
    pub flash_loan_pool_fee: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct BurnEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct AddLiquidityEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Copy)]
pub struct RemoveLiquidityEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

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
    pub component: RadixPumpPoolInterfaceScryptoStub,
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

define_interface! {
    RadixPumpPool impl [ScryptoStub, Trait, ScryptoTestStub] {

// THE FOLLOWING METHOD REQUIRES NO AUTHENTICATION, IT CAN BE CALLED BY ANYONE

        fn get_pool_info(&self) -> PoolInfo;

// THE FOLLOWING METHODS CAN ONLY BE CALLED BY ROUND 0 AND 1 HOOKS AND BY THE RadixPump COMPONENT

        fn buy(
            &mut self,
            base_coin_bucket: Bucket,
        ) -> (
            Bucket,
            HookArgument,
            AnyPoolEvent,
        );

        fn sell(
            &mut self,
            coin_bucket: Bucket,
        ) -> (
            Bucket,
            HookArgument,
            AnyPoolEvent,
        );

        fn buy_ticket(
            &mut self,
            amount: u32,
            base_coin_bucket: Bucket,
        ) -> (
            Bucket,
            HookArgument,
            AnyPoolEvent,
        );

        fn redeem_ticket(
            &mut self,
            ticket_bucket: Bucket,
        ) -> (
            Bucket, // base coin bucket
            Option<Bucket>, // coin bucket
            Option<HookArgument>,
            Option<HookArgument>,
        );

        fn add_liquidity(
            &mut self,
            base_coin_bucket: Bucket,
            coin_bucket: Bucket,
        ) -> (  
            Bucket,
            Option<Bucket>,
            HookArgument, 
            AnyPoolEvent,
            Option<PoolMode>,
        );

        fn remove_liquidity(
            &mut self,
            lp_bucket: Bucket,
        ) -> (  
            Bucket, 
            Option<Bucket>,
            HookArgument,
            AnyPoolEvent,
        );

// THE FOLLOWING METHODS CAN ONLY BE CALLED BY THE RadixPump COMPONENT

        fn launch(
            &mut self, 
            end_launch_time: i64,
            unlocking_time: i64,
        ) -> (
            PoolMode,
            HookArgument,
            AnyPoolEvent,
        );

        fn terminate_launch(&mut self) -> (
            Option<Bucket>,
            Option<PoolMode>,
            Option<HookArgument>,
            Option<AnyPoolEvent>,
        );

        fn unlock(
            &mut self,
            amount: Option<Decimal>,
        ) -> Bucket;

        fn set_liquidation_mode(&mut self) -> (
            PoolMode,
            AnyPoolEvent,
        );

        fn get_flash_loan(
            &mut self,
            amount: Decimal,
        ) -> Bucket;

        fn return_flash_loan(
            &mut self,
            base_coin_bucket: Bucket,
            coin_bucket: Bucket,
        ) -> (
            HookArgument,
            AnyPoolEvent,
        );

        fn update_pool_fees(
            &mut self,
            buy_pool_fee_percentage: Decimal,
            sell_pool_fee_percentage: Decimal,
            flash_loan_pool_fee: Decimal,
        ) -> AnyPoolEvent;

        fn burn(
            &mut self,
            amount: Decimal,
        ) -> AnyPoolEvent;
    }
}
