use scrypto::prelude::*;
use crate::common::*;
use crate::radix_pump::radix_pump::RadixPumpKeyValueStore;

// Informations about a hook
#[derive(ScryptoSbor)]
pub struct HookInfo {
    pub component_address: HookInterfaceScryptoStub,
    pub round: HookExecutionRound, // 0, 1 or 2
    pub allow_recursion: bool,
}

// RadixPump identifies hooks by a name, this KVS can contain all of the registered hooks
pub type HookByName = KeyValueStore<String, HookInfo>;

// This type keeps a list of hooks names per each operation
pub type HooksPerOperationRound = KeyValueStore<HookableOperation, Vec<String>>;

// A RedixPump component keeps a list of globally enabled hooks and a list of hooks enabled per
// pool
// This same struct is used to manage any of the two
#[derive(ScryptoSbor)]
pub struct HooksPerOperation {

    // Hooks are called in 3 different rounds (0, 1 and 2) so we need a vector of 3 elements to
    // keep track of the hooks per operation and per round
    kvs: Vec<HooksPerOperationRound>
}

impl HooksPerOperation {

    // Initialize a HooksPerOperation struct
    pub fn new() -> HooksPerOperation {
        Self {
            kvs: vec![
                KeyValueStore::new_with_registered_type(),
                KeyValueStore::new_with_registered_type(),
                KeyValueStore::new_with_registered_type(),
            ],
        }
    }

    // Enable a hook for the given operations
    pub fn add_hook(
        &mut self,

        // Hook name to enable
        name: &String,

        // Operation to enable the hook for
        operations: &Vec<String>,

        // Round 0, 1 or 2
        execution_round: HookExecutionRound,
    ) {
        // For each specified operation
        for o in operations.iter() {
            let operation = string_to_operation(o);

            if self.kvs[execution_round].get(&operation).is_none() {

                // If the list of hooks for this operation does not exist yet create a list
                // containing only this hook name
                self.kvs[execution_round].insert(operation, vec![name.clone()]);

            } else {

                // If the list of hooks for this operation already exists
                let mut vec = self.kvs[execution_round].get_mut(&operation).unwrap();

                // Make sure the hook is not already in the list
                if !vec.iter().any(|x| *x == *name) {

                    // Then add it
                    vec.push(name.clone());
                }
            }
        }
    }

    // Disable a hook for the given operations
    pub fn remove_hook(
        &mut self,

        // Hook name to disable
        name: &String,

        // Operation to disable the hook for
        operations: &Vec<String>,

        // Round 0, 1 or 2
        execution_round: HookExecutionRound,
    ) {
        // For each specified operation
        for o in operations.iter() {
            let operation = string_to_operation(o);

            // Remove the hook name from the list
            self.kvs[execution_round].get_mut(&operation).expect("Operation not found").retain(|x| *x != *name);
        }
    }

    // Check if a hook is enabled for a given operation in a given round
    pub fn hook_exists(
        &self,

        // Hook name to find
        name: &String,

        // Operation to search the hook in
        operation: &String,

        // Execution round to search the hook in
        execution_round: HookExecutionRound,
    ) -> bool {
        let operation = string_to_operation(&operation);

        // Get the list of hooks for the operation
        let vec = self.kvs[execution_round].get(&operation);
        match vec {

            // If the list is not initialised, the hook is not there
            None => return false,

            // Search the hook name in the list
            Some(vec) => vec.iter().any(|x| *x == *name),
        }
    }

    // Get the list of enabled hooks for a given operation in a given round
    pub fn get_hooks(
        &self,

        // Operation
        operation: HookableOperation,

        // Round
        execution_round: HookExecutionRound,
    ) -> Vec<String> {
        match self.kvs[execution_round].get(&operation) {
            // If the list is not iniialised, return an empty array
            None => vec![],

            // Else return the list
            Some(vec) => vec.to_vec(),
        }
    }

    // Get the 3 lists of hooks for a given operation
    pub fn get_all_hooks(
        &self,
        operation: HookableOperation,
    ) -> Vec<Vec<String>> {

        // Initialise the vector of vectors to return
        let mut vec_vec = vec![];

        // For each execution round
        for execution_round in 0..3 {

            // Add to the vector
            vec_vec.push(
                match self.kvs[execution_round].get(&operation) {

                    // An empty vector
                    None => vec![],

                    // Or the list of hooks for the operation and the round
                    Some(vec) => vec.to_vec(),
                }
            );
        }

        vec_vec
    }

    // This method can merge a list of hooks belonging to this struct with another list taken from
    // another HooksPerOperation
    pub fn merge(
        &self,

        // Operation the hooks are attached to
        operation: HookableOperation,

        // List of hooks enabled in a different set
        vec: &Vec<String>,

        // Execution round the hooks are executed in
        execution_round: HookExecutionRound,

    ) -> Vec<String> {

        // Get the local list of hooks
        let mut merged_hooks = match self.kvs[execution_round].get(&operation) {
            None => vec![],
            Some(v) => v.to_vec(),
        };
        
        // For each hook name in the other list
        vec.iter().for_each(|x| {

            // Add it to the list if it's not already there
            for y in merged_hooks.iter() {
                if *x == *y {
                    return;
                }
            }
            merged_hooks.push(x.to_string());
        });

        // Return the merged list
        merged_hooks
    }
}

// String to operation conversion
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
        "PostRedeemLosingTicket" => HookableOperation::PostRedeemLosingTicket,
        "PostAddLiquidity" => HookableOperation::PostAddLiquidity,
        "PostRemoveLiquidity" => HookableOperation::PostRemoveLiquidity,
        _ => Runtime::panic("Operation not found".to_string()),
    }
}
