use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct LoanSafeVault {
    vault: Vault,
    ongoing_loan: Decimal,
}

impl LoanSafeVault {

    pub fn new(resource_address: ResourceAddress) -> LoanSafeVault {
        Self {
            vault: Vault::new(resource_address),
            ongoing_loan: Decimal::ZERO,
        }
    }

    pub fn with_bucket(bucket: Bucket) -> LoanSafeVault {
        Self {
            vault: Vault::with_bucket(bucket),
            ongoing_loan: Decimal::ZERO,
        }
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.vault.resource_address()
    }

    pub fn amount(&self) -> Decimal {
        self.vault.amount() + self.ongoing_loan
    }

    pub fn put(&mut self, bucket: Bucket) {
        self.vault.put(bucket);
    }

    pub fn take(&mut self, amount: Decimal) -> Bucket {
        self.vault.take(amount)
    }

    pub fn get_loan(&mut self, amount: Decimal) -> Bucket {
        assert!(
            self.ongoing_loan == Decimal::ZERO,
            "There's already an ongoing loan",
        );

        self.ongoing_loan = amount;

        self.vault.take(amount)
    }

    pub fn return_loan(&mut self, bucket: Bucket) {
        assert!(
            self.ongoing_loan != Decimal::ZERO,
            "There's no ongoing loan",
        );
        assert!(
            self.ongoing_loan <= bucket.amount(),
            "Insufficient loan refund",
        );

        self.ongoing_loan = Decimal::ZERO;

        self.vault.put(bucket);
    }
}
