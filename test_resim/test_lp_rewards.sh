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
export flash_loan_nft=$(grep 'Resource:' $OUTPUTFILE | head -n 2 | tail -n 1 | cut -d ' ' -f 3)
export proxy_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 4 | tail -n 1 | cut -d ' ' -f 3)
export integrator_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 5 | tail -n 1 | cut -d ' ' -f 3)
echo -e "RadixPump component: ${radix_pump_component}\nCreator badge: ${creator_badge}\nFlash loan transient NFT: ${flash_loan_nft}\nProxy badge: ${proxy_badge}\n\nIntegrator badge: ${integrator_badge}"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
resim publish ../hooks/lp_rewards >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export hooks_package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo Hooks package: ${hooks_package}

echo
export grace_period=604800 # One week
echo resim call-function ${hooks_package} LpRewardsHook new ${owner_badge} ${proxy_badge} ${creator_badge} ${grace_period}
resim call-function ${hooks_package} LpRewardsHook new ${owner_badge} ${proxy_badge} ${creator_badge} ${grace_period} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export lp_rewards_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
echo LpRewardsHook component: ${lp_rewards_component}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export hook_name=LpRewards
export test_hook_component=${lp_rewards_component}
export operations='"PostAddLiquidity", "PostRemoveLiquidity"'
echo resim run register_hook.rtm
resim run register_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Registered hook ${hook_name} for operations ${operations}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export globally_enabled_operations='"PostAddLiquidity"'
echo resim run owner_enable_hook.rtm
resim run owner_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Globally enabled hook ${hook_name} for operation ${globally_enabled_operations}
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
export buy_pool_fee=0.1
export sell_pool_fee=0.1
export flash_loan_pool_fee=1
echo run new_quick_launch.rtm
resim run new_quick_launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export quick_launched_coin=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
export lp_quick=$(grep 'Resource:' $OUTPUTFILE | tail -n 1 | cut -d ' ' -f 3)
export creator_badge_id="#$(grep -A 1 "ResAddr: ${creator_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
export quick_launched_coin_received=$(increase_in_wallet ${quick_launched_coin})
echo Quick launched ${quick_launched_coin}, received ${quick_launched_coin_received}
echo Creator badge id: ${creator_badge_id}
echo LP token: ${lp_quick}

echo
echo resim new-token-fixed --name RewardCoin --symbol RW $supply
resim new-token-fixed --name RewardCoin --symbol RW $supply >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export reward_coin=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
echo Created RewardCoin ${reward_coin}

echo
export enabled_operations='"PostRemoveLiquidity"'
echo resim run creator_enable_hook.rtm
resim run creator_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Enabled hook ${hook_name} for operations ${enabled_operations} on ${quick_launched_coin}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export unix_epoch=1800000000
date=$(date -u -d @${unix_epoch} +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $unix_epoch

echo
export base_coin_amount=${minimum_deposit}
export quick_coin_amount=$(echo ${quick_launched_coin_received} / 2 | bc)
echo resim call-method ${radix_pump_component} add_liquidity ${base_coin}:${base_coin_amount} ${quick_launched_coin}:${quick_coin_amount}
resim call-method ${radix_pump_component} add_liquidity ${base_coin}:${base_coin_amount} ${quick_launched_coin}:${quick_coin_amount} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export lp_id1="#$(grep -A 1 "ResAddr: ${lp_quick}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
echo Added ${quick_coin_amount} ${quick_launched_coin} to the pool, LP id ${lp_id1} received

echo
echo resim call-method ${radix_pump_component} add_liquidity ${base_coin}:${base_coin_amount} ${quick_launched_coin}:${quick_coin_amount}
resim call-method ${radix_pump_component} add_liquidity ${base_coin}:${base_coin_amount} ${quick_launched_coin}:${quick_coin_amount} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export lp_id2="#$(grep -A 1 "ResAddr: ${lp_quick}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
echo Added ${quick_coin_amount} ${quick_launched_coin} to the pool, LP id ${lp_id2} received

echo
export reward_coin_amount=$((${supply} / 2))
export start_time=${unix_epoch}
export end_time=1800172800
export daily_reward_per_coin=10
echo resim run new_liquidity_campaign.rtm
resim run new_liquidity_campaign.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Created liquidity campaign: ${daily_reward_per_coin} ${reward_coin} per coin per day

echo
export unix_epoch=1800086400
date=$(date -u -d @${unix_epoch} +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $unix_epoch

echo
update_wallet_amounts
export lp_token=${lp_quick}
export lp_id=${lp_id1}
echo resim run get_rewards.rtm
resim run get_rewards.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Withdraw rewards without removing liquidity for LP token ${lp_id}, $(increase_in_wallet ${reward_coin}) ${reward_coin} received

echo
export unix_epoch=1800691200
date=$(date -u -d @${unix_epoch} +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $unix_epoch

echo
update_wallet_amounts
echo resim call-method ${radix_pump_component} remove_liquidity ${lp_quick}:${lp_id1}
resim call-method ${radix_pump_component} remove_liquidity ${lp_quick}:${lp_id1} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Removed liquidity using LP token ${lp_id1}, received $(increase_in_wallet ${reward_coin}) ${reward_coin}

echo
update_wallet_amounts
echo resim call-method ${radix_pump_component} remove_liquidity ${lp_quick}:${lp_id2}
resim call-method ${radix_pump_component} remove_liquidity ${lp_quick}:${lp_id2} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Removed liquidity using LP token ${lp_id2}, received $(increase_in_wallet ${reward_coin}) ${reward_coin}

echo
echo resim run terminate_liquidity_campaign.rtm
resim run terminate_liquidity_campaign.rtm >$OUTPUTFILE && ( echo "This transaction was supposed to fail!" ; cat $OUTPUTFILE ; exit 1 )
echo Transcation failed because the coin creator tried to terminate the campaign too soon

echo
export unix_epoch=$(($end_time + $grace_period))
date=$(date -u -d @${unix_epoch} +"%Y-%m-%dT%H:%M:%SZ")
resim set-current-time $date
echo Date is now $unix_epoch

echo
update_wallet_amounts
echo resim run terminate_liquidity_campaign.rtm
resim run terminate_liquidity_campaign.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Coin creator terminated liquidity campaign, received back $(increase_in_wallet ${reward_coin}) ${reward_coin}

