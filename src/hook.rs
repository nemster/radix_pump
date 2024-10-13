use scrypto::prelude::*;

#[derive(Debug, ScryptoSbor, PartialEq, Clone, Copy)]
pub enum HookableOperation {
    PostFairLaunch,
    PostTerminateFairLaunch,
    PostQuickLaunch,
    PostBuy,
    PostSell,
    PostReturnFlashLoan,
}

#[derive(Debug, ScryptoSbor, PartialEq, Clone)]
pub struct HookArgument {
    pub coin_address: ResourceAddress,
    pub operation: HookableOperation,
    pub amount: Option<Decimal>,
    // TODO: add mode. Other fields too?
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct HookCallEvent {
    coin_address: ResourceAddress,
    operation: HookableOperation,
    amount: Option<Decimal>,
}

#[blueprint]
#[events(HookCallEvent)]
mod hook {
    enable_method_auth! {
        roles {
            radix_pump => updatable_by: [OWNER];
        },
        methods {
            hook => restrict_to: [radix_pump];
        }
    }

    struct Hook {
    }

    impl Hook {
        pub fn new(
            owner_badge_address: ResourceAddress,
            caller_badge_address: ResourceAddress,
        ) -> Global<Hook> {
            Self {}
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(owner_badge_address))))
            .roles(roles!(
                radix_pump => rule!(require(caller_badge_address));
            ))
            .globalize()
        }

        pub fn hook(
            &self,
            argument: HookArgument,
        ) -> Option<Bucket> {
            Runtime::emit_event(
                HookCallEvent {
                    coin_address: argument.coin_address,
                    operation: argument.operation,
                    amount: argument.amount,
                }
            );

            None
        }
    }
}

