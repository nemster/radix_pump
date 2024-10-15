use scrypto::prelude::*;
use random::Random;

#[blueprint]
mod random_component {
    struct RandomComponent {
    }
    impl RandomComponent {

    }
}

pub struct RandomHelper {
    random_component: Global<AnyComponent>,
    random: Option<Random>,
    resource_manager: ResourceManager,
}

impl RandomHelper {
    pub fn new(
        random_component: Global<AnyComponent>,
    ) -> RandomHelper {
        let resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_roles(mint_roles!(
            minter => rule!(require(global_caller(Runtime::global_address())));
            minter_updater => rule!(deny_all);
        ))
        .burn_roles(burn_roles!(
            burner => rule!(require(global_caller(Runtime::global_address())));
            burner_updater => rule!(deny_all);
        ))
        .divisibility(0);

        Self {
            random_component: random_component,
            random: None,
            resource_manager: resource_manager,
        }
    }

    pub fn init(
        &self,
        key: u32,
    ) {
        self.random_component.request_random()(
            Runtime::global_component().address(),
            "init_random",
            "on_error",
            key,
            self.resource_manager.mint(1),
            0, //TODO: try to guess it
        );
    }
}
