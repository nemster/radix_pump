use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

// Event emitted by TestHook2 and TestHook2 components on each invocation
#[derive(ScryptoSbor, ScryptoEvent)]
struct TestHookEvent {
    coin_address: ResourceAddress,
    operation: HookableOperation,
    amount: Option<Decimal>,
    mode: PoolMode,
    price: Decimal,
}

// TestHook0 components emit this event when the base_coin_vault has not enough base coins to
// operate
#[derive(ScryptoSbor, ScryptoEvent)]
struct EmptyVaultEvent {
    argument: HookArgument,
}

// TestHook0 is a round 0 hook that at each invocation buys some coins from the invoking pool
// spending one base coin.
// The buy operation can trigger more hooks
#[blueprint_with_traits]
#[events(EmptyVaultEvent)]
mod test_hook0 {

    enable_method_auth! {
        roles {
            proxy => updatable_by: [OWNER];
        },
        methods {
            refill_vault => PUBLIC;
            hook => restrict_to: [proxy];
            get_hook_info => PUBLIC;
        }
    }

    struct TestHook0 {

        // A voult of base coins to buy the coins
        base_coin_vault: Vault,
    }

    impl TestHook0 {

        // Instantiate the TestHook0 component
        pub fn new(

            // Owner badge for this component
            owner_badge_address: ResourceAddress,

            // The badge RadixPump uses to authenticate against this hook
            proxy_badge_address: ResourceAddress,

            // A bucket of base coins to buy coins
            base_coin_bucket: Bucket,

        ) -> Global<TestHook0> {
            Self {
                base_coin_vault: Vault::with_bucket(base_coin_bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
            ))
            .globalize()
        }

        // Add new base coins to the vault to contnue operation
        pub fn refill_vault(
            &mut self,
            base_coin_bucket: Bucket,
        ) {
            self.base_coin_vault.put(base_coin_bucket);
        }
    }

    impl HookInterfaceTrait for TestHook0 {

        // Hook invocation method by RadixPump
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

            // Take one base coin from the vault and use the badge provided by RadixPump to call
            // the buy method of the pool
            if self.base_coin_vault.amount() >= Decimal::ONE {
                let (coin_bucket, new_hook_argument, event) = hook_badge_bucket.as_ref().unwrap().authorize_with_amount(
                    1,
                    || argument.component.buy(self.base_coin_vault.take(Decimal::ONE))
                );

                (
                    hook_badge_bucket, // The hook_badge_bucket must always be returned!
                    Some(coin_bucket), // Give the bought coins to the user
                    vec![event], // Report the pool BuyEvent back to RadixPump
                    vec![new_hook_argument], // Report the new HookArgument the Pool prepared back
                                             // to RadixPump so that it can trigger more hooks
                )
            } else {

                // Avoid panic in hooks. If something goes wrong just emit an event
                Runtime::emit_event(
                    EmptyVaultEvent {
                        argument: argument,
                    }
                );

                (
                    hook_badge_bucket, // The hook_badge_bucket must always be returned!
                    None,
                    vec![],
                    vec![],
                )
            }
        }

        // Round 0, non accepting calls trigered by other hooks
        fn get_hook_info(&self) -> (HookExecutionRound, bool) {(0, false)}
    }
}

// This round 1 hook emits an event and mints a coin at each invocation
#[blueprint_with_traits]
#[events(TestHookEvent)]
mod test_hook1 {
    enable_method_auth! {
        roles {
            proxy => updatable_by: [OWNER];
        },
        methods {
            hook => restrict_to: [proxy];
            get_hook_info => PUBLIC;
        }
    }

    struct TestHook1 {
        resource_manager: ResourceManager,
    }

    impl TestHook1 {

        // TestHook1 component instantiation
        pub fn new(
            // Owner badge for this component
            owner_badge_address: ResourceAddress,

            // The badge RadixPump uses to authenticate against this hook
            proxy_badge_address: ResourceAddress,

        ) -> Global<TestHook1> {

            // Create a coin to distribute to all users
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
            .mint_roles(mint_roles!(
                minter => rule!(allow_all);
                minter_updater => rule!(deny_all);
            ))
            .create_with_no_initial_supply();

            Self {
                resource_manager: resource_manager,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
            ))
            .globalize()
        }
    }

    impl HookInterfaceTrait for TestHook1 {

        // Hook invocation by RadixPump
        fn hook(
            &mut self,
            argument: HookArgument,
            hook_badge_bucket: Option<FungibleBucket>,
        ) -> (
            Option<FungibleBucket>,
            Option<Bucket>,
            Vec<AnyPoolEvent>,
            Vec<HookArgument>,
        ) {

            // Emit an event with information from the HookArgument
            Runtime::emit_event(
                TestHookEvent {
                    coin_address: argument.coin_address,
                    operation: argument.operation,
                    amount: argument.amount,
                    mode: argument.mode,
                    price: argument.price,
                }
            );

            (
                hook_badge_bucket, // The hook_badge_bucket must always be returned!
                Some(self.resource_manager.mint(1)), // Mint a coin for the user
                vec![],
                vec![],
            )
        }

        // Round 1, not accepting calls triggered by other hooks
        fn get_hook_info(&self) -> (HookExecutionRound, bool) {(1, false)}
    }
}

// This round 0 hook emits an event and mints a coin at each invocation
#[blueprint_with_traits]
#[events(TestHookEvent)]
mod test_hook2 {
    enable_method_auth! {
        roles {
            proxy => updatable_by: [OWNER];
        },
        methods {
            hook => restrict_to: [proxy];
            get_hook_info => PUBLIC;
        }
    }

    struct TestHook2 {
        resource_manager: ResourceManager,
    }

    impl TestHook2 {

        // TestHook2 component instantiation
        pub fn new(
            owner_badge_address: ResourceAddress,
            proxy_badge_address: ResourceAddress,
        ) -> Global<TestHook2> {

            // Create a coin to distribute to the users
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
            .mint_roles(mint_roles!(
                minter => rule!(allow_all);
                minter_updater => rule!(deny_all);
            ))
            .create_with_no_initial_supply();

            Self {
                resource_manager: resource_manager,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
            ))
            .globalize()
        }
    }

    impl HookInterfaceTrait for TestHook2 {

        // Hook invocation by RadixPump
        fn hook(
            &mut self,
            argument: HookArgument,
            hook_badge_bucket: Option<FungibleBucket>,
        ) -> (
            Option<FungibleBucket>,
            Option<Bucket>,
            Vec<AnyPoolEvent>,
            Vec<HookArgument>,
        ) {

            // Emit an event with information from the HookArgument
            Runtime::emit_event(
                TestHookEvent {
                    coin_address: argument.coin_address,
                    operation: argument.operation,
                    amount: argument.amount,
                    mode: argument.mode,
                    price: argument.price,
                }
            );

            (
                hook_badge_bucket, // The hook_badge_bucket must always be returned!
                Some(self.resource_manager.mint(1)), // Mint a coin for the user
                vec![],
                vec![],
            )
        }

        // Round 2, accept calls triggered by other hooks
        fn get_hook_info(&self) -> (HookExecutionRound, bool) {(2, true)}
    }
}

