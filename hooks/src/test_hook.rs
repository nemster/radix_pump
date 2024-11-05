use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

#[derive(ScryptoSbor, ScryptoEvent)]
struct TestHookEvent {
    coin_address: ResourceAddress,
    operation: HookableOperation,
    amount: Option<Decimal>,
    mode: PoolMode,
    price: Option<Decimal>,
}

#[blueprint_with_traits]
#[events(TestHookEvent)]
mod test_hook0 {
    struct TestHook0 {
        hook_badge_address: ResourceAddress,
        base_coin_vault: Vault,
    }

    impl TestHook0 {
        pub fn new(
            owner_badge_address: ResourceAddress,
            hook_badge_address: ResourceAddress,
            base_coin_bucket: Bucket,
        ) -> Global<TestHook0> {
            Self {
                hook_badge_address: hook_badge_address,
                base_coin_vault: Vault::with_bucket(base_coin_bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .globalize()
        }
    }

    impl HookInterfaceTrait for TestHook0 {
        fn hook(
            &mut self,
            mut argument: HookArgument,
            hook_badge_bucket: FungibleBucket,
        ) -> (
            FungibleBucket,
            Option<Bucket>,
            Vec<AnyPoolEvent>,
            Vec<HookArgument>,
        ) {

            // Make sure the proxy component is the caller
            assert!(
                hook_badge_bucket.resource_address() == self.hook_badge_address &&
                hook_badge_bucket.amount() == dec!(1),
                "Wrong badge",
            );

            if self.base_coin_vault.amount() >= Decimal::ONE {
                let (coin_bucket, new_hook_argument, event) = hook_badge_bucket.authorize_with_amount(
                    1,
                    || argument.component.buy(self.base_coin_vault.take(Decimal::ONE))
                );

                (
                    hook_badge_bucket, // The hook_badge_bucket must always be returned!
                    Some(coin_bucket),
                    vec![event],
                    vec![new_hook_argument],
                )
            } else {
                (
                    hook_badge_bucket, // The hook_badge_bucket must always be returned!
                    None,
                    vec![],
                    vec![],
                )
            }
        }

        fn get_hook_info(&self) -> (HookExecutionRound, bool) {(0, false)}
    }
}

#[blueprint_with_traits]
#[events(TestHookEvent)]
mod test_hook1 {
    struct TestHook1 {
        hook_badge_address: ResourceAddress,
        resource_manager: ResourceManager,
    }

    impl TestHook1 {
        pub fn new(
            owner_badge_address: ResourceAddress,
            hook_badge_address: ResourceAddress,
        ) -> Global<TestHook1> {
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
            .mint_roles(mint_roles!(
                minter => rule!(allow_all);
                minter_updater => rule!(deny_all);
            ))
            .create_with_no_initial_supply();

            Self {
                hook_badge_address: hook_badge_address,
                resource_manager: resource_manager,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .globalize()
        }
    }

    impl HookInterfaceTrait for TestHook1 {
        fn hook(
            &mut self,
            argument: HookArgument,
            hook_badge_bucket: FungibleBucket,
        ) -> (
            FungibleBucket,
            Option<Bucket>,
            Vec<AnyPoolEvent>,
            Vec<HookArgument>,
        ) {

            // Make sure the proxy component is the caller
            assert!(
                hook_badge_bucket.resource_address() == self.hook_badge_address &&
                hook_badge_bucket.amount() == dec!(1),
                "Wrong badge",
            );

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
                Some(self.resource_manager.mint(1)),
                vec![],
                vec![],
            )
        }

        fn get_hook_info(&self) -> (HookExecutionRound, bool) {(1, false)}
    }
}

#[blueprint_with_traits]
#[events(TestHookEvent)]
mod test_hook2 {
    struct TestHook2 {
        hook_badge_address: ResourceAddress,
        resource_manager: ResourceManager,
    }

    impl TestHook2 {
        pub fn new(
            owner_badge_address: ResourceAddress,
            hook_badge_address: ResourceAddress,
        ) -> Global<TestHook2> {
            let resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
            .mint_roles(mint_roles!(
                minter => rule!(allow_all);
                minter_updater => rule!(deny_all);
            ))
            .create_with_no_initial_supply();

            Self {
                hook_badge_address: hook_badge_address,
                resource_manager: resource_manager,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .globalize()
        }
    }

    impl HookInterfaceTrait for TestHook2 {
        fn hook(
            &mut self,
            argument: HookArgument,
            hook_badge_bucket: FungibleBucket,
        ) -> (
            FungibleBucket,
            Option<Bucket>,
            Vec<AnyPoolEvent>,
            Vec<HookArgument>,
        ) {

            // Make sure the proxy component is the caller
            assert!(
                hook_badge_bucket.resource_address() == self.hook_badge_address &&
                hook_badge_bucket.amount() == dec!(1),
                "Wrong badge",
            );

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
                Some(self.resource_manager.mint(1)),
                vec![],
                vec![],
            )
        }

        fn get_hook_info(&self) -> (HookExecutionRound, bool) {(2, true)}
    }
}

