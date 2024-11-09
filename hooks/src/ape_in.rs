use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;
use std::cmp::*;

// This blueprint implements a RadixPump hook that can let users automatically buy every quick launched coin.
// Each user must deposit the same amount of base coins, at the component intantiation time it is
// decided how many launches users will take part and how much they pay for each launch
// It has to be enabled globally to the PostQuickLaunch operation, it does nothing if hooked to any
// other operation.

// Avoid hitting transaction limits
static MAX_COINS_WITHDRAW: u16 = 80;

// NonFungibleData for the buyer badge
#[derive(ScryptoSbor, NonFungibleData)]
struct ApeInBuyer {
    date_joined: Instant,
    last_launch_id: u64,
    lunches: u16,
    #[mutable]
    withdrawn_coins: u16,
}

// Informations about one of the quick launch bought
#[derive(ScryptoSbor)]
struct CoinLaunch {
    last_buyer_id: u64,
    coins_per_buyer: Decimal,
    vault: Vault,
}

#[blueprint_with_traits]
#[types(u64, CoinLaunch, ApeInBuyer)]
mod ape_in_hook {
    struct ApeInHook {

        // The badge RadixPump uses to authenticate against this hook and that can be used to
        // authenticate towards a Pool
        hook_badge_address: ResourceAddress,

        // How many quick launch a user will automatically take part
        launches_per_buyer: u16,

        // How many coins, per each user, will be used to buy a quick launch
        base_coins_per_launch: Decimal,

        // Numeric id of the last buyer badge minted
        last_buyer_id: u64,

        // Numeric id of the last launch happened
        last_launch_id: u64,

        // This KeyValueStore contains informations about all of the launches that have been bought
        launches: KeyValueStore<u64, CoinLaunch>,

        // Resource manager for minting buyer badges
        buyers_resource_manager: ResourceManager,

        // Where to store the base coins to buy all of the launches
        base_coin_vault: Vault,
    }

    impl ApeInHook {

        // This is the constructor for an ApeInHook component
        pub fn new(

            // Owner badge of this component
            owner_badge_address: ResourceAddress,

            // The badge RadixPump uses to authenticate against this hook and that can be used to authenticate towards
            // a Pool
            hook_badge_address: ResourceAddress,

            // The coin to buy launches with
            base_coin_address: ResourceAddress,

            // How many quick launch a user will automatically take part
            launches_per_buyer: u16,

            // How many coins, per each user, will be used to buy a quick launch
            base_coins_per_launch: Decimal,
        ) -> Global<ApeInHook> {

            // Make sure parameters make sense
            assert!(
                launches_per_buyer > 0,
                "launches_per_buyer must be bigger than zero",
            );
            assert!(
                base_coins_per_launch > Decimal::ZERO,
                "base_coins_per_launch must be bigger than zero",
            );

            // Reserve a component address to set proper permissions on the buyer badge
            let (address_reservation, component_address) = Runtime::allocate_component_address(ApeInHook::blueprint_id());

            // Create a resource manager to mint buyer badges
            let buyers_resource_manager = ResourceBuilder::new_integer_non_fungible_with_registered_type::<ApeInBuyer>(
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
            .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                non_fungible_data_updater => rule!(require(global_caller(component_address)));
                non_fungible_data_updater_updater => rule!(require(owner_badge_address));
            ))
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => "ApeInBuyer", updatable;
                    "description" => "Automatically buy coins at launch", updatable;
                }
            ))
            .create_with_no_initial_supply();

            // Instantiate the component
            Self {
                hook_badge_address: hook_badge_address,
                launches_per_buyer: launches_per_buyer,
                base_coins_per_launch: base_coins_per_launch,
                last_buyer_id: 0,
                last_launch_id: 0,
                launches: KeyValueStore::new_with_registered_type(),
                buyers_resource_manager: buyers_resource_manager,
                base_coin_vault: Vault::new(base_coin_address),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "name" => "ApeInHook", updatable;
                }
            })
            .globalize()
        }

        // A user can call this method to deposit his base coins to buy the next quick launches
        // The number of coins he must deposit is fixed, any additional amount will be returned
        pub fn ape_in(
            &mut self,

            // Base coins to buy the quick launches
            mut base_coin_bucket: Bucket,

        ) -> (
            Bucket, // Buyer badge
            Bucket, // Eventual excess base coins provided
        ) {

            // Make sue the user sent the requested amount of base coins
            assert!(
                base_coin_bucket.resource_address() == self.base_coin_vault.resource_address(),
                "Wrong base coin",
            );
            assert!(
                base_coin_bucket.amount() >= self.base_coins_per_launch * self.launches_per_buyer,
                "Not enough base coins",
            );

            // Put only the requested amount in the component vault
            self.base_coin_vault.put(base_coin_bucket.take(self.base_coins_per_launch * self.launches_per_buyer));

            // Mint a buyer badge
            self.last_buyer_id += 1;
            let buyer_badge = self.buyers_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.last_buyer_id.into()),
                ApeInBuyer {
                    date_joined: Clock::current_time_rounded_to_seconds(),
                    last_launch_id: self.last_launch_id,
                    lunches: self.launches_per_buyer,
                    withdrawn_coins: 0,
                }
            );

            // Return the buyer badge and the eventual excess base coins to the user
            (buyer_badge, base_coin_bucket)
        }

        // A user can use this method to withdraw the coins this component bought for him
        // If the predefined number of launches is reached for the user and all of the coins have
        // been withdrawn, the buyer badge will be burned.
        pub fn withdraw_coins(
            &mut self,

            // The buyer badge given by the ape_in method
            buyer_badge: Bucket,

        ) -> Vec<Bucket> // The bought coins + eventually the buyer bucket
        {

            // Make sure the user passed just one buyer badge
            assert!(
                buyer_badge.resource_address() == self.buyers_resource_manager.address(),
                "Wrong badge",
            );
            assert!(
                buyer_badge.amount() == Decimal::ONE,
                "Can only handle one badge at a time",
            );

            // Read the contents of the badge
            let buyer_data = buyer_badge.as_non_fungible().non_fungible::<ApeInBuyer>().data();

            // Compute the first and the last launch this user took part in.
            // The total number of launches can't be bigger than MAX_COINS_WITHDRAW or the
            // transaction may fail.
            let first_launch: u64 = buyer_data.last_launch_id + u64::from(buyer_data.withdrawn_coins) + 1;
            let last_launch: u64 = min(

                // Can't withdraw future launches
                self.last_launch_id,
                min(
                    // Make sure not to hit transaction limits
                    first_launch + u64::from(MAX_COINS_WITHDRAW),

                    // Can't withdraw more than launches_per_buyer in total
                    buyer_data.last_launch_id + u64::from(self.launches_per_buyer)
                )
            );

            // If the user didn't take part in a launch yet, just return the badge
            if first_launch > last_launch {
                return vec![buyer_badge];
            }

            // Get the user share from each launch and put all of the buckets in a vector
            let mut buckets: Vec<Bucket> = vec![];
            for launch_id in first_launch..(last_launch + 1) {
                let mut launch = self.launches.get_mut(&launch_id).unwrap();

                let coin_amount = launch.coins_per_buyer;
                buckets.push(launch.vault.take(coin_amount));

if launch_id == last_launch {
    info!("launch_id: {}, remaining in vault: {}", launch_id, launch.vault.amount());
}
            }

            // Compute the total number of withdrawn coins by this user
            let withdrawn_coins: u16 = buyer_data.withdrawn_coins + u16::try_from(last_launch - first_launch).unwrap() + 1;
            if withdrawn_coins < self.launches_per_buyer {

                // If less than the maximum update the NonFungibleData in the badge
                let buyer_id = match buyer_badge.as_non_fungible().non_fungible_local_id() {
                    NonFungibleLocalId::Integer(id) => id.value(),
                    _ => Runtime::panic("Should not happen".to_string()),
                };
                self.buyers_resource_manager.update_non_fungible_data(
                    &NonFungibleLocalId::Integer(buyer_id.into()),
                    "withdrawn_coins",
                    withdrawn_coins,
                );

                // And return the badge to the user
                buckets.push(buyer_badge);
            } else {

                // Else the badge has no more use, we can burn it
                buyer_badge.burn();
            }

            buckets
        }
    }

    impl HookInterfaceTrait for ApeInHook {

        // Hook invocation method by RadixPump
        fn hook(
            &mut self,
            mut argument: HookArgument,
            hook_badge_bucket: FungibleBucket,
        ) -> (
            FungibleBucket,
            Option<Bucket>, // This is always None
            Vec<AnyPoolEvent>,
            Vec<HookArgument>,
        ) {

            // Make sure RadixPump is the caller
            assert!(
                hook_badge_bucket.resource_address() == self.hook_badge_address && hook_badge_bucket.amount() == Decimal::ONE,
                "Wrong badge",
            );

            // Proceed only for PostQuickLaunch operations and if some buyer joined
            if argument.operation != HookableOperation::PostQuickLaunch || self.last_buyer_id == 0 {
                return (hook_badge_bucket, None, vec![], vec![]);
            }

            // Who is the first buyer for this launch?
            let first_buyer_id = match self.last_launch_id.cmp(&self.launches_per_buyer.into()) {

                // If less than launches_per_buyer happened, he is the first one
                Ordering::Less => 1,

                // Else get the last buyer that joined more than launches_per_buyer launches ago,
                // he is the last one among the ones who don't have to take parte
                _ => self.launches.get(&(self.last_launch_id - u64::from(self.launches_per_buyer) + 1u64)).unwrap().last_buyer_id + 1,
            };

            // So, how many buyers?
            let buyers = self.last_buyer_id - first_buyer_id + 1;
            if buyers < 1 {
                return (hook_badge_bucket, None, vec![], vec![]);
            }

            // Take the same amount for each buyer
            let base_coin_bucket = self.base_coin_vault.take(self.base_coins_per_launch * buyers);

            // Buy the launched coin
            let (coin_bucket, new_hook_argument, event) = hook_badge_bucket.authorize_with_amount(
                    1,
                    || argument.component.buy(base_coin_bucket)
            );

            // Add this coin to the list
            self.last_launch_id += 1;
            self.launches.insert(
                self.last_launch_id,
                CoinLaunch {
                    last_buyer_id: self.last_buyer_id,
                    coins_per_buyer: coin_bucket.amount() / buyers,
                    vault: Vault::with_bucket(coin_bucket),
                }
            );

            // Return the hook badge, the BuyEvent and the new argument for the hooks
            (hook_badge_bucket, None, vec![event], vec![new_hook_argument])
        }

        // Round 0, non accepting calls trigered by other hooks
        fn get_hook_info(&self) -> (HookExecutionRound, bool) {(0, false)}
    }
}

