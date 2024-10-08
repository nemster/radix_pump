use scrypto::prelude::*;
use scrypto::prelude::rust::cmp::max;
use crate::pool::*;

// Metadata for the coin creator badge
static COIN_CREATOR_BADGE_NAME: &str = "Coin creator badge";

// Metadata for the flash loan transient NFT
static TRANSIENT_NFT_NAME: &str = "Flash loan transient NFT";

// Maximum allwed supply
// Although the math should be safe thans to the use of PreciseDecimal and properly optimized
// formulas, for additional security it's safer to limit the number of coins to less than
// the square root of Decimal::MAX
static MAX_SUPPLY: Decimal = dec!("100000000000000000000");

// Coin creator badge NFT data
#[derive(Debug, ScryptoSbor, NonFungibleData)]
struct CoinCreatorData {
    coin_resource_address: ResourceAddress,
    creation_date: Instant,
}

// Flash loan transient NFT data
#[derive(Debug, ScryptoSbor, NonFungibleData)]
struct FlashLoanData {
    coin_resource_address: ResourceAddress,
    coin_amount: Decimal,
    price: Decimal,
}

#[blueprint]
#[events(
    NewCoinEvent,
    BuyEvent,
    SellEvent,
    LiquidationEvent,
    FlashLoanEvent,
)]
#[types(
    u64,
    CoinCreatorData,
    FlashLoanData,
)]
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
            owner_set_liquidation_mode => restrict_to: [OWNER];
            creator_set_liquidation_mode => PUBLIC;
            get_flash_loan => PUBLIC;
            return_flash_loan => PUBLIC;
            update_flash_loan_pool_fee_percentage => PUBLIC;
            get_pool_info => PUBLIC;
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
        owner_set_liquidation_mode => Free;
        creator_set_liquidation_mode => Free;
        get_flash_loan => Usd(dec!("0.001"));
        return_flash_loan => Free;
        update_flash_loan_pool_fee_percentage => Free;
        get_pool_info => Free;
    }

    struct RadixPump {
        base_coin_address: ResourceAddress,
        minimum_deposit: Decimal,
        coin_creator_badge_resource_manager: ResourceManager,
        flash_loan_nft_resource_manager: ResourceManager,
        last_coin_creator_badge_id: u64,
        forbidden_symbols: KeyValueStore<String, u64>,
        forbidden_names: KeyValueStore<String, u64>,
        pools: KeyValueStore<ResourceAddress, Pool>,
        creation_fee_percentage: Decimal,
        buy_sell_fee_percentage: Decimal,
        flash_loan_fee_percentage: Decimal,
        fee_vault: Vault,
    }

    impl RadixPump {

        // Component inteantiation
        pub fn new(
            owner_badge_address: ResourceAddress,
            base_coin_address: ResourceAddress,
            minimum_deposit: Decimal,
            creation_fee_percentage: Decimal,
            buy_sell_fee_percentage: Decimal,
            flash_loan_fee_percentage: Decimal,
        ) -> Global<RadixPump> {

            assert!(
                minimum_deposit > Decimal::ZERO,
                "Minimum deposit can't be zero or less",
            );
            assert!(
                creation_fee_percentage >= Decimal::ZERO && creation_fee_percentage < dec!(100),
                "Creation fee percentage can go from 0 (included) to 100 (excluded)",
            );
            assert!(
                buy_sell_fee_percentage >= Decimal::ZERO && buy_sell_fee_percentage < dec!(100),
                "Buy & sell fee percentage can go from 0 (included) to 100 (excluded)",
            );
            assert!(
                flash_loan_fee_percentage >= Decimal::ZERO && flash_loan_fee_percentage < dec!(100),
                "Flash loan fee percentage can go from 0 (included) to 100 (excluded)",
            );

            // Reserve a ComponentAddress for setting rules on resources
            let (address_reservation, component_address) = Runtime::allocate_component_address(RadixPump::blueprint_id());

            // Create a ResourceManager for minting coin creator badges
            let coin_creator_badge_resource_manager = ResourceBuilder::new_integer_non_fungible_with_registered_type::<CoinCreatorData>(
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

            // Create a ResourceManager for the flash loan transient NFT
            let flash_loan_nft_resource_manager = ResourceBuilder::new_integer_non_fungible_with_registered_type::<FlashLoanData>(
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
                    "name" => TRANSIENT_NFT_NAME, updatable;
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
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(deny_all);
            ))
            .deposit_roles(deposit_roles!(
                depositor => rule!(deny_all);
                depositor_updater => rule!(deny_all);
            ))
            .create_with_no_initial_supply();

            // Instantiate the component
            Self {
                base_coin_address: base_coin_address.clone(),
                minimum_deposit: minimum_deposit,
                coin_creator_badge_resource_manager: coin_creator_badge_resource_manager,
                flash_loan_nft_resource_manager: flash_loan_nft_resource_manager,
                last_coin_creator_badge_id: 0,
                forbidden_symbols: KeyValueStore::new(),
                forbidden_names: KeyValueStore::new(),
                pools: KeyValueStore::new(),
                creation_fee_percentage: creation_fee_percentage,
                buy_sell_fee_percentage: buy_sell_fee_percentage,
                flash_loan_fee_percentage: flash_loan_fee_percentage,
                fee_vault: Vault::new(base_coin_address),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .with_address(address_reservation)
            .globalize()
        }

        // The component owner can prevent users to create coins with well known symbols
        pub fn forbid_symbols(
            &mut self,
            symbols: Vec<String>,
        ) {
            for symbol in symbols.iter() {
                self.forbidden_symbols.insert(symbol.trim().to_uppercase(), 0);
            }
        }

        // The component owner can prevent users to create coins with well known name
        pub fn forbid_names(
            &mut self,
            names: Vec<String>,
        ) {
            for name in names.iter() {
                self.forbidden_names.insert(name.trim().to_uppercase(), 0);
            }
        }

        // A user can create a new coin depositing enough base coins.
        // He gets a fair share of coins and a badge to manage the metadata
        pub fn create_new_coin(
            &mut self,
            mut base_coin_bucket: Bucket,
            mut coin_symbol: String,
            mut coin_name: String,
            coin_icon_url: String,
            coin_description: String,
            coin_supply: Decimal,
            flash_loan_pool_fee_percentage: Decimal,
        ) -> (Bucket, Bucket) {
            assert!(
                base_coin_bucket.resource_address() == self.base_coin_address,
                "Wrong base coin deposited",
            );
            assert!(
                base_coin_bucket.amount() >= self.minimum_deposit,
                "Insufficient base coin deposit",
            );
            assert!(
                coin_supply <= MAX_SUPPLY,
                "Coin supply is too big",
            );
            assert!(
                flash_loan_pool_fee_percentage >= Decimal::ZERO && flash_loan_pool_fee_percentage < dec!(100),
                "Flash loan pool fee percentage can go from 0 (included) to 100 (excluded)",
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

            // Each coin can be managed by one single NFT of the collection, a NonFungibleGlobalId
            // is needed for this
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
            .mint_initial_supply(coin_supply)
            .into();

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    self.creation_fee_percentage * base_coin_bucket.amount() / 100,
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            let coin_creator_badge_bucket = self.coin_creator_badge_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.last_coin_creator_badge_id.into()),
                CoinCreatorData {
                    coin_resource_address: coin_bucket.resource_address(),
                    creation_date: Clock::current_time_rounded_to_seconds(),
                }
            );

            let (pool, coin_creator_coin_bucket) = Pool::new(
                base_coin_bucket, 
                coin_bucket,
                flash_loan_pool_fee_percentage,
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
            flash_loan_fee_percentage: Decimal,
        ) {
            assert!(
                creation_fee_percentage >= Decimal::ZERO && creation_fee_percentage < dec!(100),
                "Creation fee percentage can go from 0 (included) to 100 (excluded)",
            );  
            assert!(
                buy_sell_fee_percentage >= Decimal::ZERO && buy_sell_fee_percentage < dec!(100),
                "Buy & sell fee percentage can go from 0 (included) to 100 (excluded)",
            );
            assert!(
                flash_loan_fee_percentage >= Decimal::ZERO && flash_loan_fee_percentage < dec!(100),
                "Flash loan fee percentage can go from 0 (included) to 100 (excluded)",
            );

            self.creation_fee_percentage = creation_fee_percentage;
            self.buy_sell_fee_percentage = buy_sell_fee_percentage;
            self.flash_loan_fee_percentage = flash_loan_fee_percentage;
        }

        pub fn owner_set_liquidation_mode(
            &mut self,
            coin_address: ResourceAddress,
        ) {
            self.pools.get_mut(&coin_address).expect("Coin not found").set_liquidation_mode();
        }

        pub fn creator_set_liquidation_mode(
            &mut self,
            creator_proof: Proof,
        ) {
            let coin_creator_data = self.get_creator_data(creator_proof);

            self.pools.get_mut(&coin_creator_data.coin_resource_address).unwrap().set_liquidation_mode();
        }

        pub fn get_flash_loan(
            &mut self,
            coin_address: ResourceAddress,
            amount: Decimal
        ) -> (Bucket, Bucket) {
            let (coin_bucket, price) = self.pools.get_mut(&coin_address).expect("Coin not found").get_flash_loan(amount);

            let transient_nft_bucket = self.flash_loan_nft_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(1),
                FlashLoanData {
                    coin_resource_address: coin_address,
                    coin_amount: amount,
                    price: price,
                }
            );

            (coin_bucket, transient_nft_bucket)
        }

        pub fn return_flash_loan(
            &mut self,
            transient_nft_bucket: Bucket,
            mut base_coin_bucket: Bucket,
            coin_bucket: Bucket,
        ) {
            assert!(
                transient_nft_bucket.resource_address() == self.flash_loan_nft_resource_manager.address(),
                "Wrong NFT",
            );
            assert!(
                base_coin_bucket.resource_address() == self.base_coin_address,
                "Wrong base coin",
            );

            let flash_loan_data = transient_nft_bucket.as_non_fungible().non_fungible::<FlashLoanData>().data();
            assert!(
                flash_loan_data.coin_resource_address == coin_bucket.resource_address(),
                "Wrong coin",
            );
            assert!(
                flash_loan_data.coin_amount <= coin_bucket.amount(),
                "Not enough coins",
            );

            transient_nft_bucket.burn();

            let mut pool = self.pools.get_mut(&coin_bucket.resource_address()).unwrap();

            // In order to avoid price manipulation affecting the fees, consider the maximum among the
            // price at the moment the flash loan was granted and the current price.
            let (_, _, mut price, _, _) = pool.get_pool_info();
            price = max(price, flash_loan_data.price);

            self.fee_vault.put(
                base_coin_bucket.take_advanced(
                    flash_loan_data.coin_amount * price * self.flash_loan_fee_percentage / dec!(100),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            );

            pool.return_flash_loan(
                base_coin_bucket,
                coin_bucket,
                price,
            );
        }

        pub fn update_flash_loan_pool_fee_percentage(
            &mut self,
            creator_proof: Proof,
            flash_loan_pool_fee_percentage: Decimal,
        ) {
            let coin_creator_data = self.get_creator_data(creator_proof);

            self.pools.get_mut(&coin_creator_data.coin_resource_address).unwrap()
            .update_flash_loan_pool_fee_percentage(flash_loan_pool_fee_percentage);
        }

        pub fn get_pool_info(
            &self,
            coin_address: ResourceAddress,
        ) -> (Decimal, Decimal, Decimal, Decimal, Decimal, PoolMode, ResourceAddress) {
            let (
                base_coin_amount,
                coin_amount,
                last_price,
                flash_loan_pool_fee_percentage,
                pool_mode,
            ) = self.pools.get(&coin_address).expect("Coin not found").get_pool_info();

            (
                base_coin_amount,
                coin_amount,
                last_price,
                self.buy_sell_fee_percentage,
                flash_loan_pool_fee_percentage + self.flash_loan_fee_percentage * (100 + flash_loan_pool_fee_percentage) / dec!(100),
                pool_mode,
                self.flash_loan_nft_resource_manager.address(),
            )
        }

        fn get_creator_data(
            &self,
            creator_proof: Proof
        ) -> CoinCreatorData {
            creator_proof.check_with_message(
                self.coin_creator_badge_resource_manager.address(),
                "Wrong badge",
            )
            .as_non_fungible()
            .non_fungible::<CoinCreatorData>()
            .data()
        }

    }
}
