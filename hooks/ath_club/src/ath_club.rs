use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;
use crate::ath_club_data::*;

/* This blueprint implements a RadixPump hook that reward users with an NFT when they buy an ATH.
   The ATH Club NFT contains information about the coin and price bought and if the ATH is still valid or has been passed (obsoleted) by a new one.
   Coin creators have to enable this hook for the Buy operation for it to work for their coins.
*/

// Internal representation of an ATH
#[derive(ScryptoSbor)]
struct Ath {

    // Local id of the NFT minted to celebrate the last ATH
    nft_id: Option<u64>,

    // Last ATH price
    price: Decimal,

    // Minimum amount of bought coins for a new ATH to be accepted
    min_amount: Decimal,
}

#[blueprint_with_traits]
#[types(
    ResourceAddress,
    Ath,
    AthClubData,
)]
mod ath_club_hook {

    enable_method_auth! {
        roles {
            proxy => updatable_by: [OWNER];
        },
        methods {
            init_coin => PUBLIC;
            hook => restrict_to: [proxy];
            get_hook_info => PUBLIC;
        }
    }

    struct AthClubHook {

        // The resource manager to mint the ATH Club NFTs
        ath_club_resource_manager: ResourceManager,

        // Numeric id of the last minted NFT
        last_ath_club_id: u64,

        // Internale storage of the last minted NFT per coin
        aths: KeyValueStore<ResourceAddress, Ath>,

        // The resource address of the badge given by RadixPump to coin creators
        coin_creator_badge_address: ResourceAddress,

        // The default image to use if a coin has no coin_url metadata (should not happen)
        default_image_url: UncheckedUrl,
    }

    impl AthClubHook {

        // This function instantiates a AthClubHook component
        pub fn new(

            // Owner badge of this component
            owner_badge_address: ResourceAddress,

            // The badge RadixPump will use to authenticate against this hook
            proxy_badge_address: ResourceAddress,

            // The resource address of the creator badges minted by RadixPump
            coin_creator_badge_address: ResourceAddress,

            // The default image to use if a coin has no icon_url metadata (should not happen)
            default_image_url: String,

        ) -> Global<AthClubHook> {

            // Reserve a component address to set proper permissions on the ATH Club NFT
            let (address_reservation, component_address) = Runtime::allocate_component_address(AthClubHook::blueprint_id());

            // Create a resource manager to mint ATH Club NFTs
            let ath_club_resource_manager = ResourceBuilder::new_integer_non_fungible_with_registered_type::<AthClubData>(
                OwnerRole::Updatable(rule!(require(owner_badge_address)))
            )
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(owner_badge_address));
            ))
            .burn_roles(burn_roles!(
                burner => rule!(deny_all);
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
                    "name" => "ATH Club", locked;
                    "tags" => vec!["NFT", "Collectible"], updatable;
                }
            ))
            .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                non_fungible_data_updater => rule!(require(global_caller(component_address)));
                non_fungible_data_updater_updater => rule!(require(owner_badge_address));
            ))
            .create_with_no_initial_supply();

            // Instantiate the component
            Self {
                ath_club_resource_manager: ath_club_resource_manager,
                last_ath_club_id: 0,
                aths: KeyValueStore::new_with_registered_type(),
                coin_creator_badge_address: coin_creator_badge_address,
                default_image_url: UncheckedUrl::of(default_image_url),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
            ))
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "name" => "AthClubHook", updatable;
                }
            })
            .globalize()
        }

        // A coin creator may want to initialize this hook with past ATH value and set a minimum amount of bought
        // coins for a new ATH to be registered.
        // This method can't be used if an ATH Club NFT has already been minted for the coin.
        pub fn init_coin(
            &mut self,

            // Proof that you are the creator of a coin
            coin_creator_proof: Proof,

            // Past ATH price
            ath_price: Decimal,

            // Minimum amount of bought coins for a new ATH to be accepted
            min_amount: Decimal,
        ) {

            // Verify the coin creator proof
            let checked_proof = coin_creator_proof.check_with_message(
                self.coin_creator_badge_address,
                "Wrong badge",
            );

            // Make sure the proof contains exactly one badge and get the address of the coin
            // created coin by this user
            let coin_resource_address = checked_proof.as_non_fungible().non_fungible::<CreatorData>().data().coin_resource_address;

            assert!(
                self.aths.get(&coin_resource_address).is_none(),
                "An ATH Club NFT for this coin has already been minted",
            );

            assert!(
                min_amount >= Decimal::ZERO,
                "Minimum amount can't be a negative number",
            );

            // Add this ATH to the list without a cossesponding NFT local id
            self.aths.insert(
                coin_resource_address,
                Ath {
                    nft_id: None,
                    price: ath_price,
                    min_amount: min_amount,
                }
            );
        }

        // Private method to mint a new ATH Club NFT (called by the hook method)
        fn mint(
            &mut self,

            // The argument to the hook method
            argument: &HookArgument,

        ) -> Bucket {

            // Prepare the non fungible local id for the new NFT
            self.last_ath_club_id += 1;
            let nft_id = NonFungibleLocalId::integer(self.last_ath_club_id.into());

            // Build the resource manager of the coin to read the metadata
            let coin_resource_manager = ResourceManager::from_address(argument.coin_address);

            // Get coin symbol
            let coin_symbol: String = match coin_resource_manager.get_metadata("symbol") {
                Ok(opt_symbol) => match opt_symbol {
                    Some(symbol) => symbol,
                    None => "Unknown".to_string(), // Should not happen but better safe than sorry
                },
                Err(_) => "Unknown".to_string(), // Should not happen but better safe than sorry
            };

            // Get coin icon_url to use it ad key_image_url for the NFT
            let coin_icon_url: UncheckedUrl = match coin_resource_manager.get_metadata("icon_url") {
                Ok(opt_url) => match opt_url {
                    Some(url) => url,
                    None => self.default_image_url.clone(), // Should not happen but better safe
                                                            // than sorry
                },
                Err(_) => self.default_image_url.clone(), // Should not happen but better safe than
                                                          // sorry
            };

            // Mint the NFT and return it
            self.ath_club_resource_manager.mint_non_fungible(
                &nft_id,
                AthClubData {
                    coin_address: argument.coin_address,
                    coin_symbol: coin_symbol,
                    price: argument.price,
                    date: Clock::current_time_rounded_to_seconds(),
                    key_image_url: coin_icon_url,
                    obsoleted_by: 0, // Zero stands for None
                }
            )
        }
    }

    impl HookInterfaceTrait for AthClubHook {

        // Hook invocation method by RadixPump
        fn hook(
            &mut self,
            argument: HookArgument,
            hook_badge_bucket: Option<FungibleBucket>, // Should be None
        ) -> (
            Option<FungibleBucket>,
            Option<Bucket>,
            Vec<AnyPoolEvent>, // Always empty
            Vec<HookArgument>, // Always empty
        ) {
            if argument.operation != HookableOperation::Buy {
                return (hook_badge_bucket, None, vec![], vec![]);
            }

            // Is there already an ATH for this coin?
            let mut previous_ath = self.aths.get_mut(&argument.coin_address);
            let ath_club_nft = match previous_ath {

                // If not
                None => {

                    // Release the mutable borrow
                    drop(previous_ath);

                    // Mint the new NFT
                    let ath_club_nft = self.mint(&argument);

                    // And add it to the list
                    self.aths.insert(
                        argument.coin_address,
                        Ath {
                            nft_id: Some(self.last_ath_club_id),
                            price: argument.price,
                            min_amount: Decimal::ZERO,
                        }
                    );

                    Some(ath_club_nft)
                },

                // If yes
                Some(ref mut ath) => {

                    // Check if we passed it
                    if argument.price > ath.price && argument.amount.unwrap() >= ath.min_amount {

                    // If so, obsolete the previous ATH for this coin
                    if ath.nft_id.is_some() {
                        self.ath_club_resource_manager.update_non_fungible_data(
                            &NonFungibleLocalId::integer(ath.nft_id.unwrap().into()),
                            "obsoleted_by",
                            self.last_ath_club_id + 1
                        );
                    }

                    // Register new ATH
                    ath.nft_id = Some(self.last_ath_club_id + 1);
                    ath.price = argument.price;

                    // Release the mutable borrow
                    drop(previous_ath);

                    // Mint the new NFT
                    Some(self.mint(&argument))

                } else {
                    None // Below previous ATH, nothing to do
                }
            },
        };

        (hook_badge_bucket, ath_club_nft, vec![], vec![])
    }

    // Round 2, non accepting calls triggered by other hooks
    fn get_hook_info(&self) -> (HookExecutionRound, bool) {(2, false)}
    }
}
