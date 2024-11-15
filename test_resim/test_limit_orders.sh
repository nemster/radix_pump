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
resim publish ../hooks/limit_buy >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export limit_buy_package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo Limit buy package: ${limit_buy_package}

echo
echo resim call-function ${limit_buy_package} LimitBuyHook new ${owner_badge} ${proxy_badge} ${base_coin} ${radix_pump_component}
resim call-function ${limit_buy_package} LimitBuyHook new ${owner_badge} ${proxy_badge} ${base_coin} ${radix_pump_component} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export limit_buy_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
export buy_order_nft=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
echo LimitBuyHook component: ${limit_buy_component}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export hook_name=LimitBuy
export test_hook_component=${limit_buy_component}
export operations='"PostSell"'
echo resim run register_hook.rtm
resim run register_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
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
export buy_pool_fee=0.1
export sell_pool_fee=0.1
export flash_loan_pool_fee=1
echo run new_quick_launch.rtm
resim run new_quick_launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export quick_launched_coin=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
export quick_launched_coin_received=$(increase_in_wallet ${quick_launched_coin})
export creator_badge_id="#$(grep -A 1 "ResAddr: ${creator_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
echo Quick launched ${quick_launched_coin}, received ${quick_launched_coin_received}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export enabled_operations='"PostSell"'
echo resim run creator_enable_hook.rtm
resim run creator_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Enabled hook ${hook_name} for operations ${enabled_operations} on ${quick_launched_coin}
grep 'Transaction Cost: ' $OUTPUTFILE

export orders=100
for I in $(seq ${orders})
do
    echo
    update_wallet_amounts
    export price=$(echo "scale=18; $RANDOM / 5461 + 5" | bc)
    export amount=$(echo "scale=18; $RANDOM / 1638 + 1" | bc)
    echo resim call-method ${limit_buy_component} new_order ${base_coin}:${amount} ${quick_launched_coin} ${price}
    resim call-method ${limit_buy_component} new_order ${base_coin}:${amount} ${quick_launched_coin} ${price} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
    export buy_order_nft_id="#$(grep -A 1 "ResAddr: ${buy_order_nft}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
    if [ "${buy_order_nft_id}" == "##" ]
    then
	echo Tried to insert an order at price: ${price}, base coin amount: ${amount}, received $(increase_in_wallet ${quick_launched_coin}) coins instead
    else
        echo Inserted order ${buy_order_nft_id}: price: ${price}, base coin amount: ${amount}
    fi
    grep 'Transaction Cost: ' $OUTPUTFILE
done

echo
export orders_nfts=$(resim show ${account} | grep LimitBuyOrder | cut -d ' ' -f 3)
echo Done ${orders} orders, some of them were matched immediately so there are only ${orders_nfts} LimitBuyOrder NFTs in the wallet

echo
export integrator_id=0
echo resim call-method ${radix_pump_component} swap ${quick_launched_coin}:${quick_launched_coin_received} ${base_coin} ${integrator_id} >$OUTPUTFILE
resim call-method ${radix_pump_component} swap ${quick_launched_coin}:${quick_launched_coin_received} ${base_coin} ${integrator_id} >$OUTPUTFILE
first=$(grep -n filled_orders_id $OUTPUTFILE | head -n 1 | cut -d : -f 1)
last=$(grep -n filled_orders_id $OUTPUTFILE | tail -n 1 | cut -d : -f 1)
export filled_orders=$(($last - $first - 2))
grep INFO $OUTPUTFILE
echo Sold ${quick_launched_coin_received} ${quick_launched_coin}, this triggered ${hook_name} that matched $filled_orders orders
grep 'Transaction Cost: ' $OUTPUTFILE

for I in $(seq ${orders_nfts})
do
    echo
    update_wallet_amounts
    if [ $RANDOM -gt 16384 ]
    then
        export coins_only=true
    else
        export coins_only=false
    fi
    echo resim call-method ${limit_buy_component} withdraw "${buy_order_nft}:#$I#" $coins_only
    resim call-method ${limit_buy_component} withdraw "${buy_order_nft}:#$I#" $coins_only >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
    echo "Withdraw of order id #$I# with option $coins_only, $(increase_in_wallet ${quick_launched_coin}) coins received, $(increase_in_wallet ${base_coin}) base coins received"
    grep 'Transaction Cost: ' $OUTPUTFILE
done

echo
export orders_nfts=$(resim show ${account} | grep LimitBuyOrder | cut -d ' ' -f 3)
echo There are still ${orders_nfts} LimitBuyOrder NFTs in the wallet

resim show $account | grep -A ${orders_nfts} LimitBuyOrder | grep -v LimitBuyOrder | cut -d ' ' -f 5 | while read buy_order_nft_id
do
    echo
    update_wallet_amounts
    echo resim call-method ${limit_buy_component} withdraw "${buy_order_nft}:${buy_order_nft_id}" false
    resim call-method ${limit_buy_component} withdraw "${buy_order_nft}:${buy_order_nft_id}" false >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
    echo "Withdraw of order id ${buy_order_nft_id} with option false, $(increase_in_wallet ${quick_launched_coin}) coins received, $(increase_in_wallet ${base_coin}) base coins received"
    grep 'Transaction Cost: ' $OUTPUTFILE
done

