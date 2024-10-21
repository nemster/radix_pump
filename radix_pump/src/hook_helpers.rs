use scrypto::prelude::*;
use crate::common::*;

pub type HookByName = KeyValueStore<String, HookInterfaceScryptoStub>;

#[derive(ScryptoSbor)]
pub struct HooksPerOperation {
    kvs: KeyValueStore<HookableOperation, Vec<String>>,
}

impl HooksPerOperation {
    pub fn new() -> HooksPerOperation {
        Self {
            kvs: KeyValueStore::new()
        }
    }

    pub fn add_hook(
        &mut self,
        name: &String,
        operations: &Vec<String>,
    ) {
        for o in operations.iter() {
            let operation = string_to_operation(o);

            if self.kvs.get(&operation).is_none() {
                self.kvs.insert(operation, vec![name.clone()]);
            } else {
                let mut vec = self.kvs.get_mut(&operation).unwrap();

                if !vec.iter().any(|x| *x == *name) {
                    vec.push(name.clone());
                }
            }
        }
    }

    pub fn remove_hook(
        &mut self,
        name: &String,
        operations: &Vec<String>,
    ) {
        for o in operations.iter() {
            let operation = string_to_operation(o);

            self.kvs.get_mut(&operation).expect("Operation not found").retain(|x| *x != *name);
        }
    }

    pub fn hook_exists(
        &self,
        name: &String,
        operation: &String,
    ) -> bool {
        let operation = string_to_operation(&operation);

        let vec = self.kvs.get(&operation);
        match vec {
            None => return false,
            Some(vec) => vec.iter().any(|x| *x == *name),
        }
    }

    pub fn get_hooks(
        &self,
        operation: HookableOperation,
    ) -> Vec<String> {
        match self.kvs.get(&operation) {
            None => vec![],
            Some(vec) => vec.to_vec(),
        }
    }

    pub fn merge(
        &self,
        operation: HookableOperation,
        vec: &Vec<String>,
    ) -> Vec<String> {
        let mut merged_hooks = match self.kvs.get(&operation) {
            None => vec![],
            Some(v) => v.to_vec(),
        };
        
        vec.iter().for_each(|x| {
            for y in merged_hooks.iter() {
                if *x == *y {
                    return;
                }
            }
            merged_hooks.push(x.to_string());
        });

        merged_hooks
    }

}

pub fn string_to_operation(operation: &String) -> HookableOperation {
    match operation.as_str() {
        "PostFairLaunch" => HookableOperation::PostFairLaunch,
        "PostTerminateFairLaunch" => HookableOperation::PostTerminateFairLaunch,
        "PostQuickLaunch" => HookableOperation::PostQuickLaunch,
        "PostRandomLaunch" => HookableOperation::PostRandomLaunch,
        "PostTerminateRandomLaunch" => HookableOperation::PostTerminateRandomLaunch,
        "PostBuy" => HookableOperation::PostBuy,
        "PostSell" => HookableOperation::PostSell,
        "PostReturnFlashLoan" => HookableOperation::PostReturnFlashLoan,
        "PostBuyTicket" => HookableOperation::PostBuyTicket,
        "PostRedeemWinningTicket" => HookableOperation::PostRedeemWinningTicket,
        "PostRedeemLousingTicket" => HookableOperation::PostRedeemLousingTicket,
        _ => Runtime::panic("Operation not found".to_string()),
    }
}


