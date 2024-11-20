use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;
use std::cmp::min;

#[derive(ScryptoSbor)]
struct TaskInfo {
    coin1_vault: Vault,
    coin2_vault: Vault,
    coin1_per_buy_operation: Decimal,
    max_price: Decimal,
    min_interval_buy_operations: u32,
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
        timer_badge_address: ResourceAddress,
        tasks: KeyValueStore<u64, TaskInfo>,
        base_coin_address: ResourceAddress,
        radix_pump_component: Global<AnyComponent>,
        pools: KeyValueStore<ResourceAddress, RadixPumpPoolInterfaceScryptoStub>,
    }

    impl Dca {

        pub fn new(
            // Owner badge of this component
            owner_badge_address: ResourceAddress,

            // The badge RadixPump uses to authenticate against this hook
            proxy_badge_address: ResourceAddress,

            timer_badge_address: ResourceAddress,

            base_coin_address: ResourceAddress,

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

        fn check_timer_badge_proof(
            &self,
            timer_badge_proof: Proof,
        ) -> (
            u64,
            TimerBadge,
        ) {
            // Check the proof or die trying
            let checked_proof = timer_badge_proof.check_with_message(
                self.timer_badge_address,
                "Wrong badge",
            );

            let non_fungible = checked_proof.as_non_fungible().non_fungible::<TimerBadge>();

            let id = match non_fungible.local_id() {
                NonFungibleLocalId::Integer(id) => id.value(),
                _ => Runtime::panic("WTF".to_string()),
            };

            (
                id,
                non_fungible.data(),
            )
        }

        fn get_pool_address(
            &mut self,
            coin_address: ResourceAddress,
        ) -> RadixPumpPoolInterfaceScryptoStub {
            let pool = self.pools.get(&coin_address);

            match pool {
                Some(pool) => *pool,
                None => {
                    let pool_info: PoolInfo = self.radix_pump_component.call("get_pool_info", &(coin_address, ));

                    self.pools.insert(coin_address, pool_info.component);

                    pool_info.component
                },
            }
        }

        pub fn new_task(
            &mut self,
            timer_badge_proof: Proof,
            coin1_bucket: Bucket,
            coin1_per_buy_operation: Decimal,
            max_price: Decimal,
            min_interval_buy_operations: u32,
        ) {
            let (timer_badge_id, timer_badge_data) = self.check_timer_badge_proof(timer_badge_proof);

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

            if coin1_bucket.resource_address() != self.base_coin_address {
                self.get_pool_address(coin1_bucket.resource_address());
            }

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

        pub fn withdraw(
            &mut self,
            timer_badge_proof: Proof,
            bought_coins_only: bool,
        ) -> (
            Option<Bucket>,
            Bucket,
        ) {
            let (timer_badge_id, _) = self.check_timer_badge_proof(timer_badge_proof);
            
            let mut task = self.tasks.get_mut(&timer_badge_id).expect("Task not found");
            let coin2_bucket = task.coin2_vault.take_all();

            let coin1_bucket = match bought_coins_only {
                true => None,
                false => Some(task.coin1_vault.take_all()),
            };

            (coin1_bucket, coin2_bucket)
        }

        pub fn add_funds(
            &mut self,
            timer_badge_proof: Proof,
            coin1_bucket: Bucket,
        ) {
            let (timer_badge_id, _) = self.check_timer_badge_proof(timer_badge_proof);

            let mut task = self.tasks.get_mut(&timer_badge_id).expect("Task not found");

            task.coin1_vault.put(coin1_bucket);
        }

        pub fn update_task(
            &mut self,
            timer_badge_proof: Proof,
            coin1_per_buy_operation: Decimal,
            max_price: Decimal,
            min_interval_buy_operations: u32,
        ) {
            assert!(
                coin1_per_buy_operation > Decimal::ZERO,
                "Number of coins to sell per operation must be greater than zero",
            );

            assert!(
                max_price > Decimal::ZERO,
                "Price can't be a negative number",
            );

            let (timer_badge_id, _) = self.check_timer_badge_proof(timer_badge_proof);

            let mut task = self.tasks.get_mut(&timer_badge_id).expect("Task not found");

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
            if argument.operation != HookableOperation::Timer {
                return (hook_badge_bucket, None, vec![], vec![]);
            }

            let task_id = argument.ids[0];
            let mut task = self.tasks.get_mut(&task_id);
            let task = task.as_mut().expect("Task not found");

            let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
            assert!(
                now >= task.last_buy_operation + i64::from(task.min_interval_buy_operations),
                "Too soon",
            );

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

            let mut events: Vec<AnyPoolEvent> = vec![];
            let mut hook_arguments: Vec<HookArgument> = vec![];

            let base_coin_bucket = match coin1_bucket.resource_address() == self.base_coin_address {
                true => coin1_bucket,
                false => {
                    let mut component = self.pools.get_mut(&coin1_bucket.resource_address());

                    let (base_coin_bucket, new_argument, event) =
                        hook_badge_bucket.as_ref().unwrap().authorize_with_amount(
                            1,
                            || component.as_mut().unwrap().sell(coin1_bucket)
                        );

                    events.push(event);
                    hook_arguments.push(new_argument);

                    base_coin_bucket
                },
            };

            let coin2_bucket = match self.base_coin_address == task.coin2_vault.resource_address() {
                true => base_coin_bucket,
                false => {
                    let (coin2_bucket, new_argument, event) =
                        hook_badge_bucket.as_ref().unwrap().authorize_with_amount(
                            1,
                            || argument.component.buy(base_coin_bucket)
                        );

                    events.push(event);
                    hook_arguments.push(new_argument);

                    coin2_bucket
                },
            };

            let bought_price = coin1_amount / coin2_bucket.amount();
            assert!(
                bought_price <= task.max_price,
                "Price too high",
            );

            task.last_buy_operation = now;

            task.coin2_vault.put(coin2_bucket);

            (hook_badge_bucket, None, events, hook_arguments)
        }

        fn get_hook_info(&self) -> (HookExecutionRound, bool) {(0, false)}
    }
}
