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
    pub end_launch_time: Option<i64>,
    pub unlocking_time: Option<i64>,
    pub initial_locked_amount: Option<Decimal>,
    pub unlocked_amount: Option<Decimal>,
    pub ticket_price: Option<Decimal>,
    pub winning_tickets: Option<u32>,
    pub coins_per_winning_ticket: Option<Decimal>,
    pub flash_loan_nft_resource_address: ResourceAddress,
    pub hooks_badge_resource_address: ResourceAddress,
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
}

#[derive(ScryptoSbor, ScryptoEvent)]
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

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FairLaunchEndEvent {
    pub resource_address: ResourceAddress,
    pub creator_proceeds: Decimal,
    pub creator_locked_allocation: Decimal,
    pub supply: Decimal,
    pub coins_in_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct QuickLaunchEvent {
    pub resource_address: ResourceAddress,
    pub price: Decimal,
    pub coins_in_pool: Decimal,
    pub creator_allocation: Decimal,
    pub buy_pool_fee_percentage: Decimal,
    pub sell_pool_fee_percentage: Decimal,
    pub flash_loan_pool_fee_percentage: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
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

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct RandomLaunchEndEvent {
    pub resource_address: ResourceAddress,
    pub creator_proceeds: Decimal,
    pub creator_locked_allocation: Decimal,
    pub supply: Decimal,
    pub coins_in_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct BuyEvent {
    pub resource_address: ResourceAddress,
    pub mode: PoolMode,
    pub amount: Decimal,
    pub price: Decimal,
    pub coins_in_pool: Decimal,
    pub fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct SellEvent {
    pub resource_address: ResourceAddress,
    pub mode: PoolMode,
    pub amount: Decimal,
    pub price: Decimal,
    pub coins_in_pool: Decimal,
    pub fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct LiquidationEvent {
    pub resource_address: ResourceAddress,
    pub price: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FlashLoanEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
    pub fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct BuyTicketEvent {
    pub resource_address: ResourceAddress,
    pub amount: u32,
    pub price: Decimal,
    pub ticket_resource_address: ResourceAddress,
    pub sold_tickets: u32,
    pub fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FeeUpdateEvent {
    pub resource_address: ResourceAddress,
    pub buy_pool_fee_percentage: Decimal,
    pub sell_pool_fee_percentage: Decimal,
    pub flash_loan_pool_fee_percentage: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct BurnEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(ScryptoSbor)]
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
}

#[derive(Debug, ScryptoSbor, NonFungibleData)]
pub struct TicketData {
    pub coin_resource_address: ResourceAddress,
    pub buy_date: Instant,
}

#[derive(ScryptoSbor, Clone)]
pub struct HookArgument {
    pub component: Global<Pool>,
    pub coin_address: ResourceAddress,
    pub operation: HookableOperation,
    pub amount: Option<Decimal>,
    pub mode: PoolMode,
    pub price: Option<Decimal>,
}

define_interface! {
    Hook impl [ScryptoStub, Trait, ScryptoTestStub] {
        fn hook(
            &self,
            argument: HookArgument,
            hook_badge_bucket: FungibleBucket,
        ) -> (
            FungibleBucket,
            Option<Bucket>
        );
    }
}
