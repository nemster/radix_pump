use scrypto::prelude::*;
use crate::common::*;
use crate::radix_pump::radix_pump::RadixPumpKeyValueStore;

#[derive(ScryptoSbor)]
pub struct HookInfo {
    pub component_address: HookInterfaceScryptoStub,
    pub round: HookExecutionRound,
    pub allow_recursion: bool,
}

pub type HookByName = KeyValueStore<String, HookInfo>;

pub type HooksPerOperationRound = KeyValueStore<HookableOperation, Vec<String>>;

#[derive(ScryptoSbor)]
pub struct HooksPerOperation {
    kvs: Vec<HooksPerOperationRound>
}

impl HooksPerOperation {
    pub fn new() -> HooksPerOperation {
        Self {
            kvs: vec![
                KeyValueStore::new_with_registered_type(),
                KeyValueStore::new_with_registered_type(),
                KeyValueStore::new_with_registered_type(),
            ],
        }
    }


    pub fn add_hook(
        &mut self,
        name: &String,
        operations: &Vec<String>,
        execution_round: HookExecutionRound,
    ) {
        for o in operations.iter() {
            let operation = string_to_operation(o);

            if self.kvs[execution_round].get(&operation).is_none() {
                self.kvs[execution_round].insert(operation, vec![name.clone()]);
            } else {
                let mut vec = self.kvs[execution_round].get_mut(&operation).unwrap();

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
        execution_round: HookExecutionRound,
    ) {
        for o in operations.iter() {
            let operation = string_to_operation(o);

            self.kvs[execution_round].get_mut(&operation).expect("Operation not found").retain(|x| *x != *name);
        }
    }

    pub fn hook_exists(
        &self,
        name: &String,
        operation: &String,
        execution_round: HookExecutionRound,
    ) -> bool {
        let operation = string_to_operation(&operation);

        let vec = self.kvs[execution_round].get(&operation);
        match vec {
            None => return false,
            Some(vec) => vec.iter().any(|x| *x == *name),
        }
    }

    pub fn get_hooks(
        &self,
        operation: HookableOperation,
        execution_round: HookExecutionRound,
    ) -> Vec<String> {
        match self.kvs[execution_round].get(&operation) {
            None => vec![],
            Some(vec) => vec.to_vec(),
        }
    }

    pub fn get_all_hooks(
        &self,
        operation: HookableOperation,
    ) -> Vec<Vec<String>> {
        let mut vec_vec = vec![];

        for execution_round in 0..3 {
            vec_vec.push(
                match self.kvs[execution_round].get(&operation) {
                    None => vec![],
                    Some(vec) => vec.to_vec(),
                }
            );
        }

        vec_vec
    }

    pub fn merge(
        &self,
        operation: HookableOperation,
        vec: &Vec<String>,
        execution_round: HookExecutionRound,
    ) -> Vec<String> {
        let mut merged_hooks = match self.kvs[execution_round].get(&operation) {
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
        "PostAddLiquidity" => HookableOperation::PostAddLiquidity,
        "PostRemoveLiquidity" => HookableOperation::PostRemoveLiquidity,
        _ => Runtime::panic("Operation not found".to_string()),
    }
}


