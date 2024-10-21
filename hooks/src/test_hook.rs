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
    enable_method_auth! {
        roles {
            radix_pump => updatable_by: [OWNER];
        },
        methods {
            hook => restrict_to: [radix_pump];
        }
    }

    struct TestHook {
        resource_manager: ResourceManager,
    }

    impl TestHook {
        pub fn new(
            owner_badge_address: ResourceAddress,
            caller_badge_address: ResourceAddress,
        ) -> Global<TestHook> {
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
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(owner_badge_address))))
            .roles(roles!(
                radix_pump => rule!(require(caller_badge_address));
            ))
            .globalize()
        }
    }

    impl HookInterfaceTrait for TestHook {
        fn hook(
            &self,
            argument: HookArgument,
        ) -> Option<Bucket> {
            Runtime::emit_event(
                TestHookEvent {
                    coin_address: argument.coin_address,
                    operation: argument.operation,
                    amount: argument.amount,
                    mode: argument.mode,
                    price: argument.price,
                }
            );

            Some(self.resource_manager.mint(1))
        }
    }
}

