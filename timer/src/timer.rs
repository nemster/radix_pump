use scrypto::prelude::*;
use crate::common::*;

static XRD_RESOURCE_ADDRESS: &str = "resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3";

static LOCKED_FEE_AMOUNT: Decimal = dec![11];

#[derive(ScryptoSbor, ScryptoEvent)]
struct CoinEnableEvent {
    coin_address: ResourceAddress,
    enabled: bool,
}

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

#[derive(ScryptoSbor, ScryptoEvent)]
struct RemoveTaskEvent {
    id: u64,
}

#[derive(ScryptoSbor)]
struct RegisteredHook {
    component_address: HookInterfaceScryptoStub,
    execution_round: HookExecutionRound,
}

#[blueprint]
#[events(
    CoinEnableEvent,
    NewTaskEvent,
    RemoveTaskEvent,
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

            alarm_clock => restrict_to: [alarm_clock];

            enable_coin => PUBLIC;

            new_task => PUBLIC;
            change_schedule => PUBLIC;
            add_gas => PUBLIC;
            remove_task => PUBLIC;
        }
    }

    struct Timer {
        fee_vaults: KeyValueStore<u64, FungibleVault>,

        timer_badge_resource_manager: ResourceManager,
        last_timer_badge_id: u64,

        alarm_clock_badge_resource_manager: ResourceManager,

        radix_pump_component: Global<AnyComponent>,

        coin_creator_badge_address: ResourceAddress,

        proxy_badge_vault: FungibleVault,

        hook_badge_vault: FungibleVault,

        registered_hooks: KeyValueStore<String, RegisteredHook>,

        enabled_coins: KeyValueStore<ResourceAddress, RadixPumpPoolInterfaceScryptoStub>,

        max_hourly_frequency: u8,
    }

    impl Timer {
        pub fn new(
            // Resource address of the owner badge
            owner_badge_address: ResourceAddress,
            radix_pump_component: Global<AnyComponent>,
            coin_creator_badge_address: ResourceAddress,
            proxy_badge_bucket: Bucket,
            hook_badge_bucket: Bucket,
            max_hourly_frequency: u8,
        ) -> (
            Global<Timer>,
            FungibleBucket,
        ) {
            assert!(
                max_hourly_frequency > 0 && max_hourly_frequency <= 60,
                "Max hourly frequency out of 1-60 range",
            );

            // Reserve a component address to set proper permissions on the TimerBadge
            let (address_reservation, component_address) = Runtime::allocate_component_address(Timer::blueprint_id());

            // Create a resource manager to mint TimerBadges
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
                    "name" => "Clock badge", updatable;
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

            let timer = Self {
                fee_vaults: KeyValueStore::new_with_registered_type(),
                timer_badge_resource_manager: timer_badge_resource_manager,
                last_timer_badge_id: 0,
                alarm_clock_badge_resource_manager: ResourceManager::from_address(alarm_clock_badge_address),
                radix_pump_component: radix_pump_component,
                coin_creator_badge_address: coin_creator_badge_address,
                proxy_badge_vault: FungibleVault::with_bucket(FungibleBucket(proxy_badge_bucket)),
                hook_badge_vault: FungibleVault::with_bucket(FungibleBucket(hook_badge_bucket)),
                registered_hooks: KeyValueStore::new_with_registered_type(),
                enabled_coins: KeyValueStore::new_with_registered_type(),
                max_hourly_frequency: max_hourly_frequency,
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

        pub fn register_hook(
            &mut self,

            // Name that will identify the hook component from now on
            name: String,

            // Address of the hook component
            component_address: HookInterfaceScryptoStub,
        ) {
            let (execution_round, _) = component_address.get_hook_info();

            self.registered_hooks.insert(
                name,
                RegisteredHook {
                    component_address: component_address,
                    execution_round: execution_round,
                }
            );
        }

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

        pub fn get_alarm_clock_badge(&self) -> Bucket {
            self.alarm_clock_badge_resource_manager.mint(1)
        }

        pub fn alarm_clock(
            &mut self,
            nft_id: u64,
        ) {
            let non_fungible_data = self.timer_badge_resource_manager.get_non_fungible_data::<TimerBadge>(
                &NonFungibleLocalId::Integer(nft_id.into())
            );

            let mut fee_vault = self.fee_vaults.get_mut(&nft_id).unwrap();
            let fee_vault_amount = fee_vault.amount();
            if fee_vault_amount < LOCKED_FEE_AMOUNT {
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
            fee_vault.lock_fee(LOCKED_FEE_AMOUNT);

            let mut hook_info = self.registered_hooks.get_mut(&non_fungible_data.hook);
            match hook_info {
                Some(_) => {
                    if non_fungible_data.status == TaskStatus::HookUnregistered {
                        self.timer_badge_resource_manager.update_non_fungible_data(
                            &NonFungibleLocalId::Integer(nft_id.into()),
                            "status",
                            TaskStatus::OK,
                        );
                    }
                },
                None => {
                    if non_fungible_data.status == TaskStatus::OK {
                        self.timer_badge_resource_manager.update_non_fungible_data(
                            &NonFungibleLocalId::Integer(nft_id.into()),
                            "status",
                            TaskStatus::HookUnregistered,
                        );
                        return;
                    } else {
                        Runtime::panic("Hook not registered".to_string());
                    }
                },
            }
            let hook_info = hook_info.as_mut().unwrap();
           
            let pool_component = self.enabled_coins.get(&non_fungible_data.coin_address);
            match pool_component {
                Some(_) => {
                    if non_fungible_data.status == TaskStatus::CoinDisabled {
                        self.timer_badge_resource_manager.update_non_fungible_data(
                            &NonFungibleLocalId::Integer(nft_id.into()),
                            "status",
                            TaskStatus::OK,
                        );
                    }
                },
                None => {
                    if non_fungible_data.status == TaskStatus::OK {
                        self.timer_badge_resource_manager.update_non_fungible_data(
                            &NonFungibleLocalId::Integer(nft_id.into()),
                            "status",
                            TaskStatus::CoinDisabled,
                        );
                        return;
                    } else {
                        Runtime::panic("Timer not enabled for this coin".to_string());
                    }
                },
            }
            let pool_component = pool_component.unwrap();

            let hook_argument = HookArgument {
                component: *pool_component,
                coin_address: non_fungible_data.coin_address,
                operation: HookableOperation::Timer,
                amount: None,
                mode: PoolMode::Normal, // I hope it is so
                price: Decimal::ZERO, // I don't know
                ids: vec![nft_id],
            };

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

            for event in events {
                match &event {
                    AnyPoolEvent::FairLaunchStartEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::FairLaunchEndEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::QuickLaunchEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::RandomLaunchStartEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::RandomLaunchEndEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::BuyEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::SellEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::LiquidationEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::FlashLoanEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::BuyTicketEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::FeeUpdateEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::BurnEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::AddLiquidityEvent(event) => Runtime::emit_event(*event),
                    AnyPoolEvent::RemoveLiquidityEvent(event) => Runtime::emit_event(*event),
                }
            }
        }

        pub fn enable_coin(
            &mut self,
            coin_creator_proof: Proof,
            enable: bool,
        ) {
            let checked_proof = coin_creator_proof.check_with_message(
                self.coin_creator_badge_address,
                "Wrong badge",
            );

            let coin_address =
                checked_proof.as_non_fungible().non_fungible::<CreatorData>().data().coin_resource_address;

            // Make sure the pool exists and it is not in liquidation mode
            let pool_info: PoolInfo = self.radix_pump_component.call("get_pool_info", &(coin_address, ));
            assert!(
                pool_info.pool_mode != PoolMode::Liquidation,
                "Pool in liquidation mode",
            );

            if enable {
                self.enabled_coins.insert(coin_address, pool_info.component);
            } else {
                self.enabled_coins.remove(&coin_address);
            }

            Runtime::emit_event(
                CoinEnableEvent {
                    coin_address: coin_address,
                    enabled: enable,
                }
            );
        }

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

        pub fn new_task(
            &mut self,
            mut minute: String,
            mut hour: String,
            mut day_of_month: String,
            mut month: String,
            mut day_of_week: String,
            random_delay: u32,
            hook: String,
            coin_address: ResourceAddress,
            xrd_bucket: Bucket,
        ) -> Bucket {
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

            let _hook_component = self.registered_hooks.get(&hook).expect("Hook not found");

            let _pool_component = self.enabled_coins.get(&coin_address).expect("Timer not enabled for this coin");

            let xrd_resource_address = ResourceAddress::try_from_bech32(
                &AddressBech32Decoder::new(&NetworkDefinition::simulator()),
                XRD_RESOURCE_ADDRESS,
            )
            .unwrap();

            assert!(
                xrd_bucket.resource_address() == xrd_resource_address,
                "Wrong coins in bucket",
            );
            assert!(
                xrd_bucket.amount() >= LOCKED_FEE_AMOUNT,
                "Not enough XRD",
            );

            self.last_timer_badge_id += 1;

            self.fee_vaults.insert(
                self.last_timer_badge_id,
                FungibleVault::with_bucket(xrd_bucket.as_fungible()),
            );

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

        pub fn change_schedule(
            &self,
            timer_badge_proof: Proof,
            mut minute: String,
            mut hour: String,
            mut day_of_month: String,
            mut month: String,
            mut day_of_week: String,
            random_delay: u32,
        ) {
            let (id, old_data) = self.check_timer_badge_proof(timer_badge_proof);

            minute.retain(|c| !c.is_whitespace());
            if minute != old_data.minute {
                self.check_minute(&minute);
                self.timer_badge_resource_manager.update_non_fungible_data(
                    &id,
                    "minute",
                    minute.clone(),
                );
            }

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

            let id = match &id {
                NonFungibleLocalId::Integer(id) => id.value(),
                _ => Runtime::panic("WTF".to_string()),
            };

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

        pub fn add_gas(
            &mut self,
            timer_badge_proof: Proof,
            xrd_bucket: Bucket,
        ) {
            let (nft_id, non_fungible_data) = self.check_timer_badge_proof(timer_badge_proof);

            let id = match nft_id {
                NonFungibleLocalId::Integer(ref id) => id.value(),
                _ => Runtime::panic("WTF".to_string()),
            };

            assert!(
                xrd_bucket.amount() >= LOCKED_FEE_AMOUNT,
                "Not enough XRD",
            );

            if non_fungible_data.status == TaskStatus::LowGas {
                self.timer_badge_resource_manager.update_non_fungible_data(
                    &nft_id,
                    "status",
                    TaskStatus::OK,
                );
            }

            self.fee_vaults.get_mut(&id).unwrap().put(xrd_bucket.as_fungible());
        }

        pub fn remove_task(
            &mut self,
            timer_badge_bucket: Bucket,
        ) -> Bucket {
            assert!(
                timer_badge_bucket.resource_address() == self.timer_badge_resource_manager.address(),
                "Wrong badge",
            );
            
            let id = match timer_badge_bucket.as_non_fungible().non_fungible::<TimerBadge>().local_id() {
                NonFungibleLocalId::Integer(id) => id.value(),
                _ => Runtime::panic("WTF".to_string()),
            };

            Runtime::emit_event(
                RemoveTaskEvent {
                    id: id,
                }
            );

            self.fee_vaults.get_mut(&id).unwrap().take_all().into()
        }
    }
}
