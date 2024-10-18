use scrypto::prelude::*;

#[derive(Debug, ScryptoSbor, PartialEq, Clone, Copy)]
pub enum PoolMode {
    WaitingForLaunch,
    Launching,
    TerminatingLaunch,
    Normal,
    Liquidation,
}

#[derive(Debug, ScryptoSbor, PartialEq, Clone)]
pub struct PoolInfo {
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
