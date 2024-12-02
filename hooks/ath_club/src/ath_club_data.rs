use scrypto::prelude::*;

// NonFungibleData for the ATH Club NFT
#[derive(ScryptoSbor, NonFungibleData)]
pub struct AthClubData {
    pub coin_address: ResourceAddress,
    pub coin_symbol: String,
    pub price: Decimal,
    pub date: Instant,
    pub key_image_url: Url,
    #[mutable]
    pub obsoleted_by: u64,
}
