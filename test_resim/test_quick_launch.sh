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
export forbidden_symbols='"XRD"'
echo resim run manifests/forbid_symbols.rtm
resim run manifests/forbid_symbols.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Symbols ${forbidden_symbols} forbidden
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export forbidden_names='"Radix"'
echo resim run manifests/forbid_names.rtm
resim run manifests/forbid_names.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Names ${forbidden_names} forbidden
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

echo
update_wallet_amounts
export base_coin_amount=${minimum_deposit}
export symbol=QL
export name=QuickLaunchedCoin
export icon=https://media-cdn.tripadvisor.com/media/photo-s/1a/ce/31/66/photo-de-profil.jpg
export description="Quick launched coin"
export info_url=""
export social_url='Array<String>()'
export supply=1000000
export price=10
export buy_pool_fee=1
export sell_pool_fee=1
export flash_loan_pool_fee=1
echo run manifests/new_quick_launch.rtm
resim run manifests/new_quick_launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export quick_launched_coin=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
export lp_quick=$(grep 'Resource:' $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 3)
export creator_badge_id="#$(grep -A 1 "ResAddr: ${creator_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
export collected_fees=$(echo "${base_coin_amount} * ${creation_fee_percentage} / 100" | bc)
echo Quick launched ${quick_launched_coin}, received $(increase_in_wallet ${quick_launched_coin})
echo Creator badge id: ${creator_badge_id}
echo LP token: ${lp_quick}
echo TestHook2 coin received: $(increase_in_wallet ${test_hook2_coin})
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

echo
export name=SameSymbolCoin
echo resim run manifests/new_quick_launch.rtm
resim run manifests/new_quick_launch.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to create a new coin with the same symbol and the transaction failed as expected

echo
export symbol=QL2
export name=QuickLaunchedCoin
echo resim run manifests/new_quick_launch.rtm
resim run manifests/new_quick_launch.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to create a new coin with the same name and the transaction failed as expected

echo
export symbol=XRD
export name=FakeRadix
echo resim run manifests/new_quick_launch.rtm
resim run manifests/new_quick_launch.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to create a new coin with XRD as symbol and the transaction failed as expected

echo
export symbol=XXX
export name=Radix
echo resim run manifests/new_quick_launch.rtm
resim run manifests/new_quick_launch.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to create a new coin with Radix as name and the transaction failed as expected

echo
export base_coin_amount=$((${minimum_deposit} - 1))
export name=YYY
echo resim run manifests/new_quick_launch.rtm
resim run manifests/new_quick_launch.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Tried to create a new coin with an insufficient base coin deposit and the transaction failed as expected

echo
export enabled_operations='"Buy", "Sell", "ReturnFlashLoan"'
echo resim run manifests/creator_enable_hook.rtm
resim run manifests/creator_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Enabled hook ${hook_name} for operations ${enabled_operations} on ${quick_launched_coin}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export disabled_operations='"Sell"'
echo resim run manifests/creator_disable_hook.rtm
resim run manifests/creator_disable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Disabled hook ${hook_name} for operations ${disabled_operations} on ${quick_launched_coin}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
update_wallet_amounts
export payment=1000
export integrator_id=0
echo resim call-method ${radix_pump_component} swap ${base_coin}:$payment ${quick_launched_coin} ${integrator_id}
resim call-method ${radix_pump_component} swap ${base_coin}:$payment ${quick_launched_coin} ${integrator_id} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export collected_fees=$(echo "${collected_fees} + $payment * ${buy_sell_fee_percentage} / 100" | bc)
echo Bought $(increase_in_wallet ${quick_launched_coin}) ${quick_launched_coin} for $payment ${base_coin}
echo $(increase_in_wallet ${test_hook2_coin}) TestHook2 coin received
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

echo
update_wallet_amounts
echo resim run manifests/owner_get_fees.rtm
resim run manifests/owner_get_fees.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo "The component owner withdrawed $(increase_in_wallet ${base_coin}) ${base_coin} in fees (should be about ${collected_fees} - transaction cost)"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
integrator_name="test"
echo resim run manifests/new_integrator.rtm
resim run manifests/new_integrator.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export integrator_id="$(grep -A 1 "ResAddr: ${integrator_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)"
export integrator_badge_id="#${integrator_id}#"
echo Integrator badge ${integrator_badge_id} minted
grep 'Transaction Cost: ' $OUTPUTFILE

echo
update_wallet_amounts
export payment=10
echo resim call-method ${radix_pump_component} swap ${quick_launched_coin}:$payment ${base_coin} ${integrator_id}
resim call-method ${radix_pump_component} swap ${quick_launched_coin}:$payment ${base_coin} ${integrator_id} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Sold $payment ${quick_launched_coin} for $(increase_in_wallet ${base_coin}) ${base_coin}
echo $(increase_in_wallet ${test_hook2_coin}) TestHook2 coin received
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

echo
export burn_amount=100000000
echo resim run manifests/burn.rtm
resim run manifests/burn.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo The creator tried to burn $burn_amount ${quick_launched_coin}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

echo
echo resim run manifests/burn.rtm
resim run manifests/burn.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo The creator tried to burn $burn_amount ${quick_launched_coin} but the transaction failed becaouse all of the excess coins have already been burned

echo
get_pool_info ${quick_launched_coin}

echo
update_wallet_amounts
export payment=1000
echo resim call-method ${radix_pump_component} swap ${base_coin}:$payment ${quick_launched_coin} ${integrator_id}
resim call-method ${radix_pump_component} swap ${base_coin}:$payment ${quick_launched_coin} ${integrator_id} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export collected_fees=$(echo "${collected_fees} + $payment * ${buy_sell_fee_percentage} / 100" | bc)
echo Bought $(increase_in_wallet ${quick_launched_coin}) ${quick_launched_coin} for $payment ${base_coin}
echo $(increase_in_wallet ${test_hook2_coin}) TestHook2 coin received
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

echo
echo resim call-method ${radix_pump_component} owner_set_liquidation_mode ${quick_launched_coin} --proofs ${owner_badge}:${owner_badge_id}
resim call-method ${radix_pump_component} owner_set_liquidation_mode ${quick_launched_coin} --proofs ${owner_badge}:${owner_badge_id} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Component owner set liquidation mode for ${quick_launched_coin}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

echo
update_wallet_amounts
export payment=1
echo resim call-method ${radix_pump_component} swap ${quick_launched_coin}:$payment ${base_coin} ${integrator_id}
resim call-method ${radix_pump_component} swap ${quick_launched_coin}:$payment ${base_coin} ${integrator_id} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Sold $payment ${quick_launched_coin} for $(increase_in_wallet ${base_coin}) ${base_coin}
echo $(increase_in_wallet ${test_hook2_coin}) TestHook2 coin received
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

echo
update_wallet_amounts
export payment=1
echo resim call-method ${radix_pump_component} swap ${quick_launched_coin}:$payment ${base_coin} ${integrator_id}
resim call-method ${radix_pump_component} swap ${quick_launched_coin}:$payment ${base_coin} ${integrator_id} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo "Sold $payment ${quick_launched_coin} for $(increase_in_wallet ${base_coin}) ${base_coin} (price should not have changed)"
echo $(increase_in_wallet ${test_hook2_coin}) TestHook2 coin received
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

echo
update_wallet_amounts
echo resim run manifests/integrator_get_fees.rtm
resim run manifests/integrator_get_fees.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Integrator ${integrator_id} withdrawed $(increase_in_wallet ${base_coin}) ${base_coin}
grep 'Transaction Cost: ' $OUTPUTFILE
