use scrypto::prelude::*;

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct NewCoinEvent {
    resource_address: ResourceAddress,
    price: Decimal,
    coins_in_pool: Decimal,
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
}

#[derive(ScryptoSbor)]
pub struct Pool {
    base_coin_vault: Vault,
    coin_vault: Vault,
}

impl Pool {

    pub fn new(
        base_coin_bucket: Bucket,
        coin_bucket: Bucket,
    ) -> Pool {
        Runtime::emit_event(
            NewCoinEvent {
                resource_address: coin_bucket.resource_address(),
                price: base_coin_bucket.amount() / coin_bucket.amount(),
                coins_in_pool: coin_bucket.amount(),
            }
        );

        Pool {
            base_coin_vault: Vault::with_bucket(base_coin_bucket),
            coin_vault: Vault::with_bucket(coin_bucket),
        }
    }

    pub fn buy(
        &mut self,
        base_coin_bucket: Bucket,
    ) -> Bucket {
        let base_coin_amount = PreciseDecimal::from(self.base_coin_vault.amount());
        let coin_amount = PreciseDecimal::from(self.coin_vault.amount());
        let constant_product = coin_amount * base_coin_amount;
        let new_coin_amount = constant_product / (base_coin_amount + PreciseDecimal::from(base_coin_bucket.amount()));
        let coin_amount_bought = (coin_amount - new_coin_amount).checked_truncate(RoundingMode::ToZero).unwrap();

        self.base_coin_vault.put(base_coin_bucket);
        let coin_bucket = self.coin_vault.take(coin_amount_bought);

        Runtime::emit_event(
            BuyEvent {
                resource_address: self.coin_vault.resource_address(),
                amount: coin_amount_bought,
                price: self.base_coin_vault.amount() / self.coin_vault.amount(),
                coins_in_pool: self.coin_vault.amount(),
            }
        );

        coin_bucket
    }

    pub fn sell(
        &mut self,
        coin_bucket: Bucket,
    ) -> Bucket {
        let base_coin_amount = PreciseDecimal::from(self.base_coin_vault.amount());
        let coin_amount = PreciseDecimal::from(self.coin_vault.amount());
        let coin_sold_amount = coin_bucket.amount();
        let constant_product = coin_amount * base_coin_amount;
        let new_base_coin_amount = constant_product / (coin_amount + PreciseDecimal::from(coin_sold_amount));
        let base_coin_amount_bought = (base_coin_amount - new_base_coin_amount).checked_truncate(RoundingMode::ToZero).unwrap();

        self.coin_vault.put(coin_bucket);
        let base_coin_bucket = self.base_coin_vault.take_advanced(
            base_coin_amount_bought,
            WithdrawStrategy::Rounded(RoundingMode::ToZero),
        );

        Runtime::emit_event(
            SellEvent {
                resource_address: self.coin_vault.resource_address(),
                amount: coin_sold_amount,
                price: self.base_coin_vault.amount() / self.coin_vault.amount(),
                coins_in_pool: self.coin_vault.amount(),
            }
        );

        base_coin_bucket
    }
}
