use scrypto::prelude::*;
use std::cmp::Ordering;

#[derive(ScryptoSbor)]
pub struct LimitBuyOrder {
    base_coin_amount: Decimal,
    bought_amount: Decimal,
}

impl LimitBuyOrder {
    pub fn new(base_coin_amount: Decimal) -> LimitBuyOrder {
        assert!(
            base_coin_amount > Decimal::ZERO,
            "Base coin amount must be bigger than zero",
        );

        Self {
            base_coin_amount: base_coin_amount,
            bought_amount: Decimal::ZERO,
        }
    }

    pub fn get_base_coin_amount(&self) -> &Decimal {
        &self.base_coin_amount
    }

    pub fn get_bought_amount(&self) -> &Decimal {
        &self.bought_amount
    }

    pub fn fill(
        &mut self,
        price: Decimal,
    ) {
        assert!(
            price > Decimal::ZERO,
            "Price must be bigger than zero",
        );

        self.bought_amount += self.base_coin_amount / price;
        self.base_coin_amount = Decimal::ZERO;
    }

    pub fn partially_fill(
        &mut self,
        amount: Decimal,
        price: Decimal,
    ) {
        assert!(
            amount < self.base_coin_amount,
            "Amount bigger than the available one",
        );
        assert!(
            price > Decimal::ZERO,
            "Price must be bigger than zero",
        );

        self.base_coin_amount -= amount;
        self.bought_amount += amount / price;
    }

    pub fn coins_withdrawn(
        &mut self,
    ) {
        self.bought_amount = Decimal::ZERO;
    }
}

#[derive(ScryptoSbor, Eq, PartialOrd)]
pub struct LimitBuyOrderRef {
    id: u32,
    price: Decimal,
}

impl LimitBuyOrderRef {
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

    pub fn get_id(&self) -> &u32 {
        &self.id
    }

    pub fn get_price(&self) -> &Decimal {
        &self.price
    }
}

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
