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
echo -e "RadixPump component: ${radix_pump_component}\nCreator badge: ${creator_badge}\nFlash loan transient NFT: ${flash_loan_nft}\nProxy badge: ${proxy_badge}\n\nIntegrator badge: ${integrator_badge}"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
resim publish ../hooks/ath_club >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export ath_club_package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo ATH Club package: ${ath_club_package}

echo
echo resim call-function ${ath_club_package} AthClubHook new ${owner_badge} ${proxy_badge} ${creator_badge} "https://athvodka.com/cdn/shop/files/ATHVODKA_SECONDARY-LOGO_01_1024x.png"
resim call-function ${ath_club_package} AthClubHook new ${owner_badge} ${proxy_badge} ${creator_badge} "https://athvodka.com/cdn/shop/files/ATHVODKA_SECONDARY-LOGO_01_1024x.png" >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export ath_club_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
export ath_club_nft=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
echo AthClubHook component: ${ath_club_component}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export hook_name="ATH Club"
export test_hook_component=${ath_club_component}
export operations='"Buy"'
echo resim run manifests/register_hook.rtm
resim run manifests/register_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Registered hook ${hook_name} for operations ${operations}
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
export supply=2000000
export price=10
export buy_pool_fee=0.01
export sell_pool_fee=0.01
export flash_loan_pool_fee=0.01
echo run manifests/new_quick_launch.rtm
resim run manifests/new_quick_launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export quick_launched_coin=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
export quick_launched_coin_received=$(increase_in_wallet ${quick_launched_coin})
export creator_badge_id="#$(grep -A 1 "ResAddr: ${creator_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
echo Quick launched ${quick_launched_coin}, received ${quick_launched_coin_received}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export enabled_operations='"Buy"'
echo resim run manifests/creator_enable_hook.rtm
resim run manifests/creator_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Enabled hook ${hook_name} for operations ${enabled_operations} on ${quick_launched_coin}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
update_wallet_amounts
export integrator_id=0
export base_coin_amount=20
echo resim call-method ${radix_pump_component} swap ${base_coin}:${base_coin_amount} ${quick_launched_coin} ${integrator_id}
resim call-method ${radix_pump_component} swap ${base_coin}:${base_coin_amount} ${quick_launched_coin} ${integrator_id} >$OUTPUTFILE
export bought_coins=$(increase_in_wallet ${quick_launched_coin})
export ath_club_nft_id="#$(grep -A 1 "ResAddr: ${ath_club_nft}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
echo "Bought ${bought_coins} ${quick_launched_coin}, received the ${ath_club_nft_id} ATH Club NFT"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

echo
update_wallet_amounts
echo resim call-method ${radix_pump_component} swap ${quick_launched_coin}:${bought_coins} ${base_coin} ${integrator_id}
resim call-method ${radix_pump_component} swap ${quick_launched_coin}:${bought_coins} ${base_coin} ${integrator_id}>$OUTPUTFILE
echo "Sold ${bought_coins} ${quick_launched_coin}, received $(increase_in_wallet ${base_coin}) base coin"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

echo
update_wallet_amounts
echo resim call-method ${radix_pump_component} swap ${base_coin}:$((${base_coin_amount} / 2)) ${quick_launched_coin} ${integrator_id}
resim call-method ${radix_pump_component} swap ${base_coin}:$((${base_coin_amount} / 2)) ${quick_launched_coin} ${integrator_id} >$OUTPUTFILE
export bought_coins=$(increase_in_wallet ${quick_launched_coin})
export ath_club_nft_id="#$(grep -A 1 "ResAddr: ${ath_club_nft}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#" || true
echo "Bought ${bought_coins} ${quick_launched_coin}, received the ${ath_club_nft_id} ATH Club NFT (should be none)"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

echo
update_wallet_amounts
echo resim call-method ${radix_pump_component} swap ${base_coin}:${base_coin_amount} ${quick_launched_coin} ${integrator_id}
resim call-method ${radix_pump_component} swap ${base_coin}:${base_coin_amount} ${quick_launched_coin} ${integrator_id} >$OUTPUTFILE
export bought_coins=$(increase_in_wallet ${quick_launched_coin})
export ath_club_nft_id="#$(grep -A 1 "ResAddr: ${ath_club_nft}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
echo "Bought ${bought_coins} ${quick_launched_coin}, received the ${ath_club_nft_id} ATH Club NFT"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
get_pool_info ${quick_launched_coin}

