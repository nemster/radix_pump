use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;
use crate::order::*;

// This blueprint implements a Limit buy order system as a hook for RadixPump.
// RadixPump must invoke this hook when a sell operation happens on a pool

// NonFungibleData for the limit buy order NFT
#[derive(ScryptoSbor, NonFungibleData)]
struct LimitBuyOrderData {
    date_created: Instant,
    base_coin_amount: Decimal,
    coin_to_buy: ResourceAddress,
    price: Decimal,
    #[mutable]
    unfilled_amount: Decimal,
    #[mutable]
    coin_amount_bought: Decimal,
}

// Emit this event when one or more orders are filled or partially filled
#[derive(ScryptoSbor, ScryptoEvent)]
struct MatchedOrderEvent {
    coin: ResourceAddress,
    filled_orders_id: Vec<u32>,
    partially_filled_orders_id: Option<u32>,
}

// Limits to avoid transaction fees can grow too much
static MAX_MATCHING_ORDERS: usize = 30;
static MAX_ACTIVE_ORDERS_PER_COIN: usize = 500;

#[blueprint_with_traits]
#[events(MatchedOrderEvent)]
#[types(
    ResourceAddress,
    u32,
    Vec<LimitBuyOrderRef>,
    FungibleVault,
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

        // The vault where all of the base coins for the active orders are kept
        base_coin_vault: FungibleVault,

        // The resource manager to mint BuyOrder NFTs
        orders_resource_manager: NonFungibleResourceManager,

        // The numeric id of the last created order
        last_order_id: u32,

        // In this simple implementation the order book is just a vector sorted by increasing price
        // and decreasing id
        active_orders: KeyValueStore<ResourceAddress, Vec<LimitBuyOrderRef>>,

        // The address of the RadixPump component, it is used to perform some checks when a new
        // order is created
        radix_pump_component: Global<AnyComponent>,

        // The vaults where the different bought coins are stored
        coins_vaults: KeyValueStore<ResourceAddress, FungibleVault>,
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
            .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                non_fungible_data_updater => rule!(require(global_caller(component_address)));
                non_fungible_data_updater_updater => rule!(require(owner_badge_address));
            ))
            .create_with_no_initial_supply();

            // Instantiate the component
            Self {
                base_coin_vault: FungibleVault::new(base_coin_address),
                orders_resource_manager: orders_resource_manager,
                last_order_id: 0,
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

        // Users can call this method to create a new order
        pub fn new_order(
            &mut self,

            // The bucket of base coins to buy coins with
            mut base_coin_bucket: Bucket,

            // Which coin must be bought
            coin_to_buy: ResourceAddress,

            // The desired price
            price: Decimal,

        ) -> Vec<Bucket> // This can contain just the LimitOrder NFT or the bought coins if the
                         // order can be immediately filled or both if case of a partial fill
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

            // Create the array of buckets to return
            let mut buckets: Vec<Bucket> = vec![];

            // This is the number of base coins that should be spend to make the coin reach the
            // desired price
            // It can be a negative number if the current price is higher than the desired one, or
            // a positive number in it's lower
            let base_coin_amount_to_sell = pool_info.coin_amount * price * 
                ((100 - pool_info.total_buy_fee_percentage) / 100) -
                pool_info.base_coin_amount;

            // If base_coin_amount_to_sell is bigger than zero we have a match
            if base_coin_amount_to_sell > Decimal::ZERO {

                // Fill or partial fill?
                if base_coin_amount_to_sell >= base_coin_bucket.amount() {

                    // The order can be filled by buying coins fron the RadixPump component (we
                    // have no hook badge now to talk directly to the pool)
                    let (coin_bucket, mut vec1, mut vec2): (Bucket, Vec<Bucket>, Vec<Bucket>) = 
                        self.radix_pump_component.call("swap", &(base_coin_bucket, coin_to_buy, 0u64));

                    // Put all of the buckets received by RadixPump into one vector
                    buckets.push(coin_bucket);
                    buckets.append(&mut vec1);
                    buckets.append(&mut vec2);

                    return buckets;
                } else {

                    // Order partially filled, only a part of base_coin_bucket is used
                    let (coin_bucket, mut vec1, mut vec2): (Bucket, Vec<Bucket>, Vec<Bucket>) = 
                        self.radix_pump_component.call(
                            "swap",
                            &(
                                base_coin_bucket.take_advanced(
                                    base_coin_amount_to_sell,
                                    WithdrawStrategy::Rounded(RoundingMode::ToZero)
                                ),
                                coin_to_buy,
                                0u64
                            )
                        );

                    // Put all of the buckets received by RadixPump into one vector
                    buckets.push(coin_bucket);
                    buckets.append(&mut vec1);
                    buckets.append(&mut vec2);
                }
            }

            // Create a LimitBuyOrderRef object too and add it to the active orders
            self.last_order_id += 1;
            let order_ref = LimitBuyOrderRef::new(
                self.last_order_id,
                price,
            );
            let mut active_orders = self.active_orders.get_mut(&coin_to_buy);
            match active_orders {
                None => {
                    drop(active_orders);

                    // Create an order book with just this order in it
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

                    // The order book is sorted by increasing price and decreasing id
                    // Find the right place to insert the new order
                    match active_orders.binary_search(&order_ref) {
                        Ok(_) => Runtime::panic("Should not happen".to_string()),
                        Err(pos) => active_orders.insert(pos, order_ref),
                    }
                }
            }

            // Mint an NFT for the user with visible informations about the order in it, then add
            // it to the vec of buckets for the user
            let order_nft = self.orders_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.last_order_id.into()),
                LimitBuyOrderData {
                    date_created: Clock::current_time_rounded_to_seconds(),
                    base_coin_amount: base_coin_bucket.amount(),
                    coin_to_buy: coin_to_buy,
                    price: price,
                    unfilled_amount: base_coin_bucket.amount(),
                    coin_amount_bought: Decimal::ZERO,
                }
            );
            buckets.push(order_nft.into());

            // Put the base coins in the shared vault
            self.base_coin_vault.put(FungibleBucket(base_coin_bucket));

            buckets
        }

        // Users can use this method to withdraw the bought coins so far (coins_only = true) or to cancel one or more
        // active orders (coins_only = false)
        // In case coins_only is true the order NFTs are returned back to the user, otherways they are burned
        pub fn withdraw(
            &mut self,
            order_bucket: Bucket, // Order NFTs
            coins_only: bool,
        ) -> Vec<Bucket> {
            assert!(
                order_bucket.resource_address() == self.orders_resource_manager.address(),
                "Unknown token",
            );

            // Create the array of buckets to return
            let mut buckets: Vec<Bucket> = vec![];

            // How many base coins to withdraw in case coins_only is false
            let mut base_coins_to_withdraw = Decimal::ZERO;

            // For each order NFT in the bucket
            for order_nft in order_bucket.as_non_fungible().non_fungibles::<LimitBuyOrderData>().iter() {

                // Get data and id of the NFT
                let order_data = order_nft.data();
                let id = u32::try_from(
                    match order_nft.local_id() {
                        NonFungibleLocalId::Integer(id) => id.value(),
                        _ => Runtime::panic("Should not happen".to_string()),
                    }
                )
                .unwrap();

                // Take the bought coins and put them in the vector of buckets
                if order_data.coin_amount_bought > Decimal::ZERO {
                    let mut vault = self.coins_vaults.get_mut(&order_data.coin_to_buy).unwrap();
                    let bucket = vault.take_advanced(
                        order_data.coin_amount_bought,
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    );
                    buckets.push(bucket.into());
                }

                if coins_only {

                    // Update the bought coin amount in the NFT only if the NFT will not be burned
                    self.orders_resource_manager.update_non_fungible_data(
                        &NonFungibleLocalId::Integer((id as u64).into()),
                        "coin_amount_bought",
                        Decimal::ZERO
                    );

                } else {

                    // If the order has to be closed, remove it from the active orders list and
                    // add the amount of unfilled base coins to the total to withdraw
                    if order_data.unfilled_amount > Decimal::ZERO {
                        let mut active_orders = self.active_orders.get_mut(&order_data.coin_to_buy).unwrap();
                        let order_ref = LimitBuyOrderRef::new(id, order_data.price); 
                        let pos = active_orders.binary_search(&order_ref);
                        match pos {
                            Ok(pos) => { active_orders.remove(pos); },
                            Err(_) => {},
                        }
                        base_coins_to_withdraw += order_data.unfilled_amount;
                    }
                }
            }

            if coins_only {

                // If the orders doesn't have to be closed, return them to the user
                buckets.push(order_bucket);

            } else {

                // If there are leftover base coins in the closing orders, take them
                if base_coins_to_withdraw > Decimal::ZERO {
                    buckets.push(
                        self.base_coin_vault.take_advanced(
                            base_coins_to_withdraw,
                            WithdrawStrategy::Rounded(RoundingMode::ToZero)
                        ).into()
                    );
                }

                // Burn all of the order NFTs
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

            // Proceed only for Sell and Timer operations and if the pool is in Normal mode
            if argument.operation != HookableOperation::Sell &&
                argument.operation != HookableOperation::Timer ||
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

            // Try to match the active orders starting from the end of the vector
            for (pos, order_ref) in active_orders.iter().rev().enumerate() {

                // If too many orders have been matched, just stop here
                if pos >= MAX_MATCHING_ORDERS {
                    first_non_filled_order_pos = active_orders.len() - pos - 1;
                    break;
                }

                // Compute the number of base coins I can spend to buy at the desired price or this
                // order (can be less than zero if the price is higher than the desired one)
                let base_coin_amount = pool_info.coin_amount * *order_ref.get_price() * ((100 - pool_info.total_buy_fee_percentage) / 100) - pool_info.base_coin_amount;

                // If the orders with higher priority can already buy more than this amount, no deal
                // for this order and the following
                if base_coin_amount <= base_coin_amount_so_far {
                    first_non_filled_order_pos = active_orders.len() - pos - 1;
                    break;
                }

                // Get the data of the current order
                let order_data = self.orders_resource_manager.get_non_fungible_data::<LimitBuyOrderData>(
                    &NonFungibleLocalId::Integer(((*order_ref.get_id()) as u64).into())
                );

                // Compare buyable amount to the order unfilled amount
                if base_coin_amount - base_coin_amount_so_far > order_data.unfilled_amount {

                    // Order filled
                    base_coin_amount_so_far += order_data.unfilled_amount;

                } else {

                    // Order partially filled, take note of the position and stop
                    partial_filled_order_amount = base_coin_amount - base_coin_amount_so_far;
                    base_coin_amount_so_far = base_coin_amount;
                    partial_filled_order_id = Some(*order_ref.get_id());
                    first_non_filled_order_pos = active_orders.len() - pos - 1;

                    break
                }
            }

            // If no matches happened just stop
            if base_coin_amount_so_far == Decimal::ZERO {
                if argument.operation == HookableOperation::Timer {

                    // If the hook was invoked by the timer, it's ok to panic so we don't waste
                    // fees when doing nothing
                    Runtime::panic("Nothing to do".to_string());

                } else {
                    return (hook_badge_bucket, None, vec![], vec![]);
                }
            }

            // Take the matched base coin amount out of the vault
            let base_coin_bucket = self.base_coin_vault.take_advanced(
                base_coin_amount_so_far,
                WithdrawStrategy::Rounded(RoundingMode::ToZero)
            );

            // Use the hook badge to buy coins at the pool
            let (coin_bucket, _, event) = hook_badge_bucket.as_ref().unwrap().authorize_with_amount(
                1,
                || argument.component.buy(base_coin_bucket)
            );

            let bought_price = base_coin_amount_so_far / coin_bucket.amount();

            // Remove filled orders from self.active_orders
            if first_non_filled_order_pos < active_orders.len() - 1 {
                for i in (first_non_filled_order_pos + 1)..active_orders.len() {
                    let order_ref = &active_orders[i];
                    let id = order_ref.get_id();
                    let order_data = self.orders_resource_manager.get_non_fungible_data::<LimitBuyOrderData>(
                        &NonFungibleLocalId::Integer(((*id) as u64).into())
                    );

                    // Update their bought amounts
                    self.orders_resource_manager.update_non_fungible_data(
                        &NonFungibleLocalId::Integer(((*id) as u64).into()),
                        "coin_amount_bought",
                        order_data.coin_amount_bought + order_data.unfilled_amount / bought_price
                    );

                    // Update their unfilled amount
                    self.orders_resource_manager.update_non_fungible_data(
                        &NonFungibleLocalId::Integer(((*id) as u64).into()),
                        "unfilled_amount",
                        Decimal::ZERO
                    );
                   
                    // Add this order to the list of the filled ones (it will go in the event)
                    filled_orders_id.push(*id);
                }

                // Remove all of the filled orders from the active list
                active_orders.truncate(first_non_filled_order_pos + 1);
            }

            // update the partially filled order too (if any)
            if partial_filled_order_amount > Decimal::ZERO {
                let id = active_orders[first_non_filled_order_pos].get_id();
                let order_data = self.orders_resource_manager.get_non_fungible_data::<LimitBuyOrderData>(
                    &NonFungibleLocalId::Integer((*id as u64).into())
                );

                // Update the bought amounts
                self.orders_resource_manager.update_non_fungible_data(
                    &NonFungibleLocalId::Integer(((*id) as u64).into()),
                    "coin_amount_bought",
                    order_data.coin_amount_bought + partial_filled_order_amount / bought_price
                );

                // And the unfilled one
                self.orders_resource_manager.update_non_fungible_data(
                    &NonFungibleLocalId::Integer((*id as u64).into()),
                    "unfilled_amount",
                    order_data.unfilled_amount - partial_filled_order_amount
                );
            }

            // Put the bought coins in self.coins_vaults
            let coin_vault = self.coins_vaults.get_mut(&argument.coin_address);
            if coin_vault.is_none() {
                drop(coin_vault);

                self.coins_vaults.insert(
                    argument.coin_address,
                    FungibleVault::with_bucket(coin_bucket)
                );
            } else {
                coin_vault.unwrap().put(coin_bucket);
            }

            // Emit an event to let the users know of their matched orders
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
