use scrypto::prelude::*;
use std::cmp::Ordering;

// This struct contains the minimal information needed to sort the orders in the active_orders list
// The full informations about an order are in the NFT so the user can see them
#[derive(ScryptoSbor, Eq, PartialOrd)]
pub struct LimitBuyOrderRef {
    id: u32,
    price: Decimal,
}

impl LimitBuyOrderRef {

    // Instantiate a LimitBuyOrderRef
    pub fn new(
        id: u32,
        price: Decimal,
    ) -> LimitBuyOrderRef {
        assert!(
            price > Decimal::ZERO,
            "Price must be bigger than zero",
        );

        Self {
            id: id,
            price: price,
        }
    }

    // Get the order id
    pub fn get_id(&self) -> &u32 {
        &self.id
    }

    // Get the order desired price
    pub fn get_price(&self) -> &Decimal {
        &self.price
    }
}

// PartialEq and Eq traits implementation
impl PartialEq for LimitBuyOrderRef {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

// LimitBuyOrderRef are sortable by (price, reverse id)
impl Ord for LimitBuyOrderRef {
    fn cmp(&self, other: &Self) -> Ordering {
        let price_cmp = self.price.cmp(&other.price);

        if price_cmp == Ordering::Equal {
           other.id.cmp(&self.id)
        } else {
            price_cmp
        }
    }
}
