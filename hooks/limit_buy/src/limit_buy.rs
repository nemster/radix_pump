use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;
use crate::order::*;

// NonFungibleData for the limit buy order NFT
#[derive(ScryptoSbor, NonFungibleData)]
struct LimitBuyOrderData {
    date_created: Instant,
    base_coin_amount: Decimal,
    coin_to_buy: ResourceAddress,
    price: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct MatchedOrderEvent {
    coin: ResourceAddress,
    filled_orders_id: Vec<u32>,
    partially_filled_orders_id: Option<u32>,
}

static MAX_MATCHING_ORDERS: usize = 50;
static MAX_ACTIVE_ORDERS_PER_COIN: usize = 500;

#[blueprint_with_traits]
#[events(MatchedOrderEvent)]
#[types(
    ResourceAddress,
    u32,
    LimitBuyOrder,
    Vec<LimitBuyOrderRef>,
    Vault,
    LimitBuyOrderData,
)]
mod limit_buy_hook {

    enable_method_auth! {
        roles {
            proxy => updatable_by: [OWNER];
        },
        methods {
            new_order => PUBLIC;
            withdraw => PUBLIC;
            hook => restrict_to: [proxy];
            get_hook_info => PUBLIC;
        }
    }

    struct LimitBuyHook {
        base_coin_vault: Vault,
        orders_resource_manager: ResourceManager,
        last_order_id: u32,
        orders: KeyValueStore<u32, LimitBuyOrder>,
        active_orders: KeyValueStore<ResourceAddress, Vec<LimitBuyOrderRef>>,
        radix_pump_component: Global<AnyComponent>,
        coins_vaults: KeyValueStore<ResourceAddress, Vault>,
    }

    impl LimitBuyHook {

        // This is the constructor for a LimitBuyHook component
        pub fn new(

            // Owner badge of this component
            owner_badge_address: ResourceAddress,

            // The badge RadixPump uses to authenticate against this hook
            proxy_badge_address: ResourceAddress,

            // The coin buyers have to deposit
            base_coin_address: ResourceAddress,

            radix_pump_component: ComponentAddress,

        ) -> Global<LimitBuyHook> {

            // Reserve a component address to set proper permissions on the LimitBuyOrder NFT
            let (address_reservation, component_address) = Runtime::allocate_component_address(LimitBuyHook::blueprint_id());

            // Create a resource manager to mint LimitBuyOrder NFTs
            let orders_resource_manager = ResourceBuilder::new_integer_non_fungible_with_registered_type::<LimitBuyOrderData>(
                OwnerRole::Updatable(rule!(require(owner_badge_address)))
            )
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(owner_badge_address));
            ))
            .burn_roles(burn_roles!(
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(require(owner_badge_address));
            ))
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => "LimitBuyOrder", updatable;
                }
            ))
            .create_with_no_initial_supply();

            // Instantiate the component
            Self {
                base_coin_vault: Vault::new(base_coin_address),
                orders_resource_manager: orders_resource_manager,
                last_order_id: 0,
                orders: KeyValueStore::new_with_registered_type(),
                active_orders: KeyValueStore::new_with_registered_type(),
                radix_pump_component: Global::from(radix_pump_component),
                coins_vaults: KeyValueStore::new_with_registered_type(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
            ))
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "name" => "LimitBuyHook", updatable;
                }
            })
            .globalize()
        }

        pub fn new_order(
            &mut self,
            base_coin_bucket: Bucket,
            coin_to_buy: ResourceAddress,
            price: Decimal,

        ) -> Vec<Bucket>
        {
            assert!(
                base_coin_bucket.resource_address() == self.base_coin_vault.resource_address(),
                "Wrong base coin",
            );

            // Make sure the pool exists and is not in liquidation mode
            let pool_info: PoolInfo = self.radix_pump_component.call("get_pool_info", &(coin_to_buy, ));
            assert!(
                pool_info.pool_mode != PoolMode::Liquidation,
                "Pool in liquidation mode",
            );

            let base_coin_amount = pool_info.coin_amount * price * ((100 - pool_info.total_buy_fee_percentage) / 100) - pool_info.base_coin_amount;
            if base_coin_amount > base_coin_bucket.amount() {
                let (coin_bucket, mut vec1, mut vec2): (Bucket, Vec<Bucket>, Vec<Bucket>) =
                    self.radix_pump_component.call("swap", &(base_coin_bucket, coin_to_buy, 0u64));
                vec1.push(coin_bucket);
                vec1.append(&mut vec2);
                return vec1;
            }

            self.last_order_id += 1;
            let order = LimitBuyOrder::new(base_coin_bucket.amount());
            self.orders.insert(
                self.last_order_id,
                order,
            );

            let order_ref = LimitBuyOrderRef::new(
                self.last_order_id,
                price,
            );
            let mut active_orders = self.active_orders.get_mut(&coin_to_buy);
            match active_orders {
                None => {
                    drop(active_orders);

                    self.active_orders.insert(
                        coin_to_buy,
                        vec![order_ref]
                    );
                },
                Some(ref mut active_orders) => {
                    assert!(
                        active_orders.len() < MAX_ACTIVE_ORDERS_PER_COIN,
                        "This orderbook is full",
                    );

                    match active_orders.binary_search(&order_ref) {
                        Ok(_) => Runtime::panic("Should not happen".to_string()),
                        Err(pos) => active_orders.insert(pos, order_ref),
                    }
                }
            }

            let order_nft = self.orders_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.last_order_id.into()),
                LimitBuyOrderData {
                    date_created: Clock::current_time_rounded_to_seconds(),
                    base_coin_amount: base_coin_bucket.amount(),
                    coin_to_buy: coin_to_buy,
                    price: price,
                }
            );

            self.base_coin_vault.put(base_coin_bucket);

            vec![order_nft]
        }

        pub fn withdraw(
            &mut self,
            order_bucket: Bucket,
            coins_only: bool,
        ) -> Vec<Bucket> {
            assert!(
                order_bucket.resource_address() == self.orders_resource_manager.address(),
                "Unknown token",
            );

            let mut buckets: Vec<Bucket> = vec![];
            let mut base_coins_to_withdraw = Decimal::ZERO;

            // For each order NFT in the bucket
            for order_nft in order_bucket.as_non_fungible().non_fungibles::<LimitBuyOrderData>().iter() {

                let order_data = order_nft.data();

                let id = u32::try_from(
                    match order_nft.local_id() {
                        NonFungibleLocalId::Integer(id) => id.value(),
                        _ => Runtime::panic("Should not happen".to_string()),
                    }
                )
                .unwrap();
                let mut order = self.orders.get_mut(&id).unwrap();

                let mut vault = self.coins_vaults.get_mut(&order_data.coin_to_buy).unwrap();
                let bucket = vault.take(*order.get_bought_amount());
                buckets.push(bucket);

                if coins_only {
                    order.coins_withdrawn();
                } else {
                    let base_coins_in_this_order = order.get_base_coin_amount();

                    if  *base_coins_in_this_order > Decimal::ZERO {
                        let mut active_orders = self.active_orders.get_mut(&order_data.coin_to_buy).unwrap();
                        let order_ref = LimitBuyOrderRef::new(id, order_data.price); 
                        let pos = active_orders.binary_search(&order_ref);
                        match pos {
                            Ok(pos) => { active_orders.remove(pos); },
                            Err(_) => {},
                        }

                        base_coins_to_withdraw += *base_coins_in_this_order;
                    }
                }
            }

            if coins_only {
                buckets.push(order_bucket);
            } else {
                if base_coins_to_withdraw > Decimal::ZERO {
                    buckets.push(self.base_coin_vault.take(base_coins_to_withdraw));
                }

                order_bucket.burn();
            }

            buckets
        }
    }

    impl HookInterfaceTrait for LimitBuyHook {

        // Hook invocation method by RadixPump
        fn hook(
            &mut self,
            mut argument: HookArgument,
            hook_badge_bucket: Option<FungibleBucket>,
        ) -> (
            Option<FungibleBucket>,
            Option<Bucket>, // This is always None
            Vec<AnyPoolEvent>,
            Vec<HookArgument>,
        ) {

            // Proceed only for PostSell and PostAddLiquidity operations and if the pool is in Normal mode
            if argument.operation != HookableOperation::PostSell &&
                argument.operation != HookableOperation::PostAddLiquidity ||
                argument.mode != PoolMode::Normal {
                return (hook_badge_bucket, None, vec![], vec![]);
            }

            let mut active_orders = self.active_orders.get_mut(&argument.coin_address);
            if active_orders.is_none() {
                return (hook_badge_bucket, None, vec![], vec![]);
            }
            let active_orders = active_orders.as_mut().unwrap();

            let pool_info = argument.component.get_pool_info();
            
            let mut base_coin_amount_so_far = Decimal::ZERO;

            let mut first_non_filled_order_pos: usize = 0;

            let mut partial_filled_order_amount = Decimal::ZERO;

            let mut partial_filled_order_id: Option<u32> = None;

            let mut filled_orders_id: Vec<u32> = vec![];

            for (pos, order_ref) in active_orders.iter().enumerate() {

                if pos >= MAX_MATCHING_ORDERS {
                    break;
                }

                // Compute the number of base coins I can spend to buy at the desired price or this
                // order (can be less than zero if the price is higher than the desired one)
                let base_coin_amount = pool_info.coin_amount * *order_ref.get_price() * ((100 - pool_info.total_buy_fee_percentage) / 100) - pool_info.base_coin_amount;

                // If the orders with higher priority can already buy more than this amount, no deal
                // for this order and the following
                if base_coin_amount <= base_coin_amount_so_far {
                    break;
                }

                let order = self.orders.get(order_ref.get_id()).unwrap();

                if base_coin_amount - base_coin_amount_so_far > *order.get_base_coin_amount() {

                    // Order filled
                    first_non_filled_order_pos = pos + 1;
                    base_coin_amount_so_far += *order.get_base_coin_amount();
                } else {

                    // Order partially filled
                    partial_filled_order_amount = base_coin_amount - base_coin_amount_so_far;
                    base_coin_amount_so_far = base_coin_amount;
                    partial_filled_order_id = Some(*order_ref.get_id());

                    break
                }
            }

            if base_coin_amount_so_far == Decimal::ZERO {
                return (hook_badge_bucket, None, vec![], vec![]);
            }

            let base_coin_bucket = self.base_coin_vault.take_advanced(
                base_coin_amount_so_far,
                WithdrawStrategy::Rounded(RoundingMode::ToZero)
            );

            let (coin_bucket, _, event) = hook_badge_bucket.as_ref().unwrap().authorize_with_amount(
                1,
                || argument.component.buy(base_coin_bucket)
            );

            let bought_price = base_coin_amount_so_far / coin_bucket.amount();

            // Remove filled orders from self.active_orders
            if first_non_filled_order_pos > 0 {
                for order_ref in active_orders.drain(0..first_non_filled_order_pos as usize) {
                    let id = order_ref.get_id();
                    self.orders.get_mut(id).unwrap().fill(bought_price);
                    filled_orders_id.push(*id);
                }
            }

            // If an order has been partially filled now it is the first one, update its bought_amount
            if partial_filled_order_amount > Decimal::ZERO {
                let id = active_orders[0].get_id();
                self.orders.get_mut(id).unwrap().partially_fill(
                    partial_filled_order_amount,
                    bought_price,
                );
            }

            // Put the bought coins in self.coins_vaults
            let coin_vault = self.coins_vaults.get_mut(&argument.coin_address);
            if coin_vault.is_none() {
                drop(coin_vault);

                self.coins_vaults.insert(
                    argument.coin_address,
                    Vault::with_bucket(coin_bucket)
                );
            } else {
                coin_vault.unwrap().put(coin_bucket);
            }

            Runtime::emit_event(
                MatchedOrderEvent {
                    coin: argument.coin_address,
                    filled_orders_id: filled_orders_id,
                    partially_filled_orders_id: partial_filled_order_id,
                }
            );

            (hook_badge_bucket, None, vec![event], vec![])
        }

        // Round 1, accepting calls triggered by other hooks
        fn get_hook_info(&self) -> (HookExecutionRound, bool) {(1, true)}
    }
}
