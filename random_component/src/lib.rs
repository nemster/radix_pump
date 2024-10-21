use scrypto::prelude::*;

#[derive(ScryptoSbor)]
struct Callback {
    address: ComponentAddress,
    method_name: String,
    key: u32,
    badge: Vault,
}

#[blueprint]
mod random_component {
    struct RandomComponent {
        last_inserted_callback: u32,
        last_executed_callback: u32,
        callbacks: KeyValueStore<u32, Callback>,
    }

    impl RandomComponent {
        pub fn new() -> Global<RandomComponent> {
            Self {
                last_inserted_callback: 0,
                last_executed_callback: 0,
                callbacks: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn request_random(
            &mut self,
            address: ComponentAddress,
            method_name: String,
            _on_error: String,
            key: u32,
            badge_opt: Option<FungibleBucket>,
            _expected_fee: u8
        ) -> u32 {
            self.last_inserted_callback += 1;

            self.callbacks.insert(
                self.last_inserted_callback,
                Callback {
                    address: address,
                    method_name: method_name,
                    key: key,
                    badge: Vault::with_bucket(badge_opt.unwrap().into()),
                },
            );

            self.last_inserted_callback
        }

        pub fn do_callback(&mut self, random_number: u64) {
            if self.last_inserted_callback > self.last_executed_callback {
                self.last_executed_callback += 1;

                let mut callback = self.callbacks.get_mut(&self.last_executed_callback).unwrap();

                let badge_bucket = callback.badge.take(dec!(1));

                let comp: Global<AnyComponent> = Global::from(callback.address);

                let seed: Vec<u8> = vec![
                    (random_number >> 56) as u8,
                    ((random_number >> 48) & 0xff) as u8,
                    ((random_number >> 40) & 0xff) as u8,
                    ((random_number >> 32) & 0xff)as u8,
                    ((random_number >> 24) & 0xff)as u8,
                    ((random_number >> 16) & 0xff) as u8,
                    ((random_number >> 8) & 0xff) as u8,
                    (random_number & 0xff) as u8
                ];

                comp.call_ignore_rtn(callback.method_name.as_str(), &(callback.key, badge_bucket, seed));
            }
        }
    }
}
