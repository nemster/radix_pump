use scrypto_test::prelude::*;

use radix_pump::radix_pump::radix_pump_test::*;

#[test]
fn test_radix_pump() -> Result<(), RuntimeError> {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)?;
    let badge_address = badge_bucket.resource_address(&mut env)?;

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env)?;
    let base_coin_address = base_coin_bucket1.resource_address(&mut env)?;

    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!(1000000),
        dec!(100),
        dec!(1),
        dec!("0.3"),
        package_address,
        &mut env
    )?;

    radix_pump.forbid_symbols(vec!["XRD".to_string()], &mut env)?;
    radix_pump.forbid_names(vec!["Radix".to_string()], &mut env)?;

    let (_coin_creator_badge_bucket, coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(dec!(100), &mut env)?,
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        &mut env
    )?;

    let coin_bucket2 = radix_pump.buy(
        coin_bucket1.resource_address(&mut env)?,
        base_coin_bucket1.take(dec!(100), &mut env)?,
        &mut env
    )?;

    let _base_coin_bucket2 = radix_pump.sell(
        coin_bucket2,
        &mut env
    )?;

    Ok(())
}
