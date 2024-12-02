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

get_pool_info () {
  echo PoolInfo for $1
  resim call-method $radix_pump_component get_pool_info $1 |
    grep -A 100 '├─ Tuple(' | (
      read x
      read x
      read base_coin_amount
      echo base_coin_amount: $(echo $base_coin_amount | cut -d '"' -f 2)
      read coin_amount
      echo coin_amount: $(echo $coin_amount | cut -d '"' -f 2)
      read last_price
      echo last_price: $(echo $last_price | cut -d '"' -f 2)
      read price
      echo price: $(echo $price | cut -d '"' -f 2)
      read circulating_supply
      echo circulating_supply: $(echo $circulating_supply | cut -d '"' -f 2)
      read total_buy_fee_percentage
      echo total_buy_fee_percentage: $(echo $total_buy_fee_percentage | cut -d '"' -f 2)
      read total_sell_fee_percentage
      echo total_sell_fee_percentage: $(echo $total_sell_fee_percentage | cut -d '"' -f 2)
      read total_flash_loan_fee
      echo total_flash_loan_fee: $(echo $total_flash_loan_fee | cut -d '"' -f 2)
      read pool_mode
      case $pool_mode in 
        'Enum::[0],') echo pool_mode: WaitingForLaunch ;;
        'Enum::[1],') echo pool_mode: Launching ;;
        'Enum::[2],') echo pool_mode: TerminatingLaunch ;;
        'Enum::[3],') echo pool_mode: Normal ;;
        'Enum::[4],') echo pool_mode: Liquidation ;;
        'Enum::[5],') echo pool_mode: Uninitialised ;;
      esac
      read lp_resource_address
      read coin_lp_ratio
      echo coin_lp_ratio : $(echo $coin_lp_ratio | cut -d '"' -f 2)
      read end_launch_time
      if [ "$end_launch_time" = "Enum::[1](" ]
      then
	read end_launch_time
	read x
	echo end_launch_time: $end_launch_time
      fi
      read unlocking_time
      if [ "$unlocking_time" = "Enum::[1](" ]
      then
	read unlocking_time
	read x
	echo unlocking_time: $unlocking_time
      fi
      read initial_locked_amount
      if [ "$initial_locked_amount" = "Enum::[1](" ]
      then
	read initial_locked_amount
	read x
	echo initial_locked_amount: $(echo $initial_locked_amount | cut -d '"' -f 2)
      fi
      read unlocked_amount
      if [ "$unlocked_amount" = "Enum::[1](" ]
      then
	read unlocked_amount
	read x
	echo unlocked_amount: $(echo $unlocked_amount | cut -d '"' -f 2)
      fi
      read ticket_price
      if [ "$ticket_price" = "Enum::[1](" ]
      then
	read ticket_price
	read x
	echo ticket_price: $(echo $ticket_price | cut -d '"' -f 2)
      fi
      read winning_tickets
      if [ "$winning_tickets" = "Enum::[1](" ]
      then
        read winning_tickets
	read x
        echo winning_tickets: $winning_tickets
      fi
      read coins_per_winning_ticket
      if [ "$coins_per_winning_ticket" = "Enum::[1](" ]
      then
	read coins_per_winning_ticket
	read x
	echo coins_per_winning_ticket: $(echo $coins_per_winning_ticket | cut -d '"' -f 2)
      fi
    )
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
export flash_loan_nft=$(grep 'Resource:' $OUTPUTFILE | head -n 2 | tail -n 1 | cut -d ' ' -f 3)
export proxy_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 4 | tail -n 1 | cut -d ' ' -f 3)
export integrator_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 5 | tail -n 1 | cut -d ' ' -f 3)
echo -e "RadixPump component: ${radix_pump_component}\nCreator badge: ${creator_badge}\nFlash loan transient NFT: ${flash_loan_nft}\nProxy badge: ${proxy_badge}\nIntegrator badge: ${integrator_badge}"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
resim publish ../hooks/test_hooks >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export hooks_package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo Hooks package: ${hooks_package}

echo
echo resim call-function ${hooks_package} TestHook0 new ${owner_badge} ${proxy_badge} ${base_coin}:100
resim call-function ${hooks_package} TestHook0 new ${owner_badge} ${proxy_badge} ${base_coin}:100 >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export test_hook0_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
echo -e "TestHook0 component: ${test_hook0_component}"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export hook_name=TestHook0
export test_hook_component=${test_hook0_component}
export operations='"Buy"'
echo resim run manifests/register_hook.rtm
resim run manifests/register_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Registered hook ${hook_name} for operations ${operations}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
echo resim call-function ${hooks_package} TestHook1 new ${owner_badge} ${proxy_badge}
resim call-function ${hooks_package} TestHook1 new ${owner_badge} ${proxy_badge} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export test_hook1_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
export test_hook1_coin=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
echo -e "TestHook1 component: ${test_hook1_component}\nTestHook2 coin: ${test_hook1_coin}"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export hook_name=TestHook1
export test_hook_component=${test_hook1_component}
export operations='"Buy"'
echo resim run manifests/register_hook.rtm
resim run manifests/register_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Registered hook ${hook_name} for operations ${operations}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
echo resim call-function ${hooks_package} TestHook2 new ${owner_badge} ${proxy_badge}
resim call-function ${hooks_package} TestHook2 new ${owner_badge} ${proxy_badge} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export test_hook2_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
export test_hook2_coin=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
echo -e "TestHook2 component: ${test_hook2_component}\nTestHook2 coin: ${test_hook2_coin}"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export hook_name=TestHook2
export test_hook_component=${test_hook2_component}
export operations='"FairLaunch", "TerminateFairLaunch", "QuickLaunch", "RandomLaunch", "TerminateRandomLaunch", "Buy", "Sell", "ReturnFlashLoan", "BuyTicket", "RedeemWinningTicket", "RedeemLosingTicket", "AddLiquidity", "RemoveLiquidity"'
echo resim run manifests/register_hook.rtm
resim run manifests/register_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Registered hook ${hook_name} for operations ${operations}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export globally_enabled_operations='"FairLaunch", "TerminateFairLaunch", "QuickLaunch", "RandomLaunch"'
echo resim run manifests/owner_enable_hook.rtm
resim run manifests/owner_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Globally enabled hook ${hook_name} for operations ${globally_enabled_operations}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export globally_disabled_operations='"FairLaunch"'
echo resim run manifests/owner_disable_hook.rtm
resim run manifests/owner_disable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Globally disabled hook ${hook_name} for operations ${globally_disabled_operations}
grep 'Transaction Cost: ' $OUTPUTFILE

export integrator_id=0

echo
export min_launch_duration=604800
export min_lock_duration=5184000
echo "resim call-method ${radix_pump_component} update_time_limits $min_launch_duration $min_lock_duration --proofs ${owner_badge}:${owner_badge_id}"
resim call-method ${radix_pump_component} update_time_limits $min_launch_duration $min_lock_duration --proofs ${owner_badge}:${owner_badge_id} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Set limits min_launch_duration: 604800 min_lock_duration: 5184000
grep 'Transaction Cost: ' $OUTPUTFILE

echo
update_wallet_amounts
export symbol=RL
export name=RandomLaunchedCoin
export icon=https://img.evients.com/images/f480x480/e7/b5/09/51/e7b50951be9149fe86e26f45e019d2af.jpg
export description="Random launched coin"
export social_url='Array<String>()'
export info_url=""
export ticket_price=10
export winning_tickets=30
export coins_per_winning_ticket=10
export buy_pool_fee=5
export sell_pool_fee=0.1
export flash_loan_pool_fee=1
echo resim run manifests/new_random_launch.rtm
resim run manifests/new_random_launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export random_ticket=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
export random_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 2 | tail -n 1 | cut -d ' ' -f 3)
export random_launched_coin=$(grep 'Resource:' $OUTPUTFILE | head -n 3 | tail -n 1 | cut -d ' ' -f 3)
export lp_random=$(grep 'Resource:' $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 3)
export creator_badge_id="#$(grep -A 1 "ResAddr: ${creator_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
echo Random launch coin created $random_launched_coin
echo LP token: $lp_random
echo ticket: $random_ticket
echo Random badge: $random_badge
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${random_launched_coin}

echo
export payment=1000
echo resim call-method ${radix_pump_component} swap ${base_coin}:$payment ${random_launched_coin} ${integrator_id}
resim call-method ${radix_pump_component} swap ${base_coin}:$payment ${random_launched_coin} ${integrator_id} >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Someone tried to buy ${random_launched_coin} before it was launched, the transaction failed

echo
update_wallet_amounts
export end_launch_time=$(($unix_epoch + $min_launch_duration))
export unlocking_time=$(($unix_epoch + $min_launch_duration + $min_lock_duration))
echo resim run manifests/launch.rtm
resim run manifests/launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Random sale launched for ${random_launched_coin}, received $(increase_in_wallet ${random_launched_coin})
echo TestHook2 coin received: $(increase_in_wallet ${test_hook2_coin})
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${random_launched_coin}

echo
export amount=50
export payment=$(echo "${ticket_price} * ${amount} - 0.0000001" | bc)
echo resim call-method ${radix_pump_component} buy_ticket ${random_launched_coin} $amount ${base_coin}:$payment
resim call-method ${radix_pump_component} buy_ticket ${random_launched_coin} $amount ${base_coin}:$payment >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Failed attempt to buy one ticket without paying ticket_price + total_buy_fee

echo
export buy_pool_fee_percentage=0.1
export sell_pool_fee_percentage=0.1
export flash_loan_pool_fee=2
echo resim run manifests/update_pool_fees.rtm
resim run manifests/update_pool_fees.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Failed attempt to update fees during launch phase, this is not allowed

echo
update_wallet_amounts
export bought_tickets1=20
export payment=$(echo "${ticket_price} * ${bought_tickets1}" | bc)
echo resim call-method ${radix_pump_component} buy_ticket ${random_launched_coin} ${bought_tickets1} ${base_coin}:$payment
resim call-method ${radix_pump_component} buy_ticket ${random_launched_coin} ${bought_tickets1} ${base_coin}:$payment >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Bought $(increase_in_wallet ${random_ticket}) tickets for $payment $base_coin
echo TestHook2 coin received: $(increase_in_wallet ${test_hook2_coin})
grep 'Transaction Cost: ' $OUTPUTFILE

echo
update_wallet_amounts
export bought_tickets2=30
export payment=$(echo "${ticket_price} * ${bought_tickets2}" | bc)
echo resim call-method ${radix_pump_component} buy_ticket ${random_launched_coin} ${bought_tickets2} ${base_coin}:$payment
resim call-method ${radix_pump_component} buy_ticket ${random_launched_coin} ${bought_tickets2} ${base_coin}:$payment >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Bought $(increase_in_wallet ${random_ticket}) tickets for $payment $base_coin
echo TestHook2 coin received: $(increase_in_wallet ${test_hook2_coin})
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${random_launched_coin}

echo
unix_epoch=$(($unix_epoch + 604800))
date=$(date -u -d @$unix_epoch +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo resim run manifests/terminate_launch.rtm
resim run manifests/terminate_launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Random launch termination started for ${random_launched_coin}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${random_launched_coin}

echo
export random1=$RANDOM
export random2=$RANDOM
export random3=$RANDOM
export random4=$RANDOM
echo resim call-method ${random_component} do_callback $(($random1 * $random2 * $random3 * $random4))
resim call-method ${random_component} do_callback $(($random1 * $random2 * $random3 * $random4)) >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Called the do_callback method of the random component

echo
update_wallet_amounts
echo resim run manifests/terminate_launch.rtm
resim run manifests/terminate_launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Random launch termination completed for ${random_launched_coin}
echo $(increase_in_wallet ${base_coin}) ${base_coin} received
echo TestHook2 coin received: $(increase_in_wallet ${test_hook2_coin})
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${random_launched_coin}

echo
export creation_fee_percentage=0.2
export buy_sell_fee_percentage=0.2
export flash_loan_fee=2
export max_buy_sell_pool_fee_percentage=5
export minimum_deposit=10000
echo resim call-method ${radix_pump_component} update_fees ${creation_fee_percentage} ${buy_sell_fee_percentage} ${flash_loan_fee} ${max_buy_sell_pool_fee_percentage} ${minimum_deposit} --proofs ${owner_badge}:${owner_badge_id}
resim call-method ${radix_pump_component} update_fees ${creation_fee_percentage} ${buy_sell_fee_percentage} ${flash_loan_fee} ${max_buy_sell_pool_fee_percentage} ${minimum_deposit} --proofs ${owner_badge}:${owner_badge_id} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Updated platform fees 
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${random_launched_coin}

echo
export buy_pool_fee_percentage=0.1
export sell_pool_fee_percentage=0.2
export flash_loan_pool_fee=3
echo resim run manifests/update_pool_fees.rtm
resim run manifests/update_pool_fees.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Transaction failed because the creator tried to increase sell_pool_fee_percentage

echo
export buy_pool_fee_percentage=0.1
export sell_pool_fee_percentage=0.1
export flash_loan_pool_fee=3
echo resim run manifests/update_pool_fees.rtm
resim run manifests/update_pool_fees.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Updated ${random_launched_coin} pool fees
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${random_launched_coin}

echo
export enabled_operations='"RedeemLosingTicket"'
echo resim run manifests/creator_enable_hook.rtm
resim run manifests/creator_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Enabled hook ${hook_name} for operations ${enabled_operations} on ${random_launched_coin}
grep 'Transaction Cost: ' $OUTPUTFILE

resim show | grep -A $((${bought_tickets1} + ${bought_tickets2})) ${random_ticket} | grep -v ${random_ticket} | while read x ticket_id
do
  echo
  update_wallet_amounts
  echo resim call-method ${radix_pump_component} redeem_ticket ${random_ticket}:${ticket_id}
  resim call-method ${radix_pump_component} redeem_ticket ${random_ticket}:${ticket_id} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
  echo Ticket ${ticket_id} redeemed
  echo $(increase_in_wallet ${random_launched_coin}) ${random_launched_coin} received
  echo $(increase_in_wallet ${base_coin}) ${base_coin} received
  echo TestHook2 coin received: $(increase_in_wallet ${test_hook2_coin})
  grep 'Transaction Cost: ' $OUTPUTFILE
done

echo
get_pool_info ${random_launched_coin}

echo
update_wallet_amounts
export coin=${random_launched_coin}
export loan_amount=9
export sell_amount=9
export fee=$(($flash_loan_fee + $flash_loan_pool_fee))
echo resim run manifests/flash_loan_attack_sell.rtm
resim run manifests/flash_loan_attack_sell.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo "Tried to manipulate price via flash loan: get flash loan -> sell when the pool has few coins (so the price should be high) -> return loan -> buy"
echo "${random_launched_coin} variation in wallet: $(increase_in_wallet ${random_launched_coin}) (if negative the attack failed)"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${random_launched_coin}

echo
update_wallet_amounts
export coin=${random_launched_coin}
export coin_amount=9
export base_coin_amount=12.149159002 # coin_amount x last_price
export loan_amount=9
export lp=${lp_random}
export fee=$(($flash_loan_fee + $flash_loan_pool_fee))
echo resim run manifests/flash_loan_attack_lp.rtm
resim run manifests/flash_loan_attack_lp.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo "Tried to steal liquidity via flash loan: get flash loan -> add liquidity when the pool has few coins (so the share should be high) -> return loan -> withdraw liquidity"
echo "${random_launched_coin} variation in wallet: $(increase_in_wallet ${random_launched_coin})"
echo "${base_coin} variation in wallet: $(increase_in_wallet ${base_coin}) (if both are negative the attack failed)"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export hook_name=TestHook0
export globally_enabled_operations='"Buy"'
echo resim run manifests/owner_enable_hook.rtm
resim run manifests/owner_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Globally enabled hook ${hook_name} for operations ${globally_enabled_operations}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export hook_name=TestHook1
export globally_enabled_operations='"Buy"'
echo resim run manifests/owner_enable_hook.rtm
resim run manifests/owner_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Globally enabled hook ${hook_name} for operations ${globally_enabled_operations}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export hook_name=TestHook2
export globally_enabled_operations='"Buy"'
echo resim run manifests/owner_enable_hook.rtm
resim run manifests/owner_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Globally enabled hook ${hook_name} for operations ${globally_enabled_operations}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${random_launched_coin}

echo
update_wallet_amounts
export payment=1
echo resim call-method ${radix_pump_component} swap ${base_coin}:$payment ${random_launched_coin} ${integrator_id}
resim call-method ${radix_pump_component} swap ${base_coin}:$payment ${random_launched_coin} ${integrator_id} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo "Received $(increase_in_wallet ${random_launched_coin}) ${random_launched_coin} for $payment (+1 from TestHook0) ${base_coin}"
echo This should call once TestHook0 and TestHook1 and twice TestHook2
echo $(increase_in_wallet ${test_hook1_coin}) TestHook1 coin received
echo $(increase_in_wallet ${test_hook2_coin}) TestHook2 coin received
grep 'Transaction Cost: ' $OUTPUTFILE

