use scrypto::prelude::*;

#[derive(Debug, ScryptoSbor, PartialEq, Clone, Copy)]
pub enum PoolMode {
    WaitingForLaunch,
    Launching,
    Normal,
    Liquidation,
}

