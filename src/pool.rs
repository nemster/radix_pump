use scrypto::prelude::*;
use scrypto_math::*;

#[derive(Debug, ScryptoSbor, PartialEq, Clone, Copy)]
pub enum PoolMode {
    Normal,
    Liquidation,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct NewCoinEvent {
    resource_address: ResourceAddress,
    price: Decimal,
    coins_in_pool: Decimal,
    creator_allocation: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct BuyEvent {
    resource_address: ResourceAddress,
    amount: Decimal,
    price: Decimal,
    coins_in_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct SellEvent {
    resource_address: ResourceAddress,
    amount: Decimal,
    price: Decimal,
    coins_in_pool: Decimal,
    mode: PoolMode,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct LiquidationEvent {
    resource_address: ResourceAddress,
}

#[derive(ScryptoSbor)]
pub struct Pool {
    base_coin_vault: Vault,
    coin_vault: Vault,
    mode: PoolMode,
}

impl Pool {

    // This function instantiates a new Pool and simulates a bought so that the creater gets new coins
    // at about the same price as early birds will do. Quite fair launch.
    pub fn new(
        base_coin_bucket: Bucket,
        mut coin_bucket: Bucket,
    ) -> (Pool, Bucket) {
        let base_coin_bucket_amount = PreciseDecimal::from(base_coin_bucket.amount());
        let coin_bucket_amount = PreciseDecimal::from(coin_bucket.amount());

        let (constant_product, exponent) = Pool::compute_costant_product(
            base_coin_bucket_amount,
            coin_bucket_amount,
        );

        let new_coin_amount =
            constant_product /
            (pdec!(2) * base_coin_bucket_amount).pow(exponent).unwrap()
        ;
        let coin_amount_bought = (coin_bucket_amount - new_coin_amount)
        .checked_truncate(RoundingMode::ToZero)
        .unwrap();
        let creator_coin_bucket = coin_bucket.take(coin_amount_bought);

        Runtime::emit_event(
            NewCoinEvent {
                resource_address: coin_bucket.resource_address(),
                price: base_coin_bucket.amount() / coin_amount_bought,
                coins_in_pool: coin_bucket.amount(),
                creator_allocation: coin_amount_bought,
            }
        );

        let pool = Pool {
            base_coin_vault: Vault::with_bucket(base_coin_bucket),
            coin_vault: Vault::with_bucket(coin_bucket),
            mode: PoolMode::Normal,
        };

        (pool, creator_coin_bucket)
    }

    // When a pool is created it contains almost the enire supply of coin and just a little deposit
    // of base coins. In this situation the constant product formula would allow a user to get a
    // large part of the coin supply for only a few base coins. To prevent this issue, an exponent
    // that modifies the weight of the two terms of the product is added. The exponent value
    // depends on how unbalanced the pool is.
    // Pow(exponent) is always safe because the exponent is in the 0.2 to 1 range.
    fn compute_costant_product(
        base_coin_amount: PreciseDecimal,
        coin_amount: PreciseDecimal,
    ) -> (PreciseDecimal, PreciseDecimal) {
        let mut exponent = PreciseDecimal::ONE;
        if base_coin_amount < coin_amount {
            exponent = pdec!("0.2") + pdec!("0.8") * base_coin_amount / coin_amount;
        }

        // Multiplication is safe because both numbers fit in a Decimal but are PreciseDecimals
        let constant_product = base_coin_amount.pow(exponent).unwrap() * coin_amount;

        (constant_product, exponent)
    }

    pub fn buy(
        &mut self,
        base_coin_bucket: Bucket,
    ) -> Bucket {
        assert!(
            self.mode == PoolMode::Normal,
            "You can't buy a coin in liquidation mode",
        );

        let (constant_product, exponent) = Pool::compute_costant_product(
            PreciseDecimal::from(self.base_coin_vault.amount()),
            PreciseDecimal::from(self.coin_vault.amount()),
        );

        let new_coin_amount = (
            constant_product /
            PreciseDecimal::from(self.base_coin_vault.amount() + base_coin_bucket.amount()).pow(exponent).unwrap()
        )
        .checked_truncate(RoundingMode::ToZero)
        .unwrap();
        let coin_amount_bought = self.coin_vault.amount() - new_coin_amount;

        Runtime::emit_event(
            BuyEvent {
                resource_address: self.coin_vault.resource_address(),
                amount: coin_amount_bought,
                price: base_coin_bucket.amount() / coin_amount_bought,
                coins_in_pool: new_coin_amount,
            }
        );

        self.base_coin_vault.put(base_coin_bucket);

        self.coin_vault.take(coin_amount_bought)
    }

    pub fn sell(
        &mut self,
        coin_bucket: Bucket,
    ) -> Bucket {

        let base_coin_bucket = match self.mode {
            PoolMode::Normal => {
                let (constant_product, exponent) = Pool::compute_costant_product(
                    PreciseDecimal::from(self.base_coin_vault.amount()),
                    PreciseDecimal::from(self.coin_vault.amount()),
                );

                let new_base_coin_amount = (constant_product / PreciseDecimal::from(coin_bucket.amount() + self.coin_vault.amount()))
                // This number is smaller than base_coin_amount.pow(exponent) so it's safe to do .pow(PreciseDecimal::ONE / exponent)
                .pow(PreciseDecimal::ONE / exponent)
                .unwrap()
                .checked_truncate(RoundingMode::ToZero)
                .unwrap();

                self.base_coin_vault.take_advanced(
                    self.base_coin_vault.amount() - new_base_coin_amount,
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            },
            PoolMode::Liquidation => {
                let coin_supply = ResourceManager::from_address(
                    self.coin_vault.resource_address()
                ).total_supply().unwrap();

                let user_share = coin_bucket.amount() / (coin_supply - self.coin_vault.amount());

                self.base_coin_vault.take_advanced(
                    self.base_coin_vault.amount() * user_share,
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                )
            }
        };

        Runtime::emit_event(
            SellEvent {
                resource_address: self.coin_vault.resource_address(),
                amount: coin_bucket.amount(),
                price: base_coin_bucket.amount() / coin_bucket.amount(),
                coins_in_pool: self.coin_vault.amount() + coin_bucket.amount(),
                mode: self.mode,
            }
        );

        self.coin_vault.put(coin_bucket);

        base_coin_bucket
    }

    pub fn set_liquidation_mode(&mut self) {
        assert!(
            self.mode == PoolMode::Normal,
            "Already in Liquidation mode",
        );

        Runtime::emit_event(
            LiquidationEvent {
                resource_address: self.coin_vault.resource_address(),
            }
        );

        self.mode = PoolMode::Liquidation;
    }
}
