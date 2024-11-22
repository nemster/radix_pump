#!/bin/bash

update_wallet_amounts() {
  resim show |
    grep ' resource_sim' |
    tr -d : |
    awk '{print $2 " " $3}' >$WALLETFILE
}

increase_in_wallet() {
  old_amount=$(grep $1 $WALLETFILE | cut -d ' ' -f 2)
  if [ "$old_amount" = "" ]
  then
    old_amount=0
  fi

  amount=$(resim show | grep $1 | cut -d ' ' -f 3)
  if [ "$amount" = "" ]
  then
    amount=0
  fi

  echo $amount - $old_amount | bc
}

OUTPUTFILE=$(mktemp)
WALLETFILE=$(mktemp)

set -e

clear
resim reset

echo
resim new-account >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export account=$(grep 'Account component address:' $OUTPUTFILE | cut -d ' ' -f 4)
export owner_badge=$(grep 'Owner badge:' $OUTPUTFILE | cut -d ':' -f 2 | tr -d '[:space:]')
export owner_badge_id=$(grep 'Owner badge:' $OUTPUTFILE | cut -d ':' -f 3)
echo -e "Account address: $account\nOwner badge: $owner_badge\nOwner badge id: ${owner_badge_id}"

echo
resim publish ../random_component >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export random_component_package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo RandomComponent package: ${random_component_package}

echo
echo resim call-function ${random_component_package} RandomComponent new
resim call-function ${random_component_package} RandomComponent new >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export random_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
echo RandomComponent: ${random_component}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
resim publish ../radix_pump >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export radix_pump_package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo RadixPump package: ${radix_pump_package}

echo
export base_coin=resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3
export minimum_deposit=1000
export creation_fee_percentage=0.1
export buy_sell_fee_percentage=0.1
export flash_loan_fee=1
echo resim call-function ${radix_pump_package} RadixPump new ${owner_badge} ${base_coin} ${minimum_deposit} ${creation_fee_percentage} ${buy_sell_fee_percentage} ${flash_loan_fee} ${account}
resim call-function ${radix_pump_package} RadixPump new ${owner_badge} ${base_coin} ${minimum_deposit} ${creation_fee_percentage} ${buy_sell_fee_percentage} ${flash_loan_fee} ${account} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export radix_pump_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
export creator_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
export hook_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 3 | tail -n 1 | cut -d ' ' -f 3)
export proxy_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 4 | tail -n 1 | cut -d ' ' -f 3)
echo -e "RadixPump component: ${radix_pump_component}\nCreator badge: ${creator_badge}\nHook badge: ${hook_badge}\nProxy badge: ${proxy_badge}"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
update_wallet_amounts
echo resim call-method ${radix_pump_component} get_badges --proofs "${owner_badge}:${owner_badge_id}"
resim call-method ${radix_pump_component} get_badges --proofs "${owner_badge}:${owner_badge_id}" >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Received $(increase_in_wallet ${hook_badge}) ${hook_badge} and $(increase_in_wallet ${proxy_badge}) ${proxy_badge}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
resim publish ../timer >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export timer_package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo Timer package: ${timer_package}

echo
export max_hourly_frequency=1
echo resim call-function ${timer_package} Timer new ${owner_badge} ${radix_pump_component} ${proxy_badge}:1 ${hook_badge}:1 ${max_hourly_frequency}
resim call-function ${timer_package} Timer new ${owner_badge} ${radix_pump_component} ${proxy_badge}:1 ${hook_badge}:1 ${max_hourly_frequency} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export timer_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
export timer_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
export alarm_clock_badge=$(grep 'Resource:' $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 3)
echo -e "Timer component: ${timer_component}\nTimer badge: ${timer_badge}\nAlarm clock badge: ${alarm_clock_badge}"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
resim publish ../hooks/dca >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export dca_package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo Dca package: ${dca_package}

echo
echo resim call-function ${dca_package} Dca new ${owner_badge} ${proxy_badge} ${timer_badge} ${base_coin} ${radix_pump_component}
resim call-function ${dca_package} Dca new ${owner_badge} ${proxy_badge} ${timer_badge} ${base_coin} ${radix_pump_component} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export dca_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
echo DCA component: ${dca_component}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export hook_name=DCA
export hook_component=${dca_component}
echo resim run manifests/timer_register_hook.rtm
resim run manifests/timer_register_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo ${hook_name} hook registered
grep 'Transaction Cost: ' $OUTPUTFILE

echo
update_wallet_amounts
export base_coin_amount=${minimum_deposit}
export symbol=QL1
export name=QuickLaunched1
export icon=https://media-cdn.tripadvisor.com/media/photo-s/1a/ce/31/66/photo-de-profil.jpg
export description="Quick launched coin 1"
export info_url=""
export social_url='Array<String>()'
export supply=1000000
export price=10
export buy_pool_fee=0.1
export sell_pool_fee=0.1
export flash_loan_pool_fee=1
echo run manifests/new_quick_launch.rtm
resim run manifests/new_quick_launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export quick_launched_coin1=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
export creator_badge_id1="#$(grep -A 1 "ResAddr: ${creator_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
echo Quick launched ${quick_launched_coin1}, received creator badge ${creator_badge}:${creator_badge_id1} and $(increase_in_wallet ${quick_launched_coin1}) ${quick_launched_coin1}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
update_wallet_amounts
export symbol=QL2
export name=QuickLaunched2
echo run manifests/new_quick_launch.rtm
resim run manifests/new_quick_launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export quick_launched_coin2=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
export creator_badge_id2="#$(grep -A 1 "ResAddr: ${creator_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
echo Quick launched ${quick_launched_coin2}, received creator badge ${creator_badge}:${creator_badge_id2} and $(increase_in_wallet ${quick_launched_coin2}) ${quick_launched_coin2}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export minute=0
export hour="*"
export day_of_month="*"
export month="*"
export day_of_week="*"
export random_delay=1800
export xrd_amount=100
echo resim call-method ${timer_component} new_task "$minute" "$hour" "$day_of_month" "$month" "$day_of_week" $random_delay DCA ${quick_launched_coin2} resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:$xrd_amount
resim call-method ${timer_component} new_task "$minute" "$hour" "$day_of_month" "$month" "$day_of_week" $random_delay DCA ${quick_launched_coin2} resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:$xrd_amount >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export timer_badge_number="$(grep -A 1 "ResAddr: ${timer_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)"
export timer_badge_id="#${timer_badge_number}#"
echo "Created a task in the Timer for the DCA hook on ${quick_launched_coin2}, received timer badge ${timer_badge}:#${timer_badge_id}#"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export coin1=${quick_launched_coin1}
export coin1_amount=2
export coin1_per_buy_operation=1
export max_price=3
export min_interval_buy_operations=1800
echo resim run manifests/dca_new_task.rtm
resim run manifests/dca_new_task.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Created a task in the DCA hook associated with the Timer task and deposited ${coin1_amount} ${quick_launched_coin1} in it to buy ${quick_launched_coin2}, it will spend ${coin1_per_buy_operation} ${quick_launched_coin1} at a time
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export unix_epoch=1800000000
date=$(date -u -d @${unix_epoch} +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $unix_epoch

echo
echo resim call-method ${timer_component} alarm_clock ${timer_badge_number} --proofs ${alarm_clock_badge}:1
resim call-method ${timer_component} alarm_clock ${timer_badge_number} --proofs ${alarm_clock_badge}:1 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Invoked the Timer component to execute the DCA hook
grep 'Transaction Cost: ' $OUTPUTFILE

echo
echo resim call-method ${timer_component} alarm_clock ${timer_badge_number} --proofs ${alarm_clock_badge}:1
resim call-method ${timer_component} alarm_clock ${timer_badge_number} --proofs ${alarm_clock_badge}:1 >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo New invocation failed because not enough time has passed since the first one

echo
update_wallet_amounts
export bought_coins_only=true
echo resim run manifests/dca_withdraw.rtm
resim run manifests/dca_withdraw.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo "Called the withdraw method of the Dca component with bought_coins_only parameter = ${bought_coins_only}, $(increase_in_wallet ${quick_launched_coin2}) ${quick_launched_coin2} received"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export unix_epoch=$(($unix_epoch + $min_interval_buy_operations))
date=$(date -u -d @${unix_epoch} +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $unix_epoch

echo
echo resim call-method ${timer_component} alarm_clock ${timer_badge_number} --proofs ${alarm_clock_badge}:1
resim call-method ${timer_component} alarm_clock ${timer_badge_number} --proofs ${alarm_clock_badge}:1 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Invoked the Timer component to execute the DCA hook
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export min_interval_buy_operations=900
echo resim run manifests/dca_update_task.rtm
resim run manifests/dca_update_task.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Updated min_interval_buy_operations to ${min_interval_buy_operations}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export unix_epoch=$(($unix_epoch + $min_interval_buy_operations))
date=$(date -u -d @${unix_epoch} +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $unix_epoch

echo
echo resim call-method ${timer_component} alarm_clock ${timer_badge_number} --proofs ${alarm_clock_badge}:1
resim call-method ${timer_component} alarm_clock ${timer_badge_number} --proofs ${alarm_clock_badge}:1 >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo New invocation failed because the Dca component run out of ${quick_launched_coin1}

echo
echo resim run manifests/dca_add_funds.rtm
resim run manifests/dca_add_funds.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Deposited additional ${quick_launched_coin1} in the Dca component
grep 'Transaction Cost: ' $OUTPUTFILE

echo
echo resim call-method ${timer_component} alarm_clock ${timer_badge_number} --proofs ${alarm_clock_badge}:1
resim call-method ${timer_component} alarm_clock ${timer_badge_number} --proofs ${alarm_clock_badge}:1 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Invoked the Timer component to execute the DCA hook
grep 'Transaction Cost: ' $OUTPUTFILE

echo
update_wallet_amounts
export bought_coins_only=false
echo resim run manifests/dca_withdraw.rtm
resim run manifests/dca_withdraw.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo "Called the withdraw method of the Dca component with bought_coins_only parameter = ${bought_coins_only}, $(increase_in_wallet ${quick_launched_coin1}) ${quick_launched_coin1} and $(increase_in_wallet ${quick_launched_coin2}) ${quick_launched_coin2} received"
grep 'Transaction Cost: ' $OUTPUTFILE

