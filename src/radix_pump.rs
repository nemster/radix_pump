use scrypto::prelude::*;
use crate::pool::*;

// Metadata for the coin creator badge
static COIN_CREATOR_BADGE_NAME: &str = "Coin creator badge";

#[derive(Debug, ScryptoSbor, NonFungibleData)]
struct CoinOwnerData {
    coin_resource_address: ResourceAddress,
    creation_date: Instant,
}

#[blueprint]
#[events(NewCoinEvent, BuyEvent, SellEvent)]
#[types(u64, CoinOwnerData)]
mod radix_pump {

    enable_method_auth! {
        methods {
            forbid_symbols => restrict_to: [OWNER];
            forbid_names => restrict_to: [OWNER];
            create_new_coin => PUBLIC;
            buy => PUBLIC;
            sell => PUBLIC;
            get_fees => restrict_to: [OWNER];
            update_fees => restrict_to: [OWNER];
        }
    }

    enable_package_royalties! {
        new => Free;
        forbid_symbols => Free;
        forbid_names => Free;
        create_new_coin => Usd(dec!("0.05"));
        buy => Usd(dec!("0.005"));
        sell => Usd(dec!("0.005"));
        get_fees => Free;
        update_fees => Free;
    }

    struct RadixPump {
        base_coin_address: ResourceAddress,
        minimum_deposit: Decimal,
        coin_creator_badge_resource_manager: ResourceManager,
        last_coin_creator_badge_id: u64,
        forbidden_symbols: KeyValueStore<String, u64>,
        forbidden_names: KeyValueStore<String, u64>,
        coins_supply: Decimal,
        pools: KeyValueStore<ResourceAddress, Pool>,
        creation_fee_percentage: Decimal,
        buy_sell_fee_percentage: Decimal,
        fee_vault: Vault,
    }

    impl RadixPump {

        pub fn new(
            owner_badge_address: ResourceAddress,
            base_coin_address: ResourceAddress,
            minimum_deposit: Decimal,
            coins_supply: Decimal,
            creation_fee_percentage: Decimal,
            buy_sell_fee_percentage: Decimal,
        ) -> Global<RadixPump> {

            assert!(
                minimum_deposit > Decimal::ZERO,
                "Minimum deposit can't be zero or less",
            );
            assert!(
                coins_supply > Decimal::ZERO,
                "Coins supply can't be zero or less",
            );
            assert!(
                creation_fee_percentage >= Decimal::ZERO && creation_fee_percentage < dec!(100),
                "Creation fee percentage can go from 0 (included) to 100 (excluded)",
            );
            assert!(
                buy_sell_fee_percentage >= Decimal::ZERO && buy_sell_fee_percentage < dec!(100),
                "Buy & sell fee percentage can go from 0 (included) to 100 (excluded)",
            );

            // Reserve a ComponentAddress for setting rules on resources
            let (address_reservation, component_address) = Runtime::allocate_component_address(RadixPump::blueprint_id());

            // Create a ResourceManager for minting coin_creator badges
            let coin_creator_badge_resource_manager = ResourceBuilder::new_integer_non_fungible_with_registered_type::<CoinOwnerData>(
                OwnerRole::Updatable(rule!(require(owner_badge_address)))
            )
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => COIN_CREATOR_BADGE_NAME, updatable;
                }
            ))
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(deny_all);
            ))
            .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                non_fungible_data_updater => rule!(deny_all);
                non_fungible_data_updater_updater => rule!(deny_all);
            ))
            .burn_roles(burn_roles!(
                burner => rule!(allow_all);
                burner_updater => rule!(require(owner_badge_address));
            ))
            .create_with_no_initial_supply();

            // Instantiate the component
            Self {
                base_coin_address: base_coin_address.clone(),
                minimum_deposit: minimum_deposit,
                coin_creator_badge_resource_manager: coin_creator_badge_resource_manager,
                last_coin_creator_badge_id: 0,
                forbidden_symbols: KeyValueStore::new(),
                forbidden_names: KeyValueStore::new(),
                coins_supply: coins_supply,
                pools: KeyValueStore::new(),
                creation_fee_percentage: creation_fee_percentage,
                buy_sell_fee_percentage: buy_sell_fee_percentage,
                fee_vault: Vault::new(base_coin_address),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .with_address(address_reservation)
            .globalize()
        }

        pub fn forbid_symbols(
            &mut self,
            symbols: Vec<String>,
        ) {
            for symbol in symbols.iter() {
                self.forbidden_symbols.insert(symbol.trim().to_uppercase(), 0);
            }
        }

        pub fn forbid_names(
            &mut self,
            names: Vec<String>,
        ) {
            for name in names.iter() {
                self.forbidden_names.insert(name.trim().to_uppercase(), 0);
            }
        }

        pub fn create_new_coin(
            &mut self,
            mut base_coin_bucket: Bucket,
            mut coin_symbol: String,
            mut coin_name: String,
            coin_icon_url: String,
            coin_description: String,
        ) -> (Bucket, Bucket) {

            assert!(
                base_coin_bucket.resource_address() == self.base_coin_address,
                "Wrong base coin deposited",
            );
            assert!(
                base_coin_bucket.amount() >= self.minimum_deposit,
                "Insufficient base coin deposit",
            );

            self.last_coin_creator_badge_id += 1;

            // Enforce uniqueness of coins' symbols and names
            coin_symbol = coin_symbol.trim().to_uppercase();
            assert!(
                coin_symbol.len() > 0,
                "Coin symbol can't be empty",
            );
            assert!(
                self.forbidden_symbols.get(&coin_symbol).is_none(),
                "Symbol already used",
            );
            self.forbidden_symbols.insert(coin_symbol.clone(), self.last_coin_creator_badge_id);
            coin_name = coin_name.trim().to_string();
            assert!(
                coin_name.len() > 0,
                "Coin name can't be empty",
            );
            let uppercase_coin_name = coin_name.to_uppercase();
            assert!(
                self.forbidden_names.get(&uppercase_coin_name).is_none(),
                "Name already used",
            );
            self.forbidden_names.insert(uppercase_coin_name, self.last_coin_creator_badge_id);

            let coin_creator_badge_rule = AccessRule::Protected(
                AccessRuleNode::ProofRule(
                    ProofRule::Require (
                        ResourceOrNonFungible::NonFungible (
                            NonFungibleGlobalId::new(
                                self.coin_creator_badge_resource_manager.address(),
                                NonFungibleLocalId::integer(self.last_coin_creator_badge_id.into()),
                            )
                        )
                    )
                )
            );

            let coin_bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::Fixed(coin_creator_badge_rule.clone()))
            .metadata(metadata!(
                roles {
                    metadata_setter => coin_creator_badge_rule.clone();
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => coin_creator_badge_rule.clone();
                    metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "symbol" => coin_symbol, locked;
                    "name" => coin_name, locked;
                    "icon_url" => MetadataValue::Url(UncheckedUrl::of(coin_icon_url)), updatable;
                    "description" => coin_description, updatable;
                }
            ))
            .mint_roles(mint_roles!(
                minter => rule!(deny_all);
                minter_updater => rule!(deny_all);
            ))
            .burn_roles(burn_roles!(
                burner => coin_creator_badge_rule.clone();
                burner_updater => coin_creator_badge_rule;
            ))
            .divisibility(DIVISIBILITY_MAXIMUM)
            .mint_initial_supply(self.coins_supply)
            .into();

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    self.creation_fee_percentage * base_coin_bucket.amount() / 100,
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            let coin_creator_badge_bucket = self.coin_creator_badge_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.last_coin_creator_badge_id.into()),
                CoinOwnerData {
                    coin_resource_address: coin_bucket.resource_address(),
                    creation_date: Clock::current_time_rounded_to_seconds(),
                }
            );

            let (pool, coin_creator_coin_bucket) = Pool::new(
                base_coin_bucket, 
                coin_bucket,
            );
            self.pools.insert(
                coin_creator_coin_bucket.resource_address(),
                pool,
            );

            (coin_creator_badge_bucket, coin_creator_coin_bucket)
        }

        pub fn buy(
            &mut self,
            coin_address: ResourceAddress,
            mut base_coin_bucket: Bucket,
        ) -> Bucket {
            assert!(
                base_coin_bucket.resource_address() == self.base_coin_address,
                "Wrong base coin",
            );

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    base_coin_bucket.amount() * self.buy_sell_fee_percentage / dec!(100),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            self.pools.get_mut(&coin_address)
            .expect("Coin not found")
            .buy(base_coin_bucket)
        }

        pub fn sell(
            &mut self,
            coin_bucket: Bucket,
        ) -> Bucket {
            let mut base_coin_bucket = self.pools.get_mut(&coin_bucket.resource_address())
            .expect("Coin not found")
            .sell(coin_bucket);

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    base_coin_bucket.amount() * self.buy_sell_fee_percentage / dec!(100),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            base_coin_bucket
        }

        pub fn get_fees(
            &mut self,
        ) -> Bucket {
            self.fee_vault.take_all()
        }

        pub fn update_fees(
            &mut self,
            creation_fee_percentage: Decimal,
            buy_sell_fee_percentage: Decimal,
        ) {
            assert!(
                creation_fee_percentage >= Decimal::ZERO && creation_fee_percentage < dec!(100),
                "Creation fee percentage can go from 0 (included) to 100 (excluded)",
            );  
            assert!(
                buy_sell_fee_percentage >= Decimal::ZERO && buy_sell_fee_percentage < dec!(100),
                "Buy & sell fee percentage can go from 0 (included) to 100 (excluded)",
            );

            self.creation_fee_percentage = creation_fee_percentage;
            self.buy_sell_fee_percentage = buy_sell_fee_percentage;
        }
    }
}