use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;
use std::cmp::min;

// This blueprint implements a RadixPump hook that can be used to DCA (dollar cost average) buy a
// coin; it is ment to be invoked by the Timer, not hooked to any RadixPump operation.
// This hook doesn't mint a badge to identify users, it uses the timer badge for this purpouse; so,
// before interacting with this hook, users must create a task in the Timer referring to the coin
// they want to buy.

// A buy task creted by a user
#[derive(ScryptoSbor)]
struct TaskInfo {

    // Coins provided by the user to buy coin2
    coin1_vault: Vault,

    // Bought coins
    coin2_vault: Vault,

    // How many coin1 use per buy operation
    coin1_per_buy_operation: Decimal,

    // Max coin1/coin2 price acceptable
    max_price: Decimal,

    // Mimimum interval among buy operations
    min_interval_buy_operations: u32,

    // When the last buy operation happened
    last_buy_operation: i64,
}

#[blueprint_with_traits]
#[types(
    u64,
    TaskInfo,
    ResourceAddress,
    RadixPumpPoolInterfaceScryptoStub,
)]
mod dca {

    enable_method_auth! {
        roles {
            proxy => updatable_by: [OWNER];
        },
        methods {
            new_task => PUBLIC;
            withdraw => PUBLIC;
            add_funds => PUBLIC;
            update_task => PUBLIC;

            hook => restrict_to: [proxy];
            get_hook_info => PUBLIC;
        }
    }

    struct Dca {

        // Resource address of the timer badges minted by the Timer component when a user creates a
        // new task
        timer_badge_address: ResourceAddress,

        // All of scheduled tasks
        tasks: KeyValueStore<u64, TaskInfo>,

        // The resource address of the base coin used by RadixPump
        base_coin_address: ResourceAddress,

        // The address of the RadixPump component
        radix_pump_component: Global<AnyComponent>,

        // All of the pools this hook interacted with so far
        pools: KeyValueStore<ResourceAddress, RadixPumpPoolInterfaceScryptoStub>,
    }

    impl Dca {

        // Dca component intantiation
        pub fn new(
            // Owner badge of this component
            owner_badge_address: ResourceAddress,

            // The badge RadixPump uses to authenticate against this hook
            proxy_badge_address: ResourceAddress,

            // The resource address of the timer badges minted by the Timer component
            timer_badge_address: ResourceAddress,

            // The resource address of the base coin used by RadixPump
            base_coin_address: ResourceAddress,

            // The address of the RadixPump component
            radix_pump_component: Global<AnyComponent>,
        ) -> Global<Dca> {
            Self {
                timer_badge_address: timer_badge_address,
                tasks: KeyValueStore::new_with_registered_type(),
                base_coin_address: base_coin_address,
                radix_pump_component: radix_pump_component,
                pools: KeyValueStore::new_with_registered_type(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
            ))
            .metadata(metadata! {
                init {
                    "name" => "DcaHook", updatable;
                }
            })
            .globalize()
        }

        // Private method to get informations about a timer badge
        fn check_timer_badge_proof(
            &self,
            timer_badge_proof: Proof,
        ) -> (
            u64, // nft id
            TimerBadge, // non funglible data
        ) {
            // Check the proof or die trying
            let checked_proof = timer_badge_proof.check_with_message(
                self.timer_badge_address,
                "Wrong badge",
            );

            // Make sure the proof refers to a single NFT and get it
            let non_fungible = checked_proof.as_non_fungible().non_fungible::<TimerBadge>();

            // NonFungibleLocalId -> u64 conversion
            let id = match non_fungible.local_id() {
                NonFungibleLocalId::Integer(id) => id.value(),
                _ => Runtime::panic("WTF".to_string()),
            };

            (
                id,
                non_fungible.data(),
            )
        }

        // Private method to find the address of the Pool component of a given coin
        fn get_pool_address(
            &mut self,

            // The coin whose Pool must be find
            coin_address: ResourceAddress,

        ) -> RadixPumpPoolInterfaceScryptoStub {

            // Search the pool in the KVS
            let pool = self.pools.get(&coin_address);

            match pool {

                // If found just return it
                Some(pool) => *pool,
                None => {

                    // If not found, ask RadixPump about it
                    let pool_info: PoolInfo = self.radix_pump_component.call("get_pool_info", &(coin_address, ));

                    // Add the new found component to the KVS
                    self.pools.insert(coin_address, pool_info.component);

                    pool_info.component
                },
            }
        }

        // Users can call this method to create a buy task
        pub fn new_task(
            &mut self,

            // Proof of the task created in the Timer
            timer_badge_proof: Proof,

            // The coins to buy the target coin
            coin1_bucket: Bucket,

            // The amount of coins to spend per buy operation
            coin1_per_buy_operation: Decimal,

            // The max price the user wants to buy
            max_price: Decimal,

            // Mimimun time interval between buy operations
            min_interval_buy_operations: u32,
        ) {
            // Get info about the timer badge
            let (timer_badge_id, timer_badge_data) = self.check_timer_badge_proof(timer_badge_proof);

            // Some trivial checks
            assert!(
                self.tasks.get(&timer_badge_id).is_none(),
                "Task already exists",
            );
            assert!(
                timer_badge_data.coin_address != coin1_bucket.resource_address(),
                "Can't buy using the coin itself",
            );
            assert!(
                coin1_per_buy_operation > Decimal::ZERO,
                "Number coins to sell per operation must be greater than zero",
            );
            assert!(
                coin1_bucket.amount() >= coin1_per_buy_operation,
                "Given coins must be at least enough for one buy operation",
            );
            assert!(
                max_price > Decimal::ZERO,
                "Price can't be a negative number",
            );

            // If we are not using the base coin to buy we must first find the pool to sell the
            // provided coins
            if coin1_bucket.resource_address() != self.base_coin_address {
                self.get_pool_address(coin1_bucket.resource_address());
            }

            // Add the task to the KVS
            self.tasks.insert(
                timer_badge_id,
                TaskInfo {
                    coin1_vault: Vault::with_bucket(coin1_bucket),
                    coin2_vault: Vault::new(timer_badge_data.coin_address),
                    coin1_per_buy_operation: coin1_per_buy_operation,
                    max_price: max_price,
                    min_interval_buy_operations: min_interval_buy_operations,
                    last_buy_operation: 0,
                },
            );
        }

        // Users can invoke this method to withdraw their bought coins of to stop the buy
        // task and withdraw the provided coins too
        pub fn withdraw(
            &mut self,

            // Proof of the timer badge used to create a buy task
            timer_badge_proof: Proof,

            // Whether to withdraw just bought coins or deposited coins too
            bought_coins_only: bool,
        ) -> (
            Option<Bucket>, // Deposited coins
            Bucket, // Bought coins
        ) {

            // Verify the timer badge
            let (timer_badge_id, _) = self.check_timer_badge_proof(timer_badge_proof);
            
            // Take all bought coins
            let mut task = self.tasks.get_mut(&timer_badge_id).expect("Task not found");
            let coin2_bucket = task.coin2_vault.take_all();

            // If bought_coins_only is false take the deposited coins too
            let coin1_bucket = match bought_coins_only {
                true => None,
                false => Some(task.coin1_vault.take_all()),
            };

            (coin1_bucket, coin2_bucket)
        }

        // Users can call this method to add funds to a previously created buy task
        pub fn add_funds(
            &mut self,

            // Proof of the timer badge used to create a buy task
            timer_badge_proof: Proof,

            // The coins to deposit
            coin1_bucket: Bucket,
        ) {
            // Verify the timer badge
            let (timer_badge_id, _) = self.check_timer_badge_proof(timer_badge_proof);

            // Find the buy task and deposit the coins in it
            let mut task = self.tasks.get_mut(&timer_badge_id).expect("Task not found");
            task.coin1_vault.put(coin1_bucket);
        }

        // A user can call this method to modify one or more parameters specified during the task
        // creation
        pub fn update_task(
            &mut self,

            // Proof of the timer badge used to create a buy task
            timer_badge_proof: Proof,

            // The new amount of coins to spend per buy operation
            coin1_per_buy_operation: Decimal,

            // The new max price the user wants to buy
            max_price: Decimal,

            // New mimimun time interval between buy operations
            min_interval_buy_operations: u32,
        ) {
            // Some obvious checks
            assert!(
                coin1_per_buy_operation > Decimal::ZERO,
                "Number of coins to sell per operation must be greater than zero",
            );
            assert!(
                max_price > Decimal::ZERO,
                "Price can't be a negative number",
            );

            // Check the timer badge
            let (timer_badge_id, _) = self.check_timer_badge_proof(timer_badge_proof);

            // Find the task
            let mut task = self.tasks.get_mut(&timer_badge_id).expect("Task not found");

            // Update whatever needs to be updated
            task.coin1_per_buy_operation = coin1_per_buy_operation;
            task.max_price = max_price;
            task.min_interval_buy_operations = min_interval_buy_operations;
        }
    }

    impl HookInterfaceTrait for Dca {

        fn hook(
            &mut self,
            mut argument: HookArgument,
            hook_badge_bucket: Option<FungibleBucket>,
        ) -> (
            Option<FungibleBucket>,
            Option<Bucket>,
            Vec<AnyPoolEvent>,
            Vec<HookArgument>,
        ) {
            // This hook is only supposed to be called by the Timer
            if argument.operation != HookableOperation::Timer {
                return (hook_badge_bucket, None, vec![], vec![]);
            }

            // The timer badge id is the same of the task id used here
            let task_id = argument.ids[0];
            let mut task = self.tasks.get_mut(&task_id);
            let task = task.as_mut().expect("Task not found");

            // Check that enough time has passed since the last buy operation
            let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
            assert!(
                now >= task.last_buy_operation + i64::from(task.min_interval_buy_operations),
                "Too soon",
            );

            // Take the deposited coins for a single buy operation
            let mut coin1_amount = min(
                task.coin1_per_buy_operation,
                task.coin1_vault.amount(),
            );
            assert!(
                coin1_amount > Decimal::ZERO,
                "No coins to sell left",
            );
            let coin1_bucket = task.coin1_vault.take_advanced(
                coin1_amount,
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            );
            coin1_amount = coin1_bucket.amount();

            // Initialise the vector of events and operations
            let mut events: Vec<AnyPoolEvent> = vec![];
            let mut hook_arguments: Vec<HookArgument> = vec![];

            // If the provided coin is not the base coin we must sell it first
            let base_coin_bucket = match coin1_bucket.resource_address() == self.base_coin_address {
                true => coin1_bucket,
                false => {
                    let mut component = self.pools.get_mut(&coin1_bucket.resource_address());

                    let (base_coin_bucket, new_argument, event) =
                        hook_badge_bucket.as_ref().unwrap().authorize_with_amount(
                            1,
                            || component.as_mut().unwrap().sell(FungibleBucket(coin1_bucket))
                        );

                    events.push(event);
                    hook_arguments.push(new_argument);

                    base_coin_bucket.into()
                },
            };

            // If the coin to buy is different from the base coin, buy it.
            // In the current Timer implementation is not possible to schedule a task for the base
            // coin so this is always true.
            let coin2_bucket = match self.base_coin_address == task.coin2_vault.resource_address() {
                true => base_coin_bucket,
                false => {
                    let (coin2_bucket, new_argument, event) =
                        hook_badge_bucket.as_ref().unwrap().authorize_with_amount(
                            1,
                            || argument.component.buy(FungibleBucket(base_coin_bucket))
                        );

                    events.push(event);
                    hook_arguments.push(new_argument);

                    coin2_bucket.into()
                },
            };

            // Check if the bought price is acceptable
            let bought_price = coin1_amount / coin2_bucket.amount();
            assert!(
                bought_price <= task.max_price,
                "Price too high",
            );

            // Update the last buy operation and put the bought coins in their vault
            task.last_buy_operation = now;
            task.coin2_vault.put(coin2_bucket);

            (hook_badge_bucket, None, events, hook_arguments)
        }

        // Execution round 0, can't be triggered by other hooks
        fn get_hook_info(&self) -> (HookExecutionRound, bool) {(0, false)}
    }
}
