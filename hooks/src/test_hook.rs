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
mod test_hook {
    struct TestHook {
        hook_badge_address: ResourceAddress,
        resource_manager: ResourceManager,
    }

    impl TestHook {
        pub fn new(
            owner_badge_address: ResourceAddress,
            hook_badge_address: ResourceAddress,
        ) -> Global<TestHook> {
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
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(owner_badge_address))))
            .globalize()
        }
    }

    impl HookInterfaceTrait for TestHook {
        fn hook(
            &self,
            argument: HookArgument,
            hook_badge_bucket: FungibleBucket,
        ) -> (
            FungibleBucket,
            Option<Bucket>
        ) {

            // Make sure the proxy component is the caller
            assert!(
                hook_badge_bucket.resource_address() == self.hook_badge_address &&
                hook_badge_bucket.amount() == dec!(1),
                "Wrong badge",
            );

            // You can then use hook_badge_bucket to authenticate towards a Pool component

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
            )
        }
    }
}

