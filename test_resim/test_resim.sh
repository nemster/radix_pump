#!/bin/bash

set -e

OUTPUTFILE=$(mktemp)

clear
resim reset

echo
resim new-account >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export account=$(grep 'Account component address:' $OUTPUTFILE | cut -d ' ' -f 4)
export owner_badge=$(grep 'Owner badge:' $OUTPUTFILE | cut -d ':' -f 2 | tr -d '[:space:]')
export owner_badge_id=$(grep 'Owner badge:' $OUTPUTFILE | cut -d ':' -f 3)
echo -e "Account address: $account\nOwner badge: $owner_badge\nOwner badge id: ${owner_badge_id}"

echo
resim publish ../radix_pump >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export radix_pump_package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo RadixPump package: ${radix_pump_package}

echo
export base_coin=resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3
export minimum_deposit=1000
export creation_fee_percentage=0.1
export buy_sell_fee_percentage=0.1
export flash_loan_fee_percentage=0.1
resim call-function ${radix_pump_package} RadixPump new ${owner_badge} ${base_coin} ${minimum_deposit} ${creation_fee_percentage} ${buy_sell_fee_percentage} ${flash_loan_fee_percentage} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export radix_pump_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
export creator_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
export flash_loan_nft=$(grep 'Resource:' $OUTPUTFILE | head -n 2 | tail -n 1 | cut -d ' ' -f 3)
export hooks_badge=$(grep 'Resource:' $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 3)
echo -e "RadixPump component: ${radix_pump_component}\nCreator badge: ${creator_badge}\nFlash loan transient NFT: ${flash_loan_nft}\nHooks authentication badge: ${hooks_badge}"

echo
export forbidden_symbols='"XRD"'
resim run forbid_symbols.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Symbols ${forbidden_symbols} forbidden

echo
export forbidden_names='"Radix"'
resim run forbid_names.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Names ${forbidden_names} forbidden

echo
resim publish ../hooks >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export hooks_package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo Hooks package: ${hooks_package}

echo
resim call-function ${hooks_package} TestHook new ${owner_badge} ${hooks_badge} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export test_hook_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
export test_hook_coin=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
echo -e "TestHook component: ${test_hook_component}\nTestHook coin: ${test_hook_coin}"

echo
export hook_name=TestHook
export operations='"PostFairLaunch", "PostTerminateFairLaunch", "PostQuickLaunch", "PostBuy", "PostSell", "PostReturnFlashLoan"'
resim run register_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Registered hook ${hook_name} for operations ${operations}

echo
export globally_enabled_operations='"PostFairLaunch", "PostTerminateFairLaunch", "PostQuickLaunch"'
resim run owner_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Globally enabled hook ${hook_name} for operations ${globally_enabled_operations}

echo
export globally_disabled_operations='"PostFairLaunch"'
resim run owner_disable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Globally disabled hook ${hook_name} for operations ${globally_disabled_operations}

echo
export symbol=QL
export name=QuickLaunchedCoin
export icon=https://media-cdn.tripadvisor.com/media/photo-s/1a/ce/31/66/photo-de-profil.jpg
export description="Quick launched coin"
export info_url=""
export supply=1000000
export price=10
export buy_pool_fee=0.1
export sell_pool_fee=0.1
export flash_loan_pool_fee=0.1
resim call-method ${radix_pump_component} new_quick_launch ${base_coin}:${minimum_deposit} $symbol $name $icon "$description" "${info_url}" $supply $price $buy_pool_fee $sell_pool_fee $flash_loan_pool_fee >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export quick_launched_coin=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
export creator_badge_id="#$(grep -A 1 "ResAddr: ${creator_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
export test_hook_coin_received=$(grep -A 1 "ResAddr: ${test_hook_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
export quick_launched_coin_received=$(grep -A 1 "ResAddr: ${quick_launched_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
echo -e "Quick launched ${quick_launched_coin}, received ${quick_launched_coin_received}\nCreator badge id: ${creator_badge_id}\nTest hook coin received: ${test_hook_coin_received}"

echo
export name=SameSymbolCoin
resim call-method ${radix_pump_component} new_quick_launch ${base_coin}:${minimum_deposit} $symbol $name $icon "$description" "${info_url}" $supply $price $buy_pool_fee $sell_pool_fee $flash_loan_pool_fee >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to create a new coin with the same symbol and the transaction failed as expected

echo
export symbol=QL2
export name=QuickLaunchedCoin
resim call-method ${radix_pump_component} new_quick_launch ${base_coin}:${minimum_deposit} $symbol $name $icon "$description" "${info_url}" $supply $price $buy_pool_fee $sell_pool_fee $flash_loan_pool_fee >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to create a new coin with the same name and the transaction failed as expected

echo
export symbol=XRD
export name=FakeRadix
resim call-method ${radix_pump_component} new_quick_launch ${base_coin}:${minimum_deposit} $symbol $name $icon "$description" "${info_url}" $supply $price $buy_pool_fee $sell_pool_fee $flash_loan_pool_fee >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to create a new coin with XRD as symbol and the transaction failed as expected

echo
export symbol=XXX
export name=Radix
resim call-method ${radix_pump_component} new_quick_launch ${base_coin}:${minimum_deposit} $symbol $name $icon "$description" "${info_url}" $supply $price $buy_pool_fee $sell_pool_fee $flash_loan_pool_fee >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to create a new coin with Radix as name and the transaction failed as expected

echo
export name=YYY
resim call-method ${radix_pump_component} new_quick_launch ${base_coin}:$((${minimum_deposit} - 1)) $symbol $name $icon "$description" "${info_url}" $supply $price $buy_pool_fee $sell_pool_fee $flash_loan_pool_fee >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to create a new coin with an insufficient base coin deposit and the transaction failed as expected

echo
export enabled_operations='"PostBuy", "PostSell", "PostReturnFlashLoan"'
resim run creator_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Enabled hook ${hook_name} for operations ${enabled_operations} on ${quick_launched_coin}

echo
export disabled_operations='"PostSell"'
resim run creator_disable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Disabled hook ${hook_name} for operations ${disabled_operations} on ${quick_launched_coin}

echo
export payment=1000
resim call-method ${radix_pump_component} buy ${quick_launched_coin} ${base_coin}:$payment >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export quick_launched_coin_received=$(grep -A 1 "ResAddr: ${quick_launched_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
export test_hook_coin_received=$(grep -A 1 "ResAddr: ${test_hook_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
echo -e "Bought ${quick_launched_coin_received} ${quick_launched_coin} for $payment ${base_coin}\n${test_hook_coin_received} test hook coin received"

echo
export payment=10
resim call-method ${radix_pump_component} sell ${quick_launched_coin}:$payment >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export base_coin_received=$(grep -A 1 "ResAddr: ${base_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
export test_hook_coin_received=$(grep -A 1 "ResAddr: ${test_hook_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
echo -e "Sold $payment ${quick_launched_coin} for ${base_coin_received} ${base_coin}\n${test_hook_coin_received} test hook coin received"

echo
export symbol=FL
export name=FairLaunchedCoin
export icon=https://fairitaly.org/fair/wp-content/uploads/2023/03/logofairtondo.png
export description="Fair launched coin"
export info_url=""
export price=100
export creator_locked_percentage=10
export buy_pool_fee=5
export sell_pool_fee=0.1
export flash_loan_pool_fee=0.1
resim call-method ${radix_pump_component} new_fair_launch $symbol $name $icon "$description" "${info_url}" $price $creator_locked_percentage $buy_pool_fee $sell_pool_fee $flash_loan_pool_fee >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export fair_launched_coin=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
export creator_badge_id="#$(grep -A 1 "ResAddr: ${creator_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
export test_hook_coin_received=$(grep -A 1 "ResAddr: ${test_hook_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
export fair_launched_coin_received=$(grep -A 1 "ResAddr: ${fair_launched_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
echo -e "Fair launched ${fair_launched_coin}, received ${fair_launched_coin_received}\nCreator badge id: ${creator_badge_id}\nTest hook coin received: ${test_hook_coin_received}"

echo
export min_launch_duration=604800
export min_lock_duration=5184000
resim call-method ${radix_pump_component} update_time_limits $min_launch_duration $min_lock_duration --proofs ${owner_badge}:${owner_badge_id} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Set limits min_launch_duration: 604800 min_lock_duration: 5184000

echo
export unix_epoch=1800000000
date=$(date -u -d @${unix_epoch} +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $date

export end_launch_time=$(($unix_epoch + $min_launch_duration -1))
export unlocking_time=$(($unix_epoch + $min_launch_duration + $min_lock_duration))
resim run launch.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to launch with a launching perdiod too short, the transaction faild as expected

echo
export end_launch_time=$(($unix_epoch + $min_launch_duration))
export unlocking_time=$(($unix_epoch + $min_launch_duration + $min_lock_duration - 1))
resim run launch.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to launch with an unlocking perdiod too short, the transaction faild as expected

echo
export end_launch_time=$(($unix_epoch + $min_launch_duration))
export unlocking_time=$(($unix_epoch + $min_launch_duration + $min_lock_duration))
resim run launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export test_hook_coin_received=$(grep -A 1 "ResAddr: ${test_hook_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
export fair_launched_coin_received=$(grep -A 1 "ResAddr: ${fair_launched_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
echo -e "Fair sale launched for ${fair_launched_coin}, received ${fair_launched_coin_received}\nTest hook coin received: ${test_hook_coin_received}"

echo
export payment=1000
resim call-method ${radix_pump_component} buy ${fair_launched_coin} ${base_coin}:$payment >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export fair_launched_coin_received=$(grep -A 1 "ResAddr: ${fair_launched_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
export test_hook_coin_received=$(grep -A 1 "ResAddr: ${test_hook_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
echo -e "Bought ${fair_launched_coin_received} ${fair_launched_coin} for $payment ${base_coin}\n${test_hook_coin_received} test hook coin received"

echo
export payment=1000
resim call-method ${radix_pump_component} buy ${fair_launched_coin} ${base_coin}:$payment >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export fair_launched_coin_received=$(grep -A 1 "ResAddr: ${fair_launched_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
export test_hook_coin_received=$(grep -A 1 "ResAddr: ${test_hook_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
echo -e "Bought ${fair_launched_coin_received} ${fair_launched_coin} for $payment ${base_coin} (price should not have changed)\n${test_hook_coin_received} test hook coin received"

echo
export payment=1
resim call-method ${radix_pump_component} sell ${fair_launched_coin}:$payment >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to sellf ${fair_launched_coin} during fair launch, it is forbidden so the transaction failed

echo
date=$(date -u -d @$(($end_launch_time -1)) +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $date

resim run terminate_launch.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to terminate launch ahead of time and the transaction failed as expected

echo
date=$(date -u -d @$end_launch_time +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $date

resim run terminate_launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export base_coin_received=$(grep -A 1 "ResAddr: ${base_coin}" $OUTPUTFILE | head -n 2 | tail -n 1 | cut -d ' ' -f 5)
export test_hook_coin_received=$(grep -A 1 "ResAddr: ${test_hook_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
echo -e "Fair launch terminated, received ${base_coin_received} ${base_coin}\n${test_hook_coin_received} test hook coin received"

echo
export payment=1
resim call-method ${radix_pump_component} sell ${fair_launched_coin}:$payment >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export base_coin_received=$(grep -A 1 "ResAddr: ${base_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
export test_hook_coin_received=$(grep -A 1 "ResAddr: ${test_hook_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
echo -e "Sold $payment ${fair_launched_coin} for ${base_coin_received} ${base_coin}\n${test_hook_coin_received} test hook coin received"

echo
date=$(date -u -d @$(($end_launch_time + 604800)) +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $date

export amount=100000
export sell=false
resim run unlock.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export fair_launched_coin_received=$(grep -A 1 "ResAddr: ${fair_launched_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
echo Tried to unlock $amount $fair_launched_coin, $fair_launched_coin_received received

echo
export amount=100000
export sell=false
resim run unlock.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export fair_launched_coin_received=$(grep -A 1 "ResAddr: ${fair_launched_coin}" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
echo Tried to unlock $amount $fair_launched_coin, $fair_launched_coin_received received

echo
date=$(date -u -d @$(($unlocking_time + 604800)) +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $date

export amount=100000
export sell=true
resim run unlock.rtm
