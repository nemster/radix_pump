use scrypto::prelude::*;
use scrypto_interface::*;
use crate::common::*;

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
    pub mode: PoolMode,
    pub price: Option<Decimal>,
}

define_interface! {
    Hook impl [
        ScryptoStub,
        ScryptoTestStub,
        #[cfg(feature = "manifest-builder-stubs")]
        ManifestBuilderStub
    ] {
        fn hook(
            &self,
            argument: HookArgument,
        ) -> Option<Bucket>;
    }
}
