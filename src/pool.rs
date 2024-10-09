use scrypto::prelude::*;
use scrypto_math::*;
use scrypto::prelude::rust::cmp::min;

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
    buy_pool_fee_percentage: Decimal,
    sell_pool_fee_percentage: Decimal,
    flash_loan_pool_fee_percentage: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct BuyEvent {
    resource_address: ResourceAddress,
    amount: Decimal,
    price: Decimal,
    coins_in_pool: Decimal,
    fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct SellEvent {
    resource_address: ResourceAddress,
    amount: Decimal,
    price: Decimal,
    coins_in_pool: Decimal,
    mode: PoolMode,
    fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct LiquidationEvent {
    resource_address: ResourceAddress,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FlashLoanEvent {
    resource_address: ResourceAddress,
    amount: Decimal,
    fee_paid_to_the_pool: Decimal,
}

#[derive(ScryptoSbor)]
pub struct Pool {
    base_coin_vault: Vault,
    coin_vault: Vault,
    mode: PoolMode,
    last_price: Decimal,
    buy_pool_fee_percentage: Decimal,
    sell_pool_fee_percentage: Decimal,
    flash_loan_pool_fee_percentage: Decimal,
}

impl Pool {

    // This function instantiates a new Pool and simulates a bought so that the creater gets new coins
    // at about the same price as early birds will do. Quite fair launch.
    pub fn new(
        base_coin_bucket: Bucket,
        mut coin_bucket: Bucket,
        buy_pool_fee_percentage: Decimal,
        sell_pool_fee_percentage: Decimal,
        flash_loan_pool_fee_percentage: Decimal,
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

        let price = base_coin_bucket.amount() / coin_amount_bought;

        Runtime::emit_event(
            NewCoinEvent {
                resource_address: coin_bucket.resource_address(),
                price: price,
                coins_in_pool: coin_bucket.amount(),
                creator_allocation: coin_amount_bought,
                buy_pool_fee_percentage: buy_pool_fee_percentage,
                sell_pool_fee_percentage: sell_pool_fee_percentage,
                flash_loan_pool_fee_percentage: flash_loan_pool_fee_percentage,
            }
        );

        let pool = Pool {
            base_coin_vault: Vault::with_bucket(base_coin_bucket),
            coin_vault: Vault::with_bucket(coin_bucket),
            mode: PoolMode::Normal,
            last_price: price,
            buy_pool_fee_percentage: buy_pool_fee_percentage,
            sell_pool_fee_percentage: sell_pool_fee_percentage,
            flash_loan_pool_fee_percentage: flash_loan_pool_fee_percentage,
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
        mut base_coin_bucket: Bucket,
        max_buy_sell_pool_fee_percentage: Decimal,
    ) -> Bucket {
        assert!(
            self.mode == PoolMode::Normal,
            "You can't buy a coin in liquidation mode",
        );

        // Check if the component owner lowered max_buy_sell_pool_fee_percentage after the creator 
        // set a higher buy_sell_pool_fee_percentage
        let fee_percentage = min(max_buy_sell_pool_fee_percentage, self.buy_pool_fee_percentage);

        let fee_bucket = base_coin_bucket.take_advanced(
            base_coin_bucket.amount() * fee_percentage / dec!(100),
            WithdrawStrategy::Rounded(RoundingMode::ToZero),
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

        self.last_price = base_coin_bucket.amount() / coin_amount_bought;

        Runtime::emit_event(
            BuyEvent {
                resource_address: self.coin_vault.resource_address(),
                amount: coin_amount_bought,
                price: self.last_price,
                coins_in_pool: new_coin_amount,
                fee_paid_to_the_pool: fee_bucket.amount(),
            }
        );

        self.base_coin_vault.put(base_coin_bucket);
        self.base_coin_vault.put(fee_bucket);
        self.coin_vault.take(coin_amount_bought)
    }

    pub fn sell(
        &mut self,
        coin_bucket: Bucket,
        max_buy_sell_pool_fee_percentage: Decimal,
    ) -> Bucket {

        let (base_coin_bucket, fee_amount) = match self.mode {
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

                let mut base_coin_bucket = self.base_coin_vault.take_advanced(
                    self.base_coin_vault.amount() - new_base_coin_amount,
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                );

                // Check if the component owner lowered max_buy_sell_pool_fee_percentage after the creator 
                // set a higher buy_sell_pool_fee_percentage
                let fee_percentage = min(max_buy_sell_pool_fee_percentage, self.sell_pool_fee_percentage);

                let fee_bucket = base_coin_bucket.take_advanced(
                    base_coin_bucket.amount() * fee_percentage / dec!(100),
                    WithdrawStrategy::Rounded(RoundingMode::ToZero),
                );
                let fee_amount = fee_bucket.amount();
                self.base_coin_vault.put(fee_bucket);

                (base_coin_bucket, fee_amount)
            },
            PoolMode::Liquidation => {
                let coin_supply = ResourceManager::from_address(
                    self.coin_vault.resource_address()
                ).total_supply().unwrap();

                let user_share = coin_bucket.amount() / (coin_supply - self.coin_vault.amount());

                (
                    self.base_coin_vault.take_advanced(
                        self.base_coin_vault.amount() * user_share,
                        WithdrawStrategy::Rounded(RoundingMode::ToZero),
                    ),
                    Decimal::ZERO,
                )
            },
        };

        self.last_price = base_coin_bucket.amount() / coin_bucket.amount();

        Runtime::emit_event(
            SellEvent {
                resource_address: self.coin_vault.resource_address(),
                amount: coin_bucket.amount(),
                price: self.last_price,
                coins_in_pool: self.coin_vault.amount() + coin_bucket.amount(),
                mode: self.mode,
                fee_paid_to_the_pool: fee_amount,
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

    pub fn get_flash_loan(
        &mut self,
        amount: Decimal,
    ) -> (Bucket, Decimal) {
        assert!(
            self.mode == PoolMode::Normal,
            "You can't get a flash loan in liquidation mode",
        );

        (self.coin_vault.take(amount), self.last_price)
    }

    pub fn return_flash_loan(
        &mut self,
        base_coin_bucket: Bucket,
        coin_bucket: Bucket,
        price: Decimal,
        max_flash_loan_pool_fee_percentage: Decimal,
    ) {
        // Check if the component owner lowered max_flash_loan_pool_fee_percentage after the creator 
        // set a higher falsh_loan_pool_fee_percentage
        let fee_percentage = min(max_flash_loan_pool_fee_percentage, self.flash_loan_pool_fee_percentage);
        assert!(
            base_coin_bucket.amount() >= coin_bucket.amount() * price * fee_percentage / dec!(100),
            "Insufficient fee paid to the pool",
        );

        Runtime::emit_event(
            FlashLoanEvent {
                resource_address: coin_bucket.resource_address(),
                amount: coin_bucket.amount(),
                fee_paid_to_the_pool: base_coin_bucket.amount(),
            }
        );

        self.base_coin_vault.put(base_coin_bucket);
        self.coin_vault.put(coin_bucket);
    }

    pub fn get_pool_info(&self) -> (Decimal, Decimal, Decimal, Decimal, Decimal, Decimal, PoolMode) {
        (
            self.base_coin_vault.amount(),
            self.coin_vault.amount(),
            self.last_price,
            self.buy_pool_fee_percentage,
            self.sell_pool_fee_percentage,
            self.flash_loan_pool_fee_percentage,
            self.mode,
        )
    }

    pub fn update_pool_fee_percentage(
        &mut self,
        buy_pool_fee_percentage: Decimal,
        sell_pool_fee_percentage: Decimal,
        flash_loan_pool_fee_percentage: Decimal,
    ) {
        self.buy_pool_fee_percentage = buy_pool_fee_percentage;
        self.sell_pool_fee_percentage = sell_pool_fee_percentage;
        self.flash_loan_pool_fee_percentage = flash_loan_pool_fee_percentage;
    }
}
