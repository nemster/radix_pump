use scrypto::prelude::*;
use crate::common::*;
use scrypto_interface::*;

// This blueprint implements a RadixPump hook that can be used by a coin creator to reward his
// liquidity providers, the rewards can be any fungible.
// Liquidity providers can withdraw rewards by calling the get_rewards method or automatically when
// they remove liquidity.
// For the automatical rewards withdraw to work it is needed that both the AddLiquidity and 
// RemoveLiquidity operations are intercepted by this hook so it is advisable to enable the
// hook for AddLiquidity globally as soon as possible and let coin creators enable/disable the
// hook for RemoveLiquidity when they start/end campaigns.

// This struct contains informations about an LP token
#[derive(ScryptoSbor)]
struct LiquidityProvider {
    amount: Decimal,
    last_rewards_withdraw_time: i64
}

// This struct contains informations about a liquidity campaign
#[derive(ScryptoSbor)]
struct LiquidityCampaign {
    start_time: i64,
    end_time: i64,
    daily_reward_per_coin: Decimal,
    rewards_vault: Vault,
    lp_address: ResourceAddress,
}

// It is not good for a hook to panic when invoked by ReadixPump, better go on and emit an alert
// event
// This event is emitted if an ongoing liquidity campaign has not enough coins to reward a
// liquidity provider that removed his liquidity
#[derive(ScryptoSbor, ScryptoEvent)]
struct OutOfFundsEvent {
    coin_address: ResourceAddress,
    lp_ids: Vec<u64>,
}

// It is not good for a hook to panic when invoked by ReadixPump, better go on and emit an alert
// event
// This event is emitted if there are no informations about the amount of rewards to give a
// liquidity provider that removed his liquidity; this happens if the hook did not intercept the
// AddLiquidity operation
#[derive(ScryptoSbor, ScryptoEvent)]
struct UnknownRewardAmountEvent {
    coin_address: ResourceAddress,
    lp_ids: Vec<u64>
}

// Make everybody know when a liquidity campaign starts or is updated
#[derive(ScryptoSbor, ScryptoEvent)]
struct LiquidityCampaignCreationEvent {
    coin_address: ResourceAddress,
    start_time: i64,
    end_time: i64,
    daily_reward_per_coin: Decimal,
    rewards_amount: Decimal,
}

// Key for liquidity_providers KVS
type LpIdentifier = (ResourceAddress, u64);

static SECONDS_PER_DAY: u32 = 86400;

#[blueprint_with_traits]
#[events(
    OutOfFundsEvent,
    UnknownRewardAmountEvent,
    LiquidityCampaignCreationEvent,
)]
#[types(
    LpIdentifier,
    LiquidityProvider,
    u32,
    LiquidityCampaign,
    ResourceAddress,
)]
mod lp_rewards_hook {

    enable_method_auth! {
        roles {
            proxy => updatable_by: [OWNER];
        },
        methods {
            new_liquidity_campaign => PUBLIC;
            update_liquidity_campaign => PUBLIC;
            terminate_liquidity_campaign => PUBLIC;
            get_rewards => PUBLIC;
            hook => restrict_to: [proxy];
            get_hook_info => PUBLIC;
        }
    }

    struct LpRewardsHook {

        // The resource address of the creator badges minted by RadixPump
        coin_creator_badge_address: ResourceAddress,

        // Informations about existing and past liquidity providers
        liquidity_providers: KeyValueStore<LpIdentifier, LiquidityProvider>,

        // All of the present and past liquidity campaigns
        liquidity_campaigns: KeyValueStore<u32, LiquidityCampaign>,
        last_liquidity_campaign_id: u32,

        // Active liquidity campaigns per coin (just the index to the liquidity_campaigns KVS)
        active_campaign: KeyValueStore<ResourceAddress, u32>,

        // A coin creator can withdraw any remaining funds from a campaign aster this number of
        // seconds has passed since the end of the campaign (give liquidity providers enough time
        // to get their rewards)
        grace_period: i64,
    }

    impl LpRewardsHook {

        // This function instantiates a LpRewardsHook component
        pub fn new(

            // Owner badge of this component
            owner_badge_address: ResourceAddress,

            // The badge RadixPump will use to authenticate against this hook
            proxy_badge_address: ResourceAddress,

            // The resource address of the creator badges minted by RadixPump
            coin_creator_badge_address: ResourceAddress,

            // A coin creator can withdraw any remaining funds from a campaign aster this number of
            // seconds has passed since the end of the campaign (give liquidity providers enough time
            // to get their rewards)
            grace_period: i64,
        ) -> Global<LpRewardsHook> {
            Self {
                coin_creator_badge_address: coin_creator_badge_address,
                liquidity_providers: KeyValueStore::new_with_registered_type(),
                liquidity_campaigns: KeyValueStore::new_with_registered_type(),
                last_liquidity_campaign_id: 0,
                active_campaign: KeyValueStore::new_with_registered_type(),
                grace_period: grace_period,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                proxy => rule!(require(proxy_badge_address));
            ))
            .metadata(metadata! {
                init {
                    "name" => "LpRewardsHook", updatable;
                }
            })
            .globalize()
        }

        // A coin creator can call this method to create a liquidity campaign for his coin
        // It is not possible to start a new liquidity campaign for a coin while another one is
        // ongoing for the same coin
        pub fn new_liquidity_campaign(
            &mut self,

            // Proof of the coin creator badge minted by RadixPump
            coin_creator_proof: Proof,

            // Start time of the liquidity campaign
            start_time: i64,

            // End time of the liquidity campaign
            end_time: i64,

            // How much reward a liquidity provider that deposited one coin for a day
            daily_reward_per_coin: Decimal,

            // The bucket containing the rewards to distribute
            rewards_bucket: Bucket,
        ) {
            assert!(
                end_time > start_time,
                "End time must be bigger than start time",
            );

            // Check the creator proof and get informations about his coin
            let (coin_address, lp_address) = self.check_creator_proof(coin_creator_proof);

            assert!(
                self.active_campaign.get(&coin_address).is_none(),
                "There's already an active liquidity campaign for your coin",
            );

            let rewards_amount = rewards_bucket.amount();

            // Create the campaign and set it as active
            self.last_liquidity_campaign_id += 1;
            self.liquidity_campaigns.insert(
                self.last_liquidity_campaign_id,
                LiquidityCampaign {
                    start_time: start_time,
                    end_time: end_time,
                    daily_reward_per_coin: daily_reward_per_coin,
                    rewards_vault: Vault::with_bucket(rewards_bucket),
                    lp_address: lp_address,
                }
            );
            self.active_campaign.insert(coin_address, self.last_liquidity_campaign_id);

            // Notify people about this opportunity
            Runtime::emit_event(
                LiquidityCampaignCreationEvent {
                    coin_address: coin_address,
                    start_time: start_time,
                    end_time: end_time,
                    daily_reward_per_coin: daily_reward_per_coin,
                    rewards_amount: rewards_amount,
                }
            );
        }

        // Pospone the end of an existing campaign or add funds to it
        pub fn update_liquidity_campaign(
            &mut self,

            // Proof of the coin creator badge minted by RadixPump
            coin_creator_proof: Proof,

            // Eventual new end time of the liquidity campaign
            end_time: Option<i64>,

            // Eventual additional funds for the campaign
            rewards_bucket: Option<Bucket>,
        ) {

            // Check the creator proof and get informations about his coin
            let (coin_address, _) = self.check_creator_proof(coin_creator_proof);

            let mut campaign = self.liquidity_campaigns.get_mut(
                &self.active_campaign.get(&coin_address).expect("No liquidity campaign for your coin")
            ).unwrap();

            // Update end time if requested
            match end_time {
                Some(end_time) => {
                    assert!(
                        end_time > campaign.end_time,
                        "You can only pospone the end of a campaign",
                    );

                    campaign.end_time = end_time;
                },
                None => {
                    assert!(
                        rewards_bucket.is_some(),
                        "No update requested",
                    );
                },
            }

            // Add funds if requested
            match rewards_bucket {
                Some(rewards_bucket) => {
                    assert!(
                        rewards_bucket.amount() > Decimal::ZERO,
                        "Empty bucket",
                    );

                    campaign.rewards_vault.put(rewards_bucket);
                },
                None => {},
            }

            // Let people know
            Runtime::emit_event(
                LiquidityCampaignCreationEvent {
                    coin_address: coin_address,
                    start_time: campaign.start_time,
                    end_time: campaign.end_time,
                    daily_reward_per_coin: campaign.daily_reward_per_coin,
                    rewards_amount: campaign.rewards_vault.amount(),
                }
            );
        }

        // A coin creator can call this method to close a terminated liquidity campaign and get any
        // remaining funds back
        // He has to give liquidity providers enough time to withdraw their rewards before doing
        // this
        pub fn terminate_liquidity_campaign(
            &mut self,

            // Proof of the coin creator badge minted by RadixPump
            coin_creator_proof: Proof,
        ) -> Bucket {

            // Check the creator proof and get informations about his coin
            let (coin_address, _) = self.check_creator_proof(coin_creator_proof);

            // Remove the campaign from the active list and get the details about it
            let mut campaign = self.liquidity_campaigns.get_mut(
                &self.active_campaign.remove(&coin_address).expect("No liquidity campaign for your coin")
            )
            .unwrap();

            // Make sure the campaign has ended and some time has passed since then
            assert!(
                campaign.end_time + self.grace_period <= Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch,
                "Too soon",
            );

            // Take back any remaining rewards
            let rewards_bucket = campaign.rewards_vault.take_all();

            rewards_bucket
        }

        pub fn get_rewards(
            &mut self,
            lp_proof: Proof,
        ) -> Bucket {

            let lp_address = lp_proof.resource_address();

            // Let's skip the check for now, there are many valids LP addresses
            let checked_proof = lp_proof.skip_checking();

            let non_fungible_vec = checked_proof.as_non_fungible().non_fungibles::<LPData>();

            // The coin_resource_address is the same for all LP tokens having the same resource address, just take the
            // first one
            let coin_address = non_fungible_vec[0].data().coin_resource_address;

            let mut campaign = self.liquidity_campaigns.get_mut(
                &self.active_campaign.get(&coin_address).expect("No active campaign for this coin")
            )
            .unwrap();
           
            // Now we can check that the liquidity token is not fake
            assert!(
                campaign.lp_address == lp_address,
                "Wrong LP token",
            );

            let mut rewards_amount = Decimal::ZERO;
            let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
            let reward_per_coin_per_second = campaign.daily_reward_per_coin / SECONDS_PER_DAY;

            // For each non fugible in the proof
            for non_fungible in non_fungible_vec {

                // Find the id
                let non_fungible_id = match non_fungible.local_id() {
                    NonFungibleLocalId::Integer(id) => id.value(),
                    _ => Runtime::panic("Should not happen".to_string()),
                };

                // Do we already have information about this LP token?
                let mut liquidity_provider = self.liquidity_providers.get_mut(
                    &(coin_address, non_fungible_id)
                );

                // start_time is the biggest among:
                // - LP token mint time
                // - last time rewards were withdrawn (if any)
                // - liquidity campaign start
                let mut start_time = match liquidity_provider {
                    None => {
                        drop(liquidity_provider);

                        self.liquidity_providers.insert(
                            (coin_address, non_fungible_id),
                            LiquidityProvider {
                                amount: non_fungible.data().deposited_coins,
                                last_rewards_withdraw_time: now
                            }
                        );

                        non_fungible.data().date.seconds_since_unix_epoch
                    },
                    Some(ref mut lp) => {
                        let last_time = lp.last_rewards_withdraw_time;
                        lp.last_rewards_withdraw_time = now;
                        last_time
                    },
                };
                if campaign.start_time > start_time {
                    start_time = campaign.start_time;
                }

                // Add rewards for this LP to the total
                rewards_amount += non_fungible.data().deposited_coins *
                    reward_per_coin_per_second *
                    (now - start_time);
            }

            // Return the whole rewards
            campaign.rewards_vault.take_advanced(
                rewards_amount,
                WithdrawStrategy::Rounded(RoundingMode::ToZero)
            )
        }

        // Verify a coin creator proof and get informations about the coin he created
        fn check_creator_proof(
            &self,

            // The proof to check
            coin_creator_proof: Proof,
        ) -> (
            ResourceAddress, // Coin address
            ResourceAddress, // LP token address
        ) {
            let checked_proof = coin_creator_proof.check_with_message(
                self.coin_creator_badge_address,
                "Wrong badge",
            );

            // Get the NonFungibleData
            let non_fungible_data = checked_proof.as_non_fungible().non_fungible::<CreatorData>().data();

            (non_fungible_data.coin_resource_address, non_fungible_data.lp_token_address)
        }
    }

    impl HookInterfaceTrait for LpRewardsHook {

        // Hook invocation method by RadixPump
        fn hook(
            &mut self,
            argument: HookArgument,
            hook_badge_bucket: Option<FungibleBucket>, // None
        ) -> (
            Option<FungibleBucket>, // None
            Option<Bucket>,
            Vec<AnyPoolEvent>, // Always empty
            Vec<HookArgument>, // Always empty
        ) {

            match argument.operation {

                // In case of a AddLiquidity operation, just take note of the amount and mint
                // time of the LP token
                HookableOperation::AddLiquidity => {
                    self.liquidity_providers.insert(
                        (argument.coin_address, argument.ids[0]),
                        LiquidityProvider {
                            amount: argument.amount.unwrap(),
                            last_rewards_withdraw_time: Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch,
                        }
                    );

                    (hook_badge_bucket, None, vec![], vec![])
                },

                // In case of a RemoveLiquidity we have to give the rewards
                HookableOperation::RemoveLiquidity => {
                    
                    // Is there any active campaign for this coin?
                    // If not just quit
                    let campaign_id = self.active_campaign.get(&argument.coin_address);
                    if campaign_id.is_none() {
                        return (hook_badge_bucket, None, vec![], vec![]);
                    }
                    let mut campaign = self.liquidity_campaigns.get_mut(&campaign_id.unwrap()).unwrap();

                    let mut rewards_amount = Decimal::ZERO;
                    let mut unknown_ids: Vec<u64> = vec![];
                    let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
                    let reward_per_coin_per_second = campaign.daily_reward_per_coin / SECONDS_PER_DAY;

                    // It is possible to remove multiple LP tokens in a single operation, let's loop
                    // amoung them
                    for lp_id in argument.ids.iter() {

                        // Search information about che LP toke
                        let lp = self.liquidity_providers.get(
                            &(argument.coin_address, *lp_id)
                        );

                        match lp {

                            // If there are no informations about it there's nothing we can do (it
                            // has already been burned). Just keep track of the issue
                            None => unknown_ids.push(*lp_id),

                            // If found, add the rewards for this LP token to the total rewards
                            Some(lp) => {
                                rewards_amount += lp.amount *
                                reward_per_coin_per_second *
                                (now - lp.last_rewards_withdraw_time);
                            },
                        }
                    }

                    // If there are not enough rewards for this user, don't panic, just emit an
                    // event so people know
                    if campaign.rewards_vault.amount() < rewards_amount {
                        Runtime::emit_event(
                            OutOfFundsEvent {
                                coin_address: argument.coin_address,
                                lp_ids: argument.ids,
                            }
                        );

                        return (hook_badge_bucket, None, vec![], vec![]);
                    } else {

                        // Emit the event about the non found LPs
                        if unknown_ids.len() > 0 {
                            Runtime::emit_event(
                                UnknownRewardAmountEvent {
                                    coin_address: argument.coin_address,
                                    lp_ids: unknown_ids,
                                }
                            );
                        }

                        // Give the user his rewards
                        return (
                            hook_badge_bucket,
                            Some(
                                campaign.rewards_vault.take_advanced(
                                    rewards_amount,
                                    WithdrawStrategy::Rounded(RoundingMode::ToZero)
                                )
                            ),
                            vec![],
                            vec![]
                        );
                    }
                },
                _ => (hook_badge_bucket, None, vec![], vec![]),
            }
        }

        // Round 2, non accepting calls triggered by other hooks
        fn get_hook_info(&self) -> (HookExecutionRound, bool) {(2, false)}
    }
}

