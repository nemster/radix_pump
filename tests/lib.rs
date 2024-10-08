use scrypto_test::prelude::*;

use radix_pump::radix_pump::radix_pump_test::*;

#[test]
#[should_panic]
fn test_insufficient_deposit() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)
        .unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env)
        .unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let minimum_deposit = dec!(100);
    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        minimum_deposit,
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let (_coin_creator_badge_bucket, _coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(minimum_deposit - dec!(1), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_empty_name() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)
        .unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env)
        .unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let minimum_deposit = dec!(100);
    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        minimum_deposit,
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let (_coin_creator_badge_bucket, _coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(minimum_deposit, &mut env).unwrap(),
        "COIN".to_string(),
        "    ".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "A test coin with only spaces in the name".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_empty_symbol() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)
        .unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env)
        .unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let minimum_deposit = dec!(100);
    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        minimum_deposit,
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let (_coin_creator_badge_bucket, _coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(minimum_deposit, &mut env).unwrap(),
        "  ".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "A test coin wih only spaces in the symbol".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_same_symbol() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)
        .unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env)
        .unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let minimum_deposit = dec!(100);
    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        minimum_deposit,
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let (_coin_creator_badge_bucket, _coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(minimum_deposit, &mut env).unwrap(),
        " CoiN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();

    let (_coin_creator_badge2_bucket, _coin_bucket2) = radix_pump.create_new_coin(
        base_coin_bucket1.take(minimum_deposit, &mut env).unwrap(),
        "COIn ".to_string(),
        "AnotherCoin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Another coin with a very similar symbol".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_same_name() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)
        .unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env)
        .unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let minimum_deposit = dec!(100);
    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        minimum_deposit,
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let (_coin_creator_badge_bucket, _coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(minimum_deposit, &mut env).unwrap(),
        "COIN".to_string(),
        "Coin ".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();

    let (_coin_creator_badge2_bucket, _coin_bucket2) = radix_pump.create_new_coin(
        base_coin_bucket1.take(minimum_deposit, &mut env).unwrap(),
        "COIN2".to_string(),
        " COIN ".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Another coin with a very similar name".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_forbid_symbols() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)
        .unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env)
        .unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    radix_pump.forbid_symbols(vec!["XRD".to_string()], &mut env).unwrap();

    let (_coin_creator_badge_bucket, _coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "xrd".to_string(),
        "Radix".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "A coin with the same name and symbol as XRD".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_forbid_names() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)
        .unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env)
        .unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    radix_pump.forbid_names(vec!["radix".to_string()], &mut env).unwrap();

    let (_coin_creator_badge_bucket, _coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "xrd".to_string(),
        " Radix ".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "A coin with the same name and symbol as XRD".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
}

#[test]
fn test_buy() -> Result<(), RuntimeError> {
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
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
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
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;
    let coin_address = coin_bucket1.resource_address(&mut env)?;

    let coin_bucket2 = radix_pump.buy(
        coin_address,
        base_coin_bucket1.take(dec!(100), &mut env)?,
        &mut env
    )?;
    assert!(
        coin_bucket2.resource_address(&mut env)? == coin_address,
        "Wrong coin received",
    );

    let coin_bucket3 = radix_pump.buy(
        coin_address,
        base_coin_bucket1.take(dec!(100), &mut env)?,
        &mut env
    )?;
    assert!(
        coin_bucket3.amount(&mut env)? < coin_bucket2.amount(&mut env)?,
        "Price not increasing when buying coins",
    );

    Ok(())
}

#[test]
#[should_panic]
fn test_buy_wrong_coin() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)
        .unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env)
        .unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let (_coin_creator_badge_bucket, coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();

    let _bucket = radix_pump.buy(
        base_coin_address,
        coin_bucket1.take(dec!(100), &mut env).unwrap(),
        &mut env
    ).unwrap();
}

#[test]
fn test_sell() -> Result<(), RuntimeError> {
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
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
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
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;

    let base_coin_bucket2 = radix_pump.sell(
        coin_bucket1.take(dec!(100), &mut env)?,
        &mut env
    )?;
    assert!(
        base_coin_bucket2.resource_address(&mut env)? == base_coin_address,
        "Wrong coin received",
    );

    let base_coin_bucket3 = radix_pump.sell(
        coin_bucket1.take(dec!(100), &mut env)?,
        &mut env
    )?;
    assert!(
        base_coin_bucket3.amount(&mut env)? < base_coin_bucket2.amount(&mut env)?,
        "Price not decreasing when selling coins",
    );

    Ok(())
}

#[test]
#[should_panic]
fn test_sell_wrong_coin() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)
        .unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env)
        .unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let (_coin_creator_badge_bucket, _coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();

    let _bucket = radix_pump.sell(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        &mut env
    ).unwrap();
}

#[test]
fn test_fees() -> Result<(), RuntimeError> {
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

    let mut creation_fee_percentage = dec!(1);
    let mut buy_sell_fee_percentage = dec!(0.3);
    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        creation_fee_percentage,
        buy_sell_fee_percentage,
        dec!("0.1"),
        package_address,
        &mut env
    )?;

    let deposit_amount = dec!(1000);
    let (_coin_creator_badge_bucket, coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(deposit_amount, &mut env)?,
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;
    let coin_address = coin_bucket1.resource_address(&mut env)?;

    let base_coin_bucket2 = radix_pump.get_fees(&mut env)?;
    assert!(
        base_coin_bucket2.resource_address(&mut env)? == base_coin_address,
        "Wrong coin received",
    );
    panic_if_significantly_different(
        base_coin_bucket2.amount(&mut env)?,
        deposit_amount * creation_fee_percentage / dec!(100),
        "Wrong amount received on get_fees 1",
    );

    let base_coin_buy = dec!(100);
    let _coin_bucket2 = radix_pump.buy(
        coin_address,
        base_coin_bucket1.take(base_coin_buy, &mut env)?,
        &mut env
    )?;

    let base_coin_bucket3 = radix_pump.get_fees(&mut env)?;
    assert!(
        base_coin_bucket3.resource_address(&mut env)? == base_coin_address,
        "Wrong coin received",
    );
    panic_if_significantly_different(
        base_coin_bucket3.amount(&mut env)?,
        base_coin_buy * buy_sell_fee_percentage / dec!(100),
        "Wrong amount received on get_fees 2",
    );

    creation_fee_percentage = dec!(2);
    buy_sell_fee_percentage = dec!(0.5);
    radix_pump.update_fees(
        creation_fee_percentage,
        buy_sell_fee_percentage,
        dec!("0.1"),
        dec!(100),
        dec!(100),
        &mut env
    )?;

    let (_coin_creator_badge_bucket, coin2_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(deposit_amount, &mut env)?,
        "C2".to_string(),
        "Coin 2".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just another test coin".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;

    let base_coin_bucket4 = radix_pump.sell(
        coin2_bucket1.take(dec!(100), &mut env)?,
        &mut env
    )?;
    let base_coin_bucket4_amount = base_coin_bucket4.amount(&mut env)?;

    let base_coin_bucket5 = radix_pump.get_fees(&mut env)?;
    assert!(
        base_coin_bucket5.resource_address(&mut env)? == base_coin_address,
        "Wrong coin received",
    );
    panic_if_significantly_different(
        base_coin_bucket5.amount(&mut env)?,
        deposit_amount * creation_fee_percentage / dec!(100) + base_coin_bucket4_amount * buy_sell_fee_percentage / (dec!(100) - buy_sell_fee_percentage),
        "Wrong amount received on get_fees 3",
    );

    Ok(())
}

#[test]
fn test_liquidation() -> Result<(), RuntimeError> {
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

    let creation_fee_percentage = dec!(1);
    let buy_sell_fee_percentage = dec!(0.3);
    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        creation_fee_percentage,
        buy_sell_fee_percentage,
        dec!("0.1"),
        package_address,
        &mut env
    )?;

    let deposit_amount = dec!(1000);
    let (_coin_creator_badge_bucket, coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(deposit_amount, &mut env)?,
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;
    let coin_address = coin_bucket1.resource_address(&mut env)?;
    let base_coins_in_pool = deposit_amount * (dec!(1) - creation_fee_percentage / dec!(100));
    let coins_out_of_the_pool = coin_bucket1.amount(&mut env)?;

    radix_pump.owner_set_liquidation_mode(coin_address, &mut env)?;

    let mut coin_sold = dec!(100);
    let base_coin_bucket2 = radix_pump.sell(
        coin_bucket1.take(coin_sold, &mut env)?,
        &mut env
    )?;
    panic_if_significantly_different(
        base_coin_bucket2.amount(&mut env)?,
        (100 - buy_sell_fee_percentage ) * base_coins_in_pool * coin_sold / (100 * coins_out_of_the_pool),
        "Wrong amount received on sale 1"
    );

    coin_sold = dec!(500);
    let base_coin_bucket3 = radix_pump.sell(
        coin_bucket1.take(coin_sold, &mut env)?,
        &mut env
    )?;
    panic_if_significantly_different(
        base_coin_bucket3.amount(&mut env)?,
        (100 - buy_sell_fee_percentage ) * base_coins_in_pool * coin_sold / (100 * coins_out_of_the_pool),
        "Wrong amount received on sale 2"
    );

    Ok(())
}

#[test]
#[should_panic]
fn test_buy_liquidation() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)
        .unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env)
        .unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!(1),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let (_coin_creator_badge_bucket, coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        dec!(1000000),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
    let coin_address = coin_bucket1.resource_address(&mut env).unwrap();

    let _ = radix_pump.owner_set_liquidation_mode(coin_address, &mut env);

    let _coin_bucket2 = radix_pump.buy(
        coin_address,
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        &mut env
    ).unwrap();
}

#[test]
fn test_flash_loan() -> Result<(), RuntimeError> {
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

    let flash_loan_fee_percentage = dec!("0.1");
    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!(1),
        dec!("0.3"),
        flash_loan_fee_percentage,
        package_address,
        &mut env
    )?;

    let flash_loan_pool_fee_percentage = dec!("0.1");
    let (_coin_creator_badge_bucket, coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(dec!(100), &mut env)?,
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        dec!(1000000),
        dec!(0.1),
        flash_loan_pool_fee_percentage,
        &mut env
    )?;
    let coin_address = coin_bucket1.resource_address(&mut env)?;

    let (_, _, price, _, _, flash_loan_total_fee_percentage, _, _) = radix_pump.get_pool_info(coin_address, &mut env)?;
    panic_if_significantly_different(
        flash_loan_total_fee_percentage,
        flash_loan_pool_fee_percentage + flash_loan_fee_percentage * (100 + flash_loan_pool_fee_percentage) / dec!(100),
        "There's something wrong in flash_loan_total_fee_percentage computation",
    );

    let (coin_bucket2, transient_nft_bucket) = radix_pump.get_flash_loan(
        coin_address,
        dec!(1000),
        &mut env
    )?;

    let fees = price * coin_bucket2.amount(&mut env)? * flash_loan_total_fee_percentage / dec!(100);
    radix_pump.return_flash_loan(
        transient_nft_bucket,
        base_coin_bucket1.take(fees, &mut env)?,
        coin_bucket2,
        &mut env
    )?;

    Ok(())
}

#[test]
#[should_panic]
fn test_flash_loan_insufficient_fees() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env).unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env).unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let flash_loan_fee_percentage = dec!("0.1");
    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!(1),
        dec!("0.3"),
        flash_loan_fee_percentage,
        package_address,
        &mut env
    ).unwrap();

    let flash_loan_pool_fee_percentage = dec!("0.1");
    let (_coin_creator_badge_bucket, coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        dec!(1000000),
        dec!(0.1),
        flash_loan_pool_fee_percentage,
        &mut env
    ).unwrap();
    let coin_address = coin_bucket1.resource_address(&mut env).unwrap();

    let (_, _, price, _, _, flash_loan_total_fee_percentage, _, _) = radix_pump.get_pool_info(coin_address, &mut env).unwrap();

    let (coin_bucket2, transient_nft_bucket) = radix_pump.get_flash_loan(
        coin_address,
        dec!(1000),
        &mut env
    ).unwrap();

    let fees = price * coin_bucket2.amount(&mut env).unwrap() * flash_loan_total_fee_percentage / dec!(100) - dec!("0.00001");
    radix_pump.return_flash_loan(
        transient_nft_bucket,
        base_coin_bucket1.take(fees, &mut env).unwrap(),
        coin_bucket2,
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_flash_loan_insufficient_amount() {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address =
        PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast).unwrap();

    let badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env).unwrap();
    let badge_address = badge_bucket.resource_address(&mut env).unwrap();

    let base_coin_bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000000), &mut env).unwrap();
    let base_coin_address = base_coin_bucket1.resource_address(&mut env).unwrap();

    let flash_loan_fee_percentage = dec!("0.1");
    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!(1),
        dec!("0.3"),
        flash_loan_fee_percentage,
        package_address,
        &mut env
    ).unwrap();

    let flash_loan_pool_fee_percentage = dec!("0.1");
    let (_coin_creator_badge_bucket, coin_bucket1) = radix_pump.create_new_coin(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        dec!(1000000),
        dec!(0.1),
        flash_loan_pool_fee_percentage,
        &mut env
    ).unwrap();
    let coin_address = coin_bucket1.resource_address(&mut env).unwrap();

    let (_, _, price, _, _, flash_loan_total_fee_percentage, _, _) = radix_pump.get_pool_info(coin_address, &mut env).unwrap();

    let (coin_bucket2, transient_nft_bucket) = radix_pump.get_flash_loan(
        coin_address,
        dec!(1000),
        &mut env
    ).unwrap();

    let _coin_bucket3 = coin_bucket2.take(dec!("0.00001"), &mut env).unwrap();

    let fees = price * coin_bucket2.amount(&mut env).unwrap() * flash_loan_total_fee_percentage / dec!(100);
    radix_pump.return_flash_loan(
        transient_nft_bucket,
        base_coin_bucket1.take(fees, &mut env).unwrap(),
        coin_bucket2,
        &mut env
    ).unwrap();
}

fn panic_if_significantly_different(
    x: Decimal,
    y: Decimal,
    error_message: &str,
) {
    assert!(
        (x / dec!(10000)) * dec!(10000) == (y / dec!(10000)) * dec!(10000),
        "{} {} {}",
        error_message,
        x,
        y,
    );
}


