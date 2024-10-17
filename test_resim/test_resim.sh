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
resim run forbid_symbols.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
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
echo -e "Quick launched ${quick_launched_coin}, receivd ${quick_launched_coin_received}\nCreator badge id: ${creator_badge_id}\nTest hook coin received: ${test_hook_coin_received}"

echo
enabled_operations='"PostBuy"'
resim run creator_enable_hook.rtm

exit

export coin=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
export price_creator=$(grep 'price:' $OUTPUTFILE | cut -d '"' -f 2)
export creator_allocation=$(grep 'creator_allocation:' $OUTPUTFILE | cut -d '"' -f 2)
export coins_in_pool=$(grep 'coins_in_pool:' $OUTPUTFILE | cut -d '"' -f 2)
echo Created coin $coin, received ${creator_allocation} coins, the price was ${price_creator}, there are ${coins_in_pool} coins in the pool

echo
export base_coin_amount1=1000
resim call-method $component buy $coin ${base_coin}:${base_coin_amount1} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export price1=$(grep 'price:' $OUTPUTFILE | cut -d '"' -f 2)
export coin_amount1=$(grep -A 1 "ResAddr: $coin" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
export coins_in_pool=$(grep 'coins_in_pool:' $OUTPUTFILE | cut -d '"' -f 2)
echo Bought ${coin_amount1} coin at $price1, paid ${base_coin_amount1} base coin, there are ${coins_in_pool} coins in the pool
echo Check that price is not too far the one paid by the creator

echo
export base_coin_amount2=500
resim call-method $component buy $coin ${base_coin}:${base_coin_amount2} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export price2=$(grep 'price:' $OUTPUTFILE | cut -d '"' -f 2)
export coin_amount2=$(grep -A 1 "ResAddr: $coin" $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 5)
export coins_in_pool=$(grep 'coins_in_pool:' $OUTPUTFILE | cut -d '"' -f 2)
echo Bought ${coin_amount2} coin at $price2, paid ${base_coin_amount2} base coin, there are ${coins_in_pool} coins in the pool
echo Check that the price increased a little bit

echo
export coin_amount3=${creator_allocation}
resim call-method $component sell ${coin}:${coin_amount3} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export price3=$(grep 'price:' $OUTPUTFILE | cut -d '"' -f 2)
export base_coin_amount3=$(grep -A 1 "ResAddr: ${base_coin}" $OUTPUTFILE | head -n 11 | tail -n 1 | cut -d ' ' -f 5)
export coins_in_pool=$(grep 'coins_in_pool:' $OUTPUTFILE | cut -d '"' -f 2)
echo Creator rugged ${coin_amount3} coin at $price3, received ${base_coin_amount3} base coin, there are ${coins_in_pool} coins in the pool

echo
export coin_amount4=1
resim call-method $component sell ${coin}:${coin_amount4} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export price4=$(grep 'price:' $OUTPUTFILE | cut -d '"' -f 2)
export base_coin_amount4=$(grep -A 1 "ResAddr: ${base_coin}" $OUTPUTFILE | head -n 11 | tail -n 1 | cut -d ' ' -f 5)
export coins_in_pool=$(grep 'coins_in_pool:' $OUTPUTFILE | cut -d '"' -f 2)
echo Sold ${coin_amount4} coin at $price4, received ${base_coin_amount4} base coin, there are ${coins_in_pool} coins in the pool
echo Check that price crashed

echo
resim call-method $component owner_set_liquidation_mode $coin --proofs "${owner_badge}:#1#" >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Component owner set liquidation mode

echo
export coin_amount5=${coin_amount2}
resim call-method $component sell ${coin}:${coin_amount5} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export price5=$(grep 'price:' $OUTPUTFILE | cut -d '"' -f 2)
export base_coin_amount5=$(grep -A 1 "ResAddr: ${base_coin}" $OUTPUTFILE | head -n 11 | tail -n 1 | cut -d ' ' -f 5)
export coins_in_pool=$(grep 'coins_in_pool:' $OUTPUTFILE | cut -d '"' -f 2)
echo Sold ${coin_amount5} coin at $price5, received ${base_coin_amount5} base coin, there are ${coins_in_pool} coins in the pool
echo Check that the price is quite higher

echo
export coin_amount6=$(echo ${coin_amount1} - 1 | bc)
resim call-method $component sell ${coin}:${coin_amount6} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export price6=$(grep 'price:' $OUTPUTFILE | cut -d '"' -f 2)
export base_coin_amount6=$(grep -A 1 "ResAddr: ${base_coin}" $OUTPUTFILE | head -n 11 | tail -n 1 | cut -d ' ' -f 5)
export coins_in_pool=$(grep 'coins_in_pool:' $OUTPUTFILE | cut -d '"' -f 2)
echo Sold ${coin_amount6} coin at $price6, received ${base_coin_amount6} base coin, there are ${coins_in_pool} coins in the pool
echo Check that the price has not changed

echo
export base_coin_amount7=500
resim call-method $component buy $coin ${base_coin}:${base_coin_amount7} >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; exit 1 )
echo Tried to buy the coin, the transaction correctly failed

echo
export base_coin_amount8=500
resim call-method $component buy ${base_coin} ${base_coin}:${base_coin_amount7} >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; exit 1 )
echo Tried to buy the base coin, the transaction correctly failed

#TODO: creator try to set liquidation mode

