use scrypto::prelude::*;
use std::cmp::Ordering;
use std::rc::Rc;

#[derive(ScryptoSbor, Eq, PartialOrd)]
pub struct Order {
    id: u32,
    price: Decimal,
    base_coin_amount: Decimal,
    bought_amount: Decimal,
}

impl Order {
    pub fn new(
        id: u32,
        price: Decimal,
        base_coin_amount: Decimal
    ) -> Order {
        assert!(
            price > Decimal::ZERO,
            "Price must be bigger than zero",
        );
        assert!(
            base_coin_amount > Decimal::ZERO,
            "Base coin amount must be bigger than zero",
        );

        Self {
            id: id,
            price: price,
            base_coin_amount: base_coin_amount,
            bought_amount: Decimal::ZERO,
        }
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

// Orders are sortable by (price, reverse id)
impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        let price_cmp = self.price.cmp(&other.price);

        if price_cmp == Ordering::Equal {
           other.id.cmp(&self.id)
        } else {
            price_cmp
        }
    }
}

#[derive(ScryptoSbor, Eq, PartialOrd)]
pub struct OrderRef(Rc<RefCell<Order>>);

impl OrderRef {
    pub fn get_id(&self) -> u32 {
        self.0.borrow().id
    }

    pub fn get_price(&self) -> Decimal {
        self.0.borrow().price
    }

    pub fn get_base_coin_amount(&self) -> Decimal {
        self.0.borrow().base_coin_amount
    }
}

impl OrderRef {

    pub fn new(
        id: u32,
        price: Decimal,
        base_coin_amount: Decimal,
    ) -> OrderRef {
        let order = Order::new(id, price, base_coin_amount);

        Self(Rc::new(RefCell::new(order)))
    }

    pub fn swap(
        &mut self,
        amount: Decimal,
        ratio: Decimal,
    ) {
        let mut order = self.0.borrow_mut();

        order.bought_amount += ratio * amount;
        order.base_coin_amount -= amount;
info!("swap #{}#, bought_amount: {}, base_coin_amount: {}, strong: {}, weak: {}", order.id, order.bought_amount, order.base_coin_amount, Rc::strong_count(&self.0), Rc::weak_count(&self.0));
    }

    pub fn swap_all(
        &mut self,
        ratio: Decimal,
    ) {
        let mut order = self.0.borrow_mut();

        let order_base_coin_amount = order.base_coin_amount;
        order.bought_amount += ratio * order_base_coin_amount;
        order.base_coin_amount = Decimal::ZERO;
info!("swap_all #{}#, bought_amount: {}, base_coin_amount: {}, strong: {}, weak: {}", order.id, order.bought_amount, order.base_coin_amount, Rc::strong_count(&self.0), Rc::weak_count(&self.0));
    }
}

impl PartialEq for OrderRef {
    fn eq(&self, other: &Self) -> bool {
        self.0.borrow().eq(&other.0.borrow())
    }
}

impl Ord for OrderRef {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.borrow().cmp(&other.0.borrow())
    }
}

impl Clone for OrderRef {
    fn clone(&self) -> OrderRef {
        Self(Rc::clone(&self.0))
    }
}

#[derive(ScryptoSbor)]
pub struct OrderBook {
    orders: KeyValueStore<u32, OrderRef>,
    active_orders: Vec<OrderRef>,
}

impl OrderBook {

    pub fn new(order_ref: OrderRef) -> OrderBook {
        // TODO: register type?
        let orders = KeyValueStore::new();
        let order_ref_clone = order_ref.clone();
        orders.insert(order_ref.get_id(), order_ref_clone);

        info!("strong: {}, weak: {}", Rc::strong_count(&order_ref.0), Rc::weak_count(&order_ref.0));

        Self {
            orders: orders,
            active_orders: vec![order_ref],
        }
    }

    pub fn insert(
        &mut self,
        order_ref: OrderRef,
    ) {
        self.orders.insert(order_ref.get_id(), OrderRef(Rc::clone(&order_ref.0)));

        match self.active_orders.binary_search(&order_ref) {
            Ok(_) => Runtime::panic("Should not happen".to_string()),
            Err(pos) => self.active_orders.insert(pos, order_ref),
        }
    }

    pub fn withdraw_coins(
        &mut self,
        order_id: &u32,
    ) -> Decimal {
        let order_ref = self.orders.get_mut(order_id).unwrap();
        let mut order = order_ref.0.borrow_mut();

        let coin_amount = order.bought_amount;

        order.bought_amount = Decimal::ZERO;

        coin_amount
    }

    pub fn withdraw_all(
        &mut self,
        order_id: &u32,
    ) -> (
        Decimal,
        Decimal,
    ) {
        let order_ref = self.orders.remove(order_id).unwrap();
        let order = order_ref.0.borrow();

        match self.active_orders.binary_search(&order_ref) {
            Ok(pos) => {
                self.active_orders.remove(pos);
            },
            Err(_) => {},
        }

        (order.base_coin_amount, order.bought_amount)
    }

    pub fn find_matches(
        &self,
        base_coin_in_pool: PreciseDecimal,
        coin_in_pool: PreciseDecimal,
    ) -> (
        Decimal, // Total base coin filled amount
        Vec<u32>, // Matched orders id
        Option<u32>, // Partially matched order id
    ) {
        let mut base_coin_amount_so_far = Decimal::ZERO;
        let mut filled_orders_id: Vec<u32> = vec![];

        for order_ref in self.active_orders.iter().rev() {
            let machable_base_coin_amount = (coin_in_pool *order_ref.get_price() - base_coin_in_pool)
                .checked_truncate(RoundingMode::ToZero)
                .unwrap();

            // No match for this order
            if machable_base_coin_amount <= base_coin_amount_so_far {
                return (base_coin_amount_so_far, filled_orders_id, None);
            }

            if machable_base_coin_amount - base_coin_amount_so_far >= order_ref.get_base_coin_amount() {

                // Order filled
                base_coin_amount_so_far += order_ref.get_base_coin_amount();
                filled_orders_id.push(order_ref.get_id());
info!("Filled: #{}#, base_coin_amount_so_far: {}", order_ref.get_id(), base_coin_amount_so_far);

            } else {

                // Order partially filled
                let partially_filled_order_amount = machable_base_coin_amount - base_coin_amount_so_far;
info!("Partially filled: #{}#, base_coin_amount_so_far: {}", order_ref.get_id(), base_coin_amount_so_far + partially_filled_order_amount);
                return (base_coin_amount_so_far + partially_filled_order_amount, filled_orders_id, Some(order_ref.get_id()));
            }
        }

        return (base_coin_amount_so_far, filled_orders_id, None);
    }

    pub fn swap(
        &mut self,
        base_coin_amount: Decimal,
        coin_amount: Decimal,
    ) {
        let mut remaining_base_coin_amount = base_coin_amount;
        let ratio = coin_amount / base_coin_amount;
        let mut orders_to_remove = 0;

        for (i, order_ref) in self.active_orders.iter_mut().rev().enumerate() {
info!("i: {}", i);
            if remaining_base_coin_amount == Decimal::ZERO {
                orders_to_remove = i;
                break;
            }

            if remaining_base_coin_amount >= order_ref.get_base_coin_amount() {

                // Order filled
                remaining_base_coin_amount -= order_ref.get_base_coin_amount();
                order_ref.swap_all(ratio);

            } else {

                // Order partially filled
                order_ref.swap(remaining_base_coin_amount, ratio);
                orders_to_remove = i;
                break;
            }
        }

        self.active_orders.truncate(self.active_orders.len() - orders_to_remove);
    }

}
