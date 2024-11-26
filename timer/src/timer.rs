use scrypto::prelude::*;
use crate::common::*;

/* This blueprint allows users to schedule the executon of hooks.
   It uses the same protocol used by RadixPump to communicate with hooks: the HookInterfaceScryptoStub
   interface and proxy + hook badge for authentication
   An alarm clock badge is returned at instantiation time. This must be used by the invoking software.
*/

static XRD_RESOURCE_ADDRESS: &str = "resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3";

// Users have to deposit XRD to pay network fees for their tasks, this is the amount of XRD that
// are locked when executing a task.
// If the deposited amount goes below this limit the task is suspended
static LOCKED_FEE_AMOUNT: Decimal = dec![10];

// This event is emitted when a new scheduled task is created.
// This is mainly intended for setting up a script that invokes Timer
#[derive(ScryptoSbor, ScryptoEvent)]
struct NewTaskEvent {
    id: u64,
    minute: String,
    hour: String,
    day_of_month: String,
    month: String,
    day_of_week: String,
    random_delay: u32,
}

// This event is emitted when a user withdraws his deposited XRD and the task is stopped
#[derive(ScryptoSbor, ScryptoEvent)]
struct RemoveTaskEvent {
    id: u64,
}

// Info about a hook
#[derive(ScryptoSbor)]
struct RegisteredHook {
    component_address: HookInterfaceScryptoStub,
    execution_round: HookExecutionRound,
}

#[blueprint]
#[events(
    NewTaskEvent,
    RemoveTaskEvent,
    BuyEvent,
    SellEvent,
    BuyTicketEvent,
    AddLiquidityEvent,
    RemoveLiquidityEvent,
)]
#[types(
    u64,
    FungibleVault,
    String,
    RegisteredHook,
    ResourceAddress,
    RadixPumpPoolInterfaceScryptoStub,
    TimerBadge,
)]
mod timer {
    enable_method_auth! {
        roles {
            alarm_clock => updatable_by: [OWNER];
        },
        methods {
            register_hook => restrict_to: [OWNER];
            unregister_hook => restrict_to: [OWNER];
            get_alarm_clock_badge => restrict_to: [OWNER];
            update_owner_fee => restrict_to: [OWNER];
            get_owner_fee => restrict_to: [OWNER];

            alarm_clock => restrict_to: [alarm_clock];

            new_task => PUBLIC;
            change_schedule => PUBLIC;
            add_gas => PUBLIC;
            remove_task => PUBLIC;
        }
    }

    struct Timer {

        // Where the XRD for executing tasks is stored
        fee_vaults: KeyValueStore<u64, FungibleVault>,

        // Resource manager for the users badges
        timer_badge_resource_manager: ResourceManager,
        last_timer_badge_id: u64,

        // Resource manager for the badge that will be used by a offchain system to invoke Timer
        alarm_clock_badge_resource_manager: ResourceManager,

        // The component address of RadixPump; this is needed to query informations about coins and
        // pools
        radix_pump_component: Global<AnyComponent>,

        // Vaults containing the badges needed to invoke the hooks
        proxy_badge_vault: FungibleVault,
        hook_badge_vault: FungibleVault,

        // List of registered hooks
        registered_hooks: KeyValueStore<String, RegisteredHook>,

        // Component address of the known pools so far
        pools: KeyValueStore<ResourceAddress, RadixPumpPoolInterfaceScryptoStub>,

        // Maximum hourly frequency acceptable for scheduled tasks
        max_hourly_frequency: u8,

        // Component owner's XRDs
        owner_vault: Vault,

        // How many XRD go to the component owner at each successful execution
        owner_fee: Decimal,
    }

    impl Timer {

        // Instantiates a Timer component
        pub fn new(

            // Resource address of the owner badge
            owner_badge_address: ResourceAddress,

            // The component address of RadixPump
            radix_pump_component: Global<AnyComponent>,

            // Buckets containing the badges needed to invoke the hooks
            proxy_badge_bucket: Bucket,
            hook_badge_bucket: Bucket,

            // Maximum hourly frequency acceptable for scheduled tasks
            max_hourly_frequency: u8,

            // How many XRD go to the component owner at each successful execution
            owner_fee: Decimal,
        ) -> (
            Global<Timer>,
            FungibleBucket, // Alarm clock badge
        ) {
            assert!(
                max_hourly_frequency > 0 && max_hourly_frequency <= 60,
                "Max hourly frequency out of 1-60 range",
            );
            assert!(
                owner_fee >= Decimal::ZERO,
                "Owner fee can't be a negative number",
            );

            // Reserve a component address to set proper permissions on the TimerBadge
            let (address_reservation, component_address) = Runtime::allocate_component_address(Timer::blueprint_id());

            // Create a resource manager to mint TimerBadges for users
            let timer_badge_resource_manager = ResourceBuilder::new_integer_non_fungible_with_registered_type::<TimerBadge>(
                OwnerRole::Updatable(rule!(require(owner_badge_address)))
            )
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(owner_badge_address));
            ))
            .burn_roles(burn_roles!(
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(require(owner_badge_address));
            ))
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => "TimerBadge", updatable;
                }
            ))
            .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                non_fungible_data_updater => rule!(require(global_caller(component_address)));
                non_fungible_data_updater_updater => rule!(require(owner_badge_address));
            ))
            .create_with_no_initial_supply();

            // Create an alarm clock badge for the invoking software
            let alarm_clock_badge_bucket = ResourceBuilder::new_fungible(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .divisibility(0)
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "symbol" => "CLOCK", updatable;
                    "name" => "Alarm clock badge", updatable;
                }
            ))
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(owner_badge_address));
            ))
            .burn_roles(burn_roles!(
                burner => rule!(deny_all);
                burner_updater => rule!(require(owner_badge_address));
            ))
            .mint_initial_supply(dec![1]);

            let alarm_clock_badge_address = alarm_clock_badge_bucket.resource_address();

            // Find the ResourceAddress of XRD for this network
            let xrd_resource_address = ResourceAddress::try_from_bech32(
                &AddressBech32Decoder::new(&NetworkDefinition::simulator()),
                XRD_RESOURCE_ADDRESS,
            )
            .unwrap();

            // Instantiate the component
            let timer = Self {
                fee_vaults: KeyValueStore::new_with_registered_type(),
                timer_badge_resource_manager: timer_badge_resource_manager,
                last_timer_badge_id: 0,
                alarm_clock_badge_resource_manager: ResourceManager::from_address(alarm_clock_badge_address),
                radix_pump_component: radix_pump_component,
                proxy_badge_vault: FungibleVault::with_bucket(FungibleBucket(proxy_badge_bucket)),
                hook_badge_vault: FungibleVault::with_bucket(FungibleBucket(hook_badge_bucket)),
                registered_hooks: KeyValueStore::new_with_registered_type(),
                pools: KeyValueStore::new_with_registered_type(),
                max_hourly_frequency: max_hourly_frequency,
                owner_vault: Vault::new(xrd_resource_address),
                owner_fee: owner_fee,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                alarm_clock => rule!(require(alarm_clock_badge_address));
            ))
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "name" => "Timer", updatable;
                }
            })
            .globalize();

            (timer, alarm_clock_badge_bucket)
        }

        // Make a hook available for users
        pub fn register_hook(
            &mut self,

            // Name that will identify the hook component from now on
            name: String,

            // Address of the hook component
            component_address: HookInterfaceScryptoStub,
        ) {

            // Get information about the hook
            let (execution_round, _) = component_address.get_hook_info();

            // Add the hook info to the KVS
            self.registered_hooks.insert(
                name,
                RegisteredHook {
                    component_address: component_address,
                    execution_round: execution_round,
                }
            );
        }

        // Remove a previously added hook
        pub fn unregister_hook(
            &mut self,

            // Name of the hook
            name: String,
        ) {
            let hook = self.registered_hooks.remove(&name);
            assert!(
                hook.is_some(),
                "Hook not found",
            );
        }

        // Get a new alarm clock badge
        pub fn get_alarm_clock_badge(&self) -> Bucket {
            self.alarm_clock_badge_resource_manager.mint(1)
        }

        // Set the XRD amount owed to the component owner for each successful execution
        pub fn update_owner_fee(
            &mut self,
            owner_fee: Decimal,
        ) {
            assert!(
                owner_fee >= Decimal::ZERO,
                "Owner fee can't be a negative number",
            );

            self.owner_fee = owner_fee;
        }

        // Withdraw the component owner collected fees
        pub fn get_owner_fee(&mut self) -> Bucket {
            self.owner_vault.take_all()
        }

        // This method must be called when it's time to execute a scheduled task
        pub fn alarm_clock(
            &mut self,

            // The numeric id of the scheduled task to execute
            nft_id: u64,
        ) {

            // Get info about the task to execute
            let non_fungible_data = self.timer_badge_resource_manager.get_non_fungible_data::<TimerBadge>(
                &NonFungibleLocalId::Integer(nft_id.into())
            );

            // Find the vault to pay network fees and make sure thera are enough funds
            let mut fee_vault = self.fee_vaults.get_mut(&nft_id).unwrap();
            let fee_vault_amount = fee_vault.amount();
            if fee_vault_amount < LOCKED_FEE_AMOUNT + self.owner_fee {
                if non_fungible_data.status == TaskStatus::OK {

                    // If the status in the TimerBadge is OK but fees level is low, use the remaining XRD to
                    // update the NFT status and quit
                    // This should avoid the caller to pay fees
                    fee_vault.lock_fee(fee_vault_amount);
                    self.timer_badge_resource_manager.update_non_fungible_data(
                        &NonFungibleLocalId::Integer(nft_id.into()),
                        "status",
                        TaskStatus::LowGas,
                    );
                    return;

                } else {
                    Runtime::panic("Low gas".to_string());
                }
            }

            // Lock the fees so that the invoking software doesn't have to pay network fees
            fee_vault.lock_fee(LOCKED_FEE_AMOUNT);

            // Take the owner share of the fee
            self.owner_vault.put(fee_vault.take(self.owner_fee).into());
            drop(fee_vault);

            // Find the pool component address that must be priveded to the hook
            let pool_component = self.get_pool_address(non_fungible_data.coin_address);

            // Build the HookArgument
            let hook_argument = HookArgument {
                component: pool_component,
                coin_address: non_fungible_data.coin_address,
                operation: HookableOperation::Timer,
                amount: None,
                mode: PoolMode::Normal, // I hope it is so
                price: Decimal::ZERO, // I don't know
                ids: vec![nft_id],
            };

            // Find the hook
            let mut hook_info = self.registered_hooks.get_mut(&non_fungible_data.hook);
            match hook_info {
                Some(_) => {

                    // If the hook is found and the task was previuosly marked as HookUnregistered,
                    // mare it OK again
                    if non_fungible_data.status == TaskStatus::HookUnregistered {
                        self.timer_badge_resource_manager.update_non_fungible_data(
                            &NonFungibleLocalId::Integer(nft_id.into()),
                            "status",
                            TaskStatus::OK,
                        );
                    }
                },
                None => {

                    // If the hook is not found change the task status to HookUnregistered and quit
                    if non_fungible_data.status == TaskStatus::OK {
                        self.timer_badge_resource_manager.update_non_fungible_data(
                            &NonFungibleLocalId::Integer(nft_id.into()),
                            "status",
                            TaskStatus::HookUnregistered,
                        );
                        return;

                    } else {

                        // If the hook is not found change the task status was alreay HookUnregistered, just
                        // panic to save fees
                        Runtime::panic("Hook not registered".to_string());
                    }
                },
            }
            let hook_info = hook_info.as_mut().unwrap();

            // Provide the hook badge if needed for this kind of hook
            let hook_badge_bucket = match hook_info.execution_round < 2 {
                true => Some(self.hook_badge_vault.take(Decimal::ONE)),
                false => None,
            };

            // Use the proxy badge to call the hook
            let (
                return_badge_bucket,
                _bucket, // The hook must not return buckets to Timer
                events,
                _,
            ) = self.proxy_badge_vault.authorize_with_amount(
                1,
                || hook_info.component_address.hook(
                    hook_argument,
                    hook_badge_bucket,
                )
            );

            // Make sure the hook returned the hook badge if provided
            match return_badge_bucket {
                Some(bucket) => {
                    assert!(
                        bucket.resource_address() == self.hook_badge_vault.resource_address() &&
                        bucket.amount() == Decimal::ONE,
                        "Badge not returned by the hook",
                    );
                    self.hook_badge_vault.put(bucket);
                },
                None => {
                    assert!(
                        hook_info.execution_round == 2,
                        "Badge not returned by the hook",
                    );
                },
            }

            // Emit all events returned by the hook
            for event in events {
                match &event {
                    AnyPoolEvent::BuyEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::SellEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::BuyTicketEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::AddLiquidityEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::RemoveLiquidityEvent(event) => Runtime::emit_event(*event),
                    _ => Runtime::panic("The hook is not supposed to generate this event".to_string()),
                }
            }
        }

        // Return the component address of the pool for the specified coin
        fn get_pool_address(
            &mut self,
            coin_address: ResourceAddress,
        ) -> RadixPumpPoolInterfaceScryptoStub {

            // Search the pool in the KVS
            let pool = self.pools.get(&coin_address);
            match pool {
                Some(pool) => *pool,
                None => {

                    // If not found, ask the RadixPump component about it and add it to the KVS
                    let pool_info: PoolInfo = self.radix_pump_component.call("get_pool_info", &(coin_address, ));
                    self.pools.insert(coin_address, pool_info.component);

                    pool_info.component
                },
            }
        }

        // Verify that the format of minute is confirming to crontab(5) and respects the maximum
        // hourly frequency
        fn check_minute(
            &self,
            minute: &String,
        ) {

            if minute.as_str() == "*" {
                assert!(
                    self.max_hourly_frequency == 60,
                    "Frequency over the allowed limit",
                );
                return;
            }

            let mut frequency = 0u8;

            for m in minute.split(",") {

                // Try to match */number
                if m.len() > 2 {
                    let (star_slash, number) = m.split_at(2);
                    if star_slash == "*/" {
                        match number.to_string().parse::<u8>() {
                            Ok(number) => {
                                assert!(
                                    number >= 1 && number <= 60,
                                    "Invalid minute specification",
                                );
                                frequency += 60 / number;
                            },
                            Err(_) => Runtime::panic("Invalid minute specification".to_string()),
                        }
                        continue;
                    }
                }

                // Try to match start-end
                match m.split_once("-") {
                    Some((start, end)) => {
                        let start = start.to_string().parse::<u8>().unwrap();
                        let end = end.to_string().parse::<u8>().unwrap();
                        assert!(
                            start <= end && end <= 60,
                            "Invalid minute specification",
                        );
                        frequency += end - start + 1;
                        continue;
                    },
                    None => {},
                }

                // Try to match a number
                match m.parse::<u8>() {
                    Ok(number) => {
                        assert!(
                            number < 60,
                            "Invalid minute specification",
                        );
                        frequency += 1;
                    },
                    _ => Runtime::panic("Invalid minute specification".to_string()),
                }
            }

            assert!(
                frequency <= self.max_hourly_frequency,
                "Frequency over the allowed limit",
            );

        }

        // Verify that the format of a is confirming to crontab(5)
        fn check_time_parameter(
            parameter: &String,
            min: u8,
            max: u8,
        ) -> bool {
            if parameter.as_str() == "*" {
                return true;
            }

            for p in parameter.split(",") {

                // Try to match */number
                if p.len() > 2 {
                    let (star_slash, number) = p.split_at(2);
                    if star_slash == "*/" {
                        match number.to_string().parse::<u8>() {
                            Ok(number) => {
                                if number < min || number > max {
                                    return false;
                                }
                            },
                            Err(_) => return false,
                        }
                        continue;
                    }
                }

                // Try to match start-end
                match p.split_once("-") {
                    Some((start, end)) => {
                        let start = start.to_string().parse::<u8>().unwrap();
                        let end = end.to_string().parse::<u8>().unwrap();
                        if start < min || start > end || end > max {
                            return false;
                        }
                        continue;
                    },
                    None => {},
                }

                // Try to match a number
                match p.parse::<u8>() {
                    Ok(number) => {
                        if number < min || number > max {
                            return false;
                        }
                    },
                    _ => return false,
                }
            }

            return true;
        }

        // Users can call this method to shedule a new task, a user badge is returned
        pub fn new_task(
            &mut self,

            // Schedule conforming the crontab(5) syntax
            mut minute: String,
            mut hour: String,
            mut day_of_month: String,
            mut month: String,
            mut day_of_week: String,

            // Random delay in seconds to prevent front running
            random_delay: u32,

            // Registered nave of the hook to invoke
            hook: String,

            // The main coin the hook should have to deal to
            coin_address: ResourceAddress,

            // XRD to pay the network fees for all of the task executions
            xrd_bucket: Bucket,

        ) -> Bucket {

            // Normalize and check the schedule syntax
            minute.retain(|c| !c.is_whitespace());
            self.check_minute(&minute);
            hour.retain(|c| !c.is_whitespace());
            if !Timer::check_time_parameter(&hour, 0, 23) {
                Runtime::panic("Invalid hour specification".to_string());
            }
            day_of_month.retain(|c| !c.is_whitespace());
            if !Timer::check_time_parameter(&hour, 1, 31) {
                Runtime::panic("Invalid day of month specification".to_string());
            }
            month.retain(|c| !c.is_whitespace());
            if !Timer::check_time_parameter(&hour, 1, 12) {
                Runtime::panic("Invalid month specification".to_string());
            }
            day_of_week.retain(|c| !c.is_whitespace());
            if !Timer::check_time_parameter(&hour, 0, 7) {
                Runtime::panic("Invalid day of week specification".to_string());
            }
           
            assert!(
                random_delay < 2678400, // One month
                "Random delay too big",
            );

            // Make sure the hook is registered
            let _hook_component = self.registered_hooks.get(&hook).expect("Hook not found");
            drop(_hook_component);

            // Make sure we know about the pool for this coin
            let _pool_component = self.get_pool_address(coin_address);

            // Make sure we have fees to execute at least once the task
            assert!(
                xrd_bucket.resource_address() == self.owner_vault.resource_address(),
                "Wrong coins in bucket",
            );
            assert!(
                xrd_bucket.amount() >= LOCKED_FEE_AMOUNT,
                "Not enough XRD",
            );

            // The id of this user badge
            self.last_timer_badge_id += 1;

            // Create a Vault for the future fees
            self.fee_vaults.insert(
                self.last_timer_badge_id,
                FungibleVault::with_bucket(xrd_bucket.as_fungible()),
            );

            // Emit the NewTaskEvent event so that the invoking software knows when to invoke Timer
            Runtime::emit_event(
                NewTaskEvent {
                    id: self.last_timer_badge_id,
                    minute: minute.clone(),
                    hour: hour.clone(),
                    day_of_month: day_of_month.clone(),
                    month: month.clone(),
                    day_of_week: day_of_week.clone(),
                    random_delay: random_delay,
                }
            );

            // Mint a badge for the user
            self.timer_badge_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.last_timer_badge_id),
                TimerBadge {
                    minute: minute,
                    hour: hour,
                    day_of_month: day_of_month,
                    month: month,
                    day_of_week: day_of_week,
                    random_delay: random_delay,
                    status: TaskStatus::OK,
                    hook: hook,
                    coin_address: coin_address,
                }
            )
        }

        // Verify a timer badge proof and return the info that are in the NFT
        fn check_timer_badge_proof(
            &self,
            timer_badge_proof: Proof,
        ) -> (
            NonFungibleLocalId,
            TimerBadge,
        ) {
            // Check the proof or die trying
            let checked_proof = timer_badge_proof.check_with_message(
                self.timer_badge_resource_manager.address(),
                "Wrong badge",
            );

            let non_fungible = checked_proof.as_non_fungible().non_fungible::<TimerBadge>();

            (
                non_fungible.local_id().clone(),
                non_fungible.data(),
            )
        }

        // Update the schedule of an existing task
        pub fn change_schedule(
            &self,

            // Proof of the badge returned by the new_task method
            timer_badge_proof: Proof,

            // Schedule conforming the crontab(5) syntax
            mut minute: String,
            mut hour: String,
            mut day_of_month: String,
            mut month: String,
            mut day_of_week: String,

            // Random delay to prevent front running
            random_delay: u32,

        ) {
            // Verify che badge and get info about the task
            let (id, old_data) = self.check_timer_badge_proof(timer_badge_proof);

            // If the minute part of the schedule has to be changed, update the NFT
            minute.retain(|c| !c.is_whitespace());
            if minute != old_data.minute {
                self.check_minute(&minute);
                self.timer_badge_resource_manager.update_non_fungible_data(
                    &id,
                    "minute",
                    minute.clone(),
                );
            }

            // If the hour part of the schedule has to be changed, update the NFT
            hour.retain(|c| !c.is_whitespace());
            if hour != old_data.hour {
                if !Timer::check_time_parameter(&hour, 0, 23) {
                    Runtime::panic("Invalid hour specification".to_string());
                }
                self.timer_badge_resource_manager.update_non_fungible_data(
                    &id,
                    "hour",
                    hour.clone(),
                );
            }

            // If the day_of_month part of the schedule has to be changed, update the NFT
            day_of_month.retain(|c| !c.is_whitespace());
            if day_of_month !=  old_data.day_of_month {
                if !Timer::check_time_parameter(&hour, 1, 31) {
                    Runtime::panic("Invalid day of month specification".to_string());
                }
                self.timer_badge_resource_manager.update_non_fungible_data(
                    &id,
                    "day_of_month",
                    day_of_month.clone(),
                );
            }

            // If the month part of the schedule has to be changed, update the NFT
            month.retain(|c| !c.is_whitespace());
            if month != old_data.month {
                if !Timer::check_time_parameter(&hour, 1, 12) {
                    Runtime::panic("Invalid month specification".to_string());
                }
                self.timer_badge_resource_manager.update_non_fungible_data(
                    &id,
                    "month",
                    month.clone(),
                );
            }

            // If the day_of_week part of the schedule has to be changed, update the NFT
            day_of_week.retain(|c| !c.is_whitespace());
            if day_of_week != old_data.day_of_week {
                if !Timer::check_time_parameter(&hour, 0, 7) {
                    Runtime::panic("Invalid day of week specification".to_string());
                }
                self.timer_badge_resource_manager.update_non_fungible_data(
                    &id,
                    "day_of_week",
                    day_of_week.clone(),
                );
            }

            // If the random_delay has to be changed, update the NFT
            if random_delay != old_data.random_delay {
                assert!(
                    random_delay < 2678400, // One month
                    "Random delay too big",
                );
                self.timer_badge_resource_manager.update_non_fungible_data(
                    &id,
                    "random_delay",
                    random_delay,
                );
            }

            // NonFungibleLocalId -> u64 conversion
            let id = match &id {
                NonFungibleLocalId::Integer(id) => id.value(),
                _ => Runtime::panic("WTF".to_string()),
            };

            // Let the alarm clock know about the new schedule
            Runtime::emit_event(
                NewTaskEvent {
                    id: id,
                    minute: minute,
                    hour: hour,
                    day_of_month: day_of_month,
                    month: month,
                    day_of_week: day_of_week,
                    random_delay: random_delay,
                }
            );
        }

        // Users can call this method to deposit more XRD to pay for the network fees
        pub fn add_gas(
            &mut self,

            // Proof of the badge returned by the new_task method
            timer_badge_proof: Proof,

            // XRD to pay the fees
            xrd_bucket: Bucket,
        ) {

            // Verify the badge and get info about the task
            let (nft_id, non_fungible_data) = self.check_timer_badge_proof(timer_badge_proof);

            // NonFungibleLocalId -> u64 conversion
            let id = match nft_id {
                NonFungibleLocalId::Integer(ref id) => id.value(),
                _ => Runtime::panic("WTF".to_string()),
            };

            // Put the XRD in the fees vault
            let mut fee_vault = self.fee_vaults.get_mut(&id).unwrap();
            fee_vault.put(xrd_bucket.as_fungible());

            // If the task status was LowGas and now the fees are enough for at least one
            // execution, update the task status to OK
            if non_fungible_data.status == TaskStatus::LowGas && fee_vault.amount() >= LOCKED_FEE_AMOUNT {
                self.timer_badge_resource_manager.update_non_fungible_data(
                    &nft_id,
                    "status",
                    TaskStatus::OK,
                );
            }
        }

        // Get back all of the deposited XRD for a task, burn its badge and remove its schedule
        pub fn remove_task(
            &mut self,

            // The badge returned by the new_task method
            timer_badge_bucket: Bucket,

        ) -> Bucket {
            assert!(
                timer_badge_bucket.resource_address() == self.timer_badge_resource_manager.address(),
                "Wrong badge",
            );
            
            // Find the badge id and make sure it is only one
            let id = match timer_badge_bucket.as_non_fungible().non_fungible::<TimerBadge>().local_id() {
                NonFungibleLocalId::Integer(id) => id.value(),
                _ => Runtime::panic("WTF".to_string()),
            };

            // Let the alarm clock know that the schedule has to be removed
            Runtime::emit_event(
                RemoveTaskEvent {
                    id: id,
                }
            );

            // Return the remainings of the previously deposited XRD
            self.fee_vaults.get_mut(&id).unwrap().take_all().into()
        }
    }
}
