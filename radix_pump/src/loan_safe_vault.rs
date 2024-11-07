use scrypto::prelude::*;

/* This struct is just a wrapper around the Scrypto Vault component

   Many Pool methods use the amount() method of the Vaults for their calculations.
   This value may be temporarily altered due to flash loans making all of the calculations wrong.
   This is why this struct exists: the amount() method always returns the amount of coins that is suppesed to be in the
   Pool, as if there was no ongoing flash loan.
   To make it work, an additional get_loan() method is defined: it works just like take() but takes note of the lent
   amount.
   The specialized version of put() exists too: return_loan() expects to receive back at least the same amount was
   taken with get_loan() and takes note of the refund.
   It is not possible to take more than one loan at the same time.
*/

#[derive(ScryptoSbor)]
pub struct LoanSafeVault {
    vault: Vault,
    ongoing_loan: Decimal,
}

impl LoanSafeVault {

    // Instatiate empty Vault
    pub fn new(resource_address: ResourceAddress) -> LoanSafeVault {
        Self {
            vault: Vault::new(resource_address),
            ongoing_loan: Decimal::ZERO,
        }
    }

    // Instatiate Vault containing some coins
    pub fn with_bucket(bucket: Bucket) -> LoanSafeVault {
        Self {
            vault: Vault::with_bucket(bucket),
            ongoing_loan: Decimal::ZERO,
        }
    }

    // Get the resource address of the coins in the Vault
    pub fn resource_address(&self) -> ResourceAddress {
        self.vault.resource_address()
    }

    // Get the amount of the coins in the Vault as if there was no ongoing flash loan
    pub fn amount(&self) -> Decimal {
        self.vault.amount() + self.ongoing_loan
    }

    // Put coins in the Vault
    pub fn put(&mut self, bucket: Bucket) {
        self.vault.put(bucket);
    }

    // Take coins from the Vault
    pub fn take(&mut self, amount: Decimal) -> Bucket {
        self.vault.take(amount)
    }

    // Take coins from the Vault and take note of the taken amount
    pub fn get_loan(&mut self, amount: Decimal) -> Bucket {

        // Multiple loans not allowed
        assert!(
            self.ongoing_loan == Decimal::ZERO,
            "There's already an ongoing loan",
        );

        self.ongoing_loan = amount;

        self.vault.take(amount)
    }

    // Take put coins in the Vault and make sure they match the ongoing loan (more is ok)
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
