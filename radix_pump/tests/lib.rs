use scrypto_test::prelude::*;
use radix_pump::radix_pump::radix_pump_test::*;
use scrypto::NonFungibleData;

#[derive(Debug, ScryptoSbor, PartialEq, Clone, Copy)]
enum PoolMode {
    WaitingForLaunch,
    Launching,
    Normal,
    Liquidation,
}

#[derive(Debug, ScryptoSbor, NonFungibleData)]
struct CreatorData {
    id: u64,
    coin_resource_address: ResourceAddress,
    coin_name: String,
    coin_symbol: String,
    creation_date: Instant,
    #[mutable]
    pool_mode: PoolMode,
}

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

    let (_coin_creator_badge_bucket, _coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(minimum_deposit - dec!(1), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
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

    let (_coin_creator_badge_bucket, _coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(minimum_deposit, &mut env).unwrap(),
        "COIN".to_string(),
        "    ".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "A test coin with only spaces in the name".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
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

    let (_coin_creator_badge_bucket, _coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(minimum_deposit, &mut env).unwrap(),
        "  ".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "A test coin wih only spaces in the symbol".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
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

    let (_coin_creator_badge_bucket, _coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(minimum_deposit, &mut env).unwrap(),
        " CoiN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();

    let (_coin_creator_badge2_bucket, _coin_bucket2, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(minimum_deposit, &mut env).unwrap(),
        "COIn ".to_string(),
        "AnotherCoin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Another coin with a very similar symbol".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
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

    let (_coin_creator_badge_bucket, _coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(minimum_deposit, &mut env).unwrap(),
        "COIN".to_string(),
        "Coin ".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();

    let _coin_creator_badge2_bucket = radix_pump.new_fair_launch(
        "COIN2".to_string(),
        " COIN ".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Another coin with a very similar name".to_string(),
        "".to_string(),
        dec!(1),
        dec!(10),
        dec!("0.1"),
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

    let (_coin_creator_badge_bucket, _coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "xrd".to_string(),
        "Radix".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "A coin with the same name and symbol as XRD".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
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

    let (_coin_creator_badge_bucket, _coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "xrd".to_string(),
        " Radix ".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "A coin with the same name and symbol as XRD".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
}

#[test]
fn test_buy() -> Result<(), RuntimeError> {
    let mut env = TestEnvironment::new();

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

    env.disable_auth_module();

    radix_pump.forbid_symbols(vec!["XRD".to_string()], &mut env)?;
    radix_pump.forbid_names(vec!["Radix".to_string()], &mut env)?;

    env.enable_auth_module();

    let (_coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env)?,
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;
    let coin_address = coin_bucket1.resource_address(&mut env)?;

    let (coin_bucket2, _buckets) = radix_pump.buy(
        coin_address,
        base_coin_bucket1.take(dec!(100), &mut env)?,
        &mut env
    )?;
    assert!(
        coin_bucket2.resource_address(&mut env)? == coin_address,
        "Wrong coin received",
    );

    let (coin_bucket3, _buckets) = radix_pump.buy(
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

    let (_coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();

    let (_bucket, _buckets) = radix_pump.buy(
        base_coin_address,
        coin_bucket1.take(dec!(100), &mut env).unwrap(),
        &mut env
    ).unwrap();
}

#[test]
fn test_sell() -> Result<(), RuntimeError> {
    let mut env = TestEnvironment::new();

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

    let (_coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(1000), &mut env)?,
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;

    let (base_coin_bucket2, _buckets) = radix_pump.sell(
        coin_bucket1.take(dec!(100), &mut env)?,
        &mut env
    )?;
    assert!(
        base_coin_bucket2.resource_address(&mut env)? == base_coin_address,
        "Wrong coin received",
    );

    let (base_coin_bucket3, _buckets) = radix_pump.sell(
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

    let (_coin_creator_badge_bucket, _coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
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
    let (_coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(deposit_amount, &mut env)?,
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;
    let coin_address = coin_bucket1.resource_address(&mut env)?;

    env.disable_auth_module();

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

    let (_coin_creator_badge_bucket, coin2_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(deposit_amount, &mut env)?,
        "C2".to_string(),
        "Coin 2".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just another test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;

    let (base_coin_bucket4, _buckets) = radix_pump.sell(
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
    let (_coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(deposit_amount, &mut env)?,
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;
    let coin_address = coin_bucket1.resource_address(&mut env)?;
    let base_coins_in_pool = deposit_amount * (dec!(1) - creation_fee_percentage / dec!(100));
    let coins_out_of_the_pool = coin_bucket1.amount(&mut env)?;

    env.disable_auth_module();

    radix_pump.owner_set_liquidation_mode(coin_address, &mut env)?;

    env.enable_auth_module();

    let mut coin_sold = dec!(100);
    let (base_coin_bucket2, _buckets) = radix_pump.sell(
        coin_bucket1.take(coin_sold, &mut env)?,
        &mut env
    )?;
    panic_if_significantly_different(
        base_coin_bucket2.amount(&mut env)?,
        (100 - buy_sell_fee_percentage ) * base_coins_in_pool * coin_sold / (100 * coins_out_of_the_pool),
        "Wrong amount received on sale 1"
    );

    coin_sold = dec!(500);
    let (base_coin_bucket3, _buckets) = radix_pump.sell(
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

    let (_coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
    let coin_address = coin_bucket1.resource_address(&mut env).unwrap();

    let _ = radix_pump.owner_set_liquidation_mode(coin_address, &mut env);

    let (_coin_bucket2, _buckets) = radix_pump.buy(
        coin_address,
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        &mut env
    ).unwrap();
}

#[test]
fn test_flash_loan() -> Result<(), RuntimeError> {
    let mut env = TestEnvironment::new();

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
    let (_coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env)?,
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!(0.1),
        dec!(0.1),
        flash_loan_pool_fee_percentage,
        &mut env
    )?;
    let coin_address = coin_bucket1.resource_address(&mut env)?;

    let pool_info = radix_pump.get_pool_info(coin_address, &mut env)?;
    let price = pool_info.last_price;
    let flash_loan_total_fee_percentage = pool_info.total_flash_loan_fee_percentage;

    panic_if_significantly_different(
        flash_loan_total_fee_percentage,
        flash_loan_pool_fee_percentage + flash_loan_fee_percentage,
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
    let (_coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!(0.1),
        dec!(0.1),
        flash_loan_pool_fee_percentage,
        &mut env
    ).unwrap();
    let coin_address = coin_bucket1.resource_address(&mut env).unwrap();

    let pool_info = radix_pump.get_pool_info(coin_address, &mut env).unwrap();
    let price = pool_info.last_price;
    let flash_loan_total_fee_percentage = pool_info.total_flash_loan_fee_percentage;

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
    let (_coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!(0.1),
        dec!(0.1),
        flash_loan_pool_fee_percentage,
        &mut env
    ).unwrap();
    let coin_address = coin_bucket1.resource_address(&mut env).unwrap();

    let pool_info = radix_pump.get_pool_info(coin_address, &mut env).unwrap();
    let price = pool_info.last_price;
    let flash_loan_total_fee_percentage = pool_info.total_flash_loan_fee_percentage;

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

#[test]
fn test_fair_launch() -> Result<(), RuntimeError> {
    let mut env = TestEnvironment::new();

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

    let creation_fee = dec!("1");
    let owner_buy_sell_fee = dec!("0.3");
    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        creation_fee,
        owner_buy_sell_fee,
        dec!("0.1"),
        package_address,
        &mut env
    )?;

    let price = dec!(0.2);
    let percentage_coin_to_creator = dec!(10);
    let buy_fee = dec!("0.1");
    let coin_creator_badge_bucket = radix_pump.new_fair_launch(
        "FCOIN".to_string(),
        "Fair Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Fair launched coin".to_string(),
        "".to_string(),
        price,
        percentage_coin_to_creator,
        dec!("0.1"),
        buy_fee,
        dec!("0.1"),
        &mut env
    )?;

    let resource_manager = ResourceManager(coin_creator_badge_bucket.resource_address(&mut env)?);
    let creator_data = resource_manager.get_non_fungible_data::<_, _, CreatorData>(
        coin_creator_badge_bucket.non_fungible_local_ids(&mut env)?.first().unwrap().clone(),
        &mut env
    )?;
    let coin_resource_address = creator_data.coin_resource_address;

    env.disable_auth_module();
    let min_launch_duration = 100;
    let min_lock_duration = 10000;
    radix_pump.update_time_limits(min_launch_duration, min_lock_duration, &mut env)?;
    env.enable_auth_module();

    let now = 1800000000;
    env.set_current_time(Instant::new(now));

    radix_pump.launch(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        )?,
        now + min_launch_duration,
        now + min_launch_duration + min_lock_duration,
        &mut env
    )?;

    let pool_info = radix_pump.get_pool_info(coin_resource_address, &mut env)?;
    let last_price = pool_info.last_price;
    let total_buy_fee = pool_info.total_buy_fee_percentage;
    let end_launch_time = pool_info.end_launch_time;
    let unlocking_time = pool_info.unlocking_time;

    assert!(
        end_launch_time.unwrap() == now + min_launch_duration,
        "Wrong end_launch_time reported",
    );
    assert!(
        unlocking_time.unwrap() == now + min_launch_duration + min_lock_duration,
        "Wrong unlocking_time reported",
    );

    let base_coin_amount1 = dec!(100);
    let (coin_bucket1, _buckets) = radix_pump.buy(
        coin_resource_address,
        base_coin_bucket1.take(base_coin_amount1, &mut env)?,
        &mut env
    )?;
    assert!(
        coin_resource_address == coin_bucket1.resource_address(&mut env)?,
        "Wrong coin received",
    );
    let price1 = base_coin_amount1 / coin_bucket1.amount(&mut env)?;
    panic_if_significantly_different(
        last_price * dec!(100) / (dec!(100) - total_buy_fee),
        price1,
        "Wrong price",
    );

    let base_coin_amount2 = dec!(200);
    let (coin_bucket2, _buckets) = radix_pump.buy(
        coin_resource_address,
        base_coin_bucket1.take(base_coin_amount2, &mut env)?,
        &mut env
    )?;
    let price2 = base_coin_amount2 / coin_bucket2.amount(&mut env)?;
    panic_if_significantly_different(
        price1,
        price2,
        "Price should not change in Launching mode",
    );

    env.set_current_time(Instant::new(now + min_launch_duration));

    let (base_coin_bucket, _buckets) = radix_pump.terminate_launch(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        )?,
        &mut env
    )?;
    panic_if_significantly_different(
        base_coin_bucket.amount(&mut env)?,
        (base_coin_amount1 + base_coin_amount2) * ((dec!(100) - owner_buy_sell_fee) / dec!(100)) * ((dec!(100) - buy_fee) / dec!(100)) * ((dec!(100) - creation_fee) / dec!(100)),
        "Wrong base coin amount to the creator",
    );

    let base_coin_amount3 = dec!(300);
    let (coin_bucket3, _buckets) = radix_pump.buy(
        coin_resource_address,
        base_coin_bucket1.take(base_coin_amount3, &mut env)?,
        &mut env
    )?;
    let price3 = base_coin_amount3 / coin_bucket3.amount(&mut env)?;
    assert!(
        price3 > price2,
        "Price should move in Normal mode",
    );

    env.set_current_time(Instant::new(now + min_launch_duration + min_lock_duration));

    let (coin_bucket, _buckets) = radix_pump.unlock(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        )?,
        None,
        false,
        &mut env
    )?;

    let pool_info = radix_pump.get_pool_info(coin_resource_address, &mut env)?;
    let initial_locked_amount = pool_info.initial_locked_amount;
    let unlocked_amount = pool_info.unlocked_amount;

    panic_if_significantly_different(
        initial_locked_amount.unwrap(),
        coin_bucket.amount(&mut env)?,
        "Wrong number of coins to the creator",
    );
    panic_if_significantly_different(
        unlocked_amount.unwrap(),
        coin_bucket.amount(&mut env)?,
        "Wrong unlocked_amount reported",
    );

    Ok(())
}

#[test]
#[should_panic]
fn test_wrong_creator_badge() {
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

    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!("1"),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let _coin_creator_badge_bucket = radix_pump.new_fair_launch(
        "FCOIN".to_string(),
        "Fair Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Fair launched coin".to_string(),
        "".to_string(),
        dec!(0.2),
        dec!(10),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();

    let min_launch_duration = 100;
    let min_lock_duration = 10000;
    let _ = radix_pump.update_time_limits(min_launch_duration, min_lock_duration, &mut env).unwrap();

    let now = 1800000000;
    env.set_current_time(Instant::new(now));

    radix_pump.launch(
        badge_bucket.create_proof_of_amount( // Wrong badge
            dec!(1),
            &mut env
        ).unwrap(),
        now + min_launch_duration,
        now + min_launch_duration + min_lock_duration,
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_fair_launch_too_short_duration() {
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

    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!("1"),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let coin_creator_badge_bucket = radix_pump.new_fair_launch(
        "FCOIN".to_string(),
        "Fair Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Fair launched coin".to_string(),
        "".to_string(),
        dec!(0.2),
        dec!(10),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();

    let min_launch_duration = 100;
    let min_lock_duration = 10000;
    let _ = radix_pump.update_time_limits(min_launch_duration, min_lock_duration, &mut env).unwrap();

    let now = 1800000000;
    env.set_current_time(Instant::new(now));

    radix_pump.launch(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        ).unwrap(),
        now + min_launch_duration - 1, // Too short
        now + min_launch_duration + min_lock_duration,
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_fair_launch_too_short_unlock() {
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

    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!("1"),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let coin_creator_badge_bucket = radix_pump.new_fair_launch(
        "FCOIN".to_string(),
        "Fair Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Fair launched coin".to_string(),
        "".to_string(),
        dec!(0.2),
        dec!(10),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();

    let min_launch_duration = 100;
    let min_lock_duration = 10000;
    let _ = radix_pump.update_time_limits(min_launch_duration, min_lock_duration, &mut env).unwrap();

    let now = 1800000000;
    env.set_current_time(Instant::new(now));

    radix_pump.launch(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        ).unwrap(),
        now + min_launch_duration,
        now + min_launch_duration + min_lock_duration - 1, // Too short
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_fair_launch_terminate_too_soon() {
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

    let mut radix_pump = RadixPump::new(
        badge_address,
        base_coin_address,
        dec!(100),
        dec!("1"),
        dec!("0.3"),
        dec!("0.1"),
        package_address,
        &mut env
    ).unwrap();

    let coin_creator_badge_bucket = radix_pump.new_fair_launch(
        "FCOIN".to_string(),
        "Fair Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Fair launched coin".to_string(),
        "".to_string(),
        dec!(0.2),
        dec!(10),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();

    let min_launch_duration = 100;
    let min_lock_duration = 10000;
    let _ = radix_pump.update_time_limits(min_launch_duration, min_lock_duration, &mut env).unwrap();

    let now = 1800000000;
    env.set_current_time(Instant::new(now));

    radix_pump.launch(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        ).unwrap(),
        now + min_launch_duration,
        now + min_launch_duration + min_lock_duration,
        &mut env
    ).unwrap();

    env.set_current_time(Instant::new(now + min_launch_duration - 1));

    let (_base_coin_bucket, _buckets) = radix_pump.terminate_launch(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        ).unwrap(),
        &mut env
    ).unwrap();
}

#[test]
fn test_hook() -> Result<(), RuntimeError> {
    let mut env = TestEnvironment::new();

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

    let (coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env)?,
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;
    let coin_address = coin_bucket1.resource_address(&mut env)?;

    let pool_info = radix_pump.get_pool_info(coin_address, &mut env).unwrap();
    let hooks_badge_address = pool_info.hooks_badge_resource_address;

    let hook = Hook::new(
        badge_address,
        hooks_badge_address,
        package_address,
        &mut env
    )?;

    env.disable_auth_module();

    radix_pump.register_hook(
        "test hook".to_string(),
        vec![
            "PostFairLaunch".to_string(),
            "PostTerminateFairLaunch".to_string(),
            "PostQuickLaunch".to_string(),
            "PostBuy".to_string(),
            "PostSell".to_string(),
            "PostReturnFlashLoan".to_string()
        ],
        hook.into(),
        &mut env
    )?;
    radix_pump.register_hook(
        "test hook".to_string(),
        vec![
            "PostFairLaunch".to_string(),
        ],
        hook.into(),
        &mut env
    )?;

    radix_pump.owner_enable_hook(
        "test hook".to_string(),
        vec![
            "PostFairLaunch".to_string(),
            "PostTerminateFairLaunch".to_string(),
            "PostQuickLaunch".to_string(),
        ],
        &mut env
    )?;
    
    radix_pump.creator_enable_hook(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        )?,
        "test hook".to_string(),
        vec![
            "PostBuy".to_string(),
            "PostSell".to_string(),
            "PostReturnFlashLoan".to_string()
        ],
        &mut env
    )?;

    env.enable_auth_module();

    let (_coin2_creator_badge_bucket, _coin2_bucket1, buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env)?,
        "COIN2".to_string(),
        "Coin 2".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just another test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;
    assert!(
        buckets.len() == 1,
        "PostQuickLaunch hook not called",
    );

    let (_coin_bucket2, buckets) = radix_pump.buy(
        coin_address,
        base_coin_bucket1.take(dec!(100), &mut env)?,
        &mut env
    )?;
    assert!(
        buckets.len() == 1,
        "PostBuy hook not called",
    );

    env.disable_auth_module();

    radix_pump.unregister_hook(
        "test hook".to_string(),
        Some(vec!["PostSell".to_string()]),
        &mut env
    )?;

    env.enable_auth_module();

    let (_base_coin_bucket2, buckets) = radix_pump.sell(
        coin_bucket1,
        &mut env
    )?;
    assert!(
        buckets.len() == 0,
        "Unregistered PostSell hook called",
    );

    env.disable_auth_module();

    radix_pump.unregister_hook(
        "test hook".to_string(),
        None,
        &mut env
    )?;

    env.enable_auth_module();

    let (_coin_bucket3, buckets) = radix_pump.buy(
        coin_address,
        base_coin_bucket1.take(dec!(100), &mut env)?,
        &mut env
    )?;
    assert!(
        buckets.len() == 0,
        "Unregistered hook called",
    );

    Ok(())
}

#[test]
#[should_panic]
fn test_hook_wrong_operation() {
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

    let (coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
    let coin_address = coin_bucket1.resource_address(&mut env).unwrap();

    let pool_info = radix_pump.get_pool_info(coin_address, &mut env).unwrap();
    let hooks_badge_address = pool_info.hooks_badge_resource_address;

    let hook = Hook::new(
        badge_address,
        hooks_badge_address,
        package_address,
        &mut env
    ).unwrap();

    radix_pump.register_hook(
        "test hook".to_string(),
        vec![
            "PostQuickLaunch".to_string(),
        ],
        hook.into(),
        &mut env
    ).unwrap();

    radix_pump.creator_enable_hook(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        ).unwrap(),
        "test hook".to_string(),
        vec![
            "PostBuy".to_string(), // This is wrong
        ],
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_hook_unregistered_operation() {
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

    let (coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
    let coin_address = coin_bucket1.resource_address(&mut env).unwrap();

    let pool_info = radix_pump.get_pool_info(coin_address, &mut env).unwrap();
    let hooks_badge_address = pool_info.hooks_badge_resource_address;

    let hook = Hook::new(
        badge_address,
        hooks_badge_address,
        package_address,
        &mut env
    ).unwrap();

    radix_pump.register_hook(
        "test hook".to_string(),
        vec!["PostBuy".to_string()],
        hook.into(),
        &mut env
    ).unwrap();

    radix_pump.unregister_hook(
        "test hook".to_string(),
        Some(vec!["PostBuy".to_string()]),
        &mut env
    ).unwrap();

    radix_pump.creator_enable_hook(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        ).unwrap(),
        "test hook".to_string(),
        vec!["PostBuy".to_string()],
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_hook_unregistered_hook() {
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

    let (coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
    let coin_address = coin_bucket1.resource_address(&mut env).unwrap();

    let pool_info = radix_pump.get_pool_info(coin_address, &mut env).unwrap();
    let hooks_badge_address = pool_info.hooks_badge_resource_address;

    let hook = Hook::new(
        badge_address,
        hooks_badge_address,
        package_address,
        &mut env
    ).unwrap();

    radix_pump.register_hook(
        "test hook".to_string(),
        vec!["PostBuy".to_string()],
        hook.into(),
        &mut env
    ).unwrap();

    radix_pump.unregister_hook(
        "test hook".to_string(),
        None,
        &mut env
    ).unwrap();

    radix_pump.creator_enable_hook(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        ).unwrap(),
        "test hook".to_string(),
        vec!["PostBuy".to_string()],
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_hook_wrong_name() {
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

    let (coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
    let coin_address = coin_bucket1.resource_address(&mut env).unwrap();

    let pool_info = radix_pump.get_pool_info(coin_address, &mut env).unwrap();
    let hooks_badge_address = pool_info.hooks_badge_resource_address;

    let hook = Hook::new(
        badge_address,
        hooks_badge_address,
        package_address,
        &mut env
    ).unwrap();

    radix_pump.register_hook(
        "test hook".to_string(),
        vec!["PostBuy".to_string()],
        hook.into(),
        &mut env
    ).unwrap();

    radix_pump.creator_enable_hook(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        ).unwrap(),
        "another hook".to_string(), // This is wrong
        vec!["PostBuy".to_string()],
        &mut env
    ).unwrap();
}

#[test]
#[should_panic]
fn test_hook_wrong_badge() {
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

    let hook = Hook::new(
        badge_address,
        badge_address, // This is wrong
        package_address,
        &mut env
    ).unwrap();

    radix_pump.register_hook(
        "test hook".to_string(),
        vec!["PostQuickLaunch".to_string()],
        hook.into(),
        &mut env
    ).unwrap();

    radix_pump.owner_enable_hook(
        "test hook".to_string(),
        vec!["PostQuickLaunch".to_string()],
        &mut env
    ).unwrap();

    env.enable_auth_module();

    let (_coin_creator_badge_bucket, _coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env).unwrap(),
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    ).unwrap();
}

#[test]
fn test_burn() -> Result<(), RuntimeError> {
    let mut env = TestEnvironment::new();

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

    let (coin_creator_badge_bucket, coin_bucket1, _buckets) = radix_pump.new_quick_launch(
        base_coin_bucket1.take(dec!(100), &mut env)?,
        "COIN".to_string(),
        "Coin".to_string(),
        "https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_string(),
        "Just a test coin".to_string(),
        "".to_string(),
        dec!(1000000),
        dec!(1),
        dec!("0.1"),
        dec!("0.1"),
        dec!("0.1"),
        &mut env
    )?;
    let coin_address = coin_bucket1.resource_address(&mut env)?;

    let (_coin_bucket2, _buckets) = radix_pump.buy(
        coin_address,
        base_coin_bucket1.take(dec!(100), &mut env)?,
        &mut env
    )?;

    let pool_info1 = radix_pump.get_pool_info(coin_address, &mut env).unwrap();

    let max_amount_to_burn = dec!(500);

    env.disable_auth_module();

    radix_pump.burn(
        coin_creator_badge_bucket.create_proof_of_non_fungibles(
            IndexSet::from([1.into()]),
            &mut env
        )?,
        max_amount_to_burn,
        &mut env
    )?;

    let pool_info2 = radix_pump.get_pool_info(coin_address, &mut env).unwrap();

    let diff = pool_info1.coin_amount - pool_info2.coin_amount;
    assert!(
        diff > Decimal::ZERO,
        "No coins burned",
    );
    assert!(
        diff <= max_amount_to_burn,
        "Too many coins burned",
    );

    Ok(())
}

fn panic_if_significantly_different(
    x: Decimal,
    y: Decimal,
    error_message: &str,
) {
    assert!(
        (x - y).checked_abs().unwrap() / x < dec!("0.00001"),
        "{} {} {}",
        error_message,
        x,
        y,
    );
}
