#!/bin/bash

OUTPUTFILE=$(mktemp)

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
export minimum_deposit=10
export creation_fee_percentage=0.1
export buy_sell_fee_percentage=0.1
export flash_loan_fee=1
echo resim call-function ${radix_pump_package} RadixPump new ${owner_badge} ${base_coin} ${minimum_deposit} ${creation_fee_percentage} ${buy_sell_fee_percentage} ${flash_loan_fee} ${account}
resim call-function ${radix_pump_package} RadixPump new ${owner_badge} ${base_coin} ${minimum_deposit} ${creation_fee_percentage} ${buy_sell_fee_percentage} ${flash_loan_fee} ${account} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export radix_pump_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
export creator_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
export flash_loan_nft=$(grep 'Resource:' $OUTPUTFILE | head -n 2 | tail -n 1 | cut -d ' ' -f 3)
export proxy_badge=$(grep 'Resource:' $OUTPUTFILE | head -n 4 | tail -n 1 | cut -d ' ' -f 3)
echo -e "RadixPump component: ${radix_pump_component}\nCreator badge: ${creator_badge}\nFlash loan transient NFT: ${flash_loan_nft}\nProxy badge: ${proxy_badge}"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
resim publish ../hooks/ape_in >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export hooks_package=$(grep 'Success! New Package:' $OUTPUTFILE | cut -d ' ' -f 4)
echo Hooks package: ${hooks_package}

echo
export launches_per_buyer=85
export base_coins_per_launch=1
echo resim call-function ${hooks_package} ApeInHook new ${owner_badge} ${hooks_badge} ${proxy_coin} ${launches_per_buyer} ${base_coins_per_launch}
resim call-function ${hooks_package} ApeInHook new ${owner_badge} ${proxy_badge} ${base_coin} ${launches_per_buyer} ${base_coins_per_launch} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
export ape_in_hook_component=$(grep 'Component:' $OUTPUTFILE | cut -d ' ' -f 3)
export buyer_badge=$(grep 'Resource:' $OUTPUTFILE | cut -d ' ' -f 3)
echo -e "ApeInHook component: ${ape_in_hook_component}"
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export hook_name=ApeIn
export test_hook_component=${ape_in_hook_component}
export operations='"QuickLaunch"'
echo resim run manifests/register_hook.rtm
resim run manifests/register_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Registered hook ${hook_name} for operation ${operations}
grep 'Transaction Cost: ' $OUTPUTFILE

echo
export globally_enabled_operations='"QuickLaunch"'
echo resim run manifests/owner_enable_hook.rtm
resim run manifests/owner_enable_hook.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
echo Globally enabled hook ${hook_name} for operation ${globally_enabled_operations}
grep 'Transaction Cost: ' $OUTPUTFILE

export base_coin_amount=${minimum_deposit}
export icon=https://media-cdn.tripadvisor.com/media/photo-s/1a/ce/31/66/photo-de-profil.jpg
export description="Quick launched coin"
export info_url=""
export social_url='Array<String>()'
export supply=1000000
export price=10
export buy_pool_fee=0.1
export sell_pool_fee=0.1
export flash_loan_pool_fee=1
export ape_in_deposit=$(($launches_per_buyer * $base_coins_per_launch))
export iterations=90
for I in $(seq ${iterations})
do
    echo
    export symbol=QL${I}
    export name=QuickLaunchedCoin${I}
    echo run manifests/new_quick_launch.rtm
    resim run manifests/new_quick_launch.rtm >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
    export quick_launched_coin=$(grep 'Resource:' $OUTPUTFILE | head -n 1 | cut -d ' ' -f 3)
    export bought=$(grep -A 8 BuyEvent $OUTPUTFILE | grep amount: | cut -d '"' -f 2)
    if [ -n "$bought" ]
    then
        echo Quick launched ${quick_launched_coin}, someone aped in $bought coins
    else
        echo Quick launched ${quick_launched_coin}
    fi
    grep 'Transaction Cost: ' $OUTPUTFILE

    echo
    echo resim call-method ${ape_in_hook_component} ape_in ${base_coin}:${ape_in_deposit}
    resim call-method ${ape_in_hook_component} ape_in ${base_coin}:${ape_in_deposit} >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
    export buyer_badge_id="#$(grep -A 1 "ResAddr: ${buyer_badge}" $OUTPUTFILE | tail -n 1 | cut -d '#' -f 2)#"
    echo Deposited ${ape_in_deposit} ${base_coin} in ApeInHook, received buyer badge ${buyer_badge_id}
    grep 'Transaction Cost: ' $OUTPUTFILE
done

for I in $(seq ${iterations})
do
    echo
    echo resim call-method ${ape_in_hook_component} withdraw_coins "${buyer_badge}:#$I#"
    resim call-method ${ape_in_hook_component} withdraw_coins "${buyer_badge}:#$I#" >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
    export moved_coins=$(grep ResAddr: $OUTPUTFILE | cut -d ' ' -f 5 | sort | uniq | wc -l)
    echo "Redeemed buyer badge #$I#, the transaction moved ${moved_coins} different resources"
    grep 'Transaction Cost: ' $OUTPUTFILE

    if [ ${moved_coins} -ge 80 ]
    then
        resim call-method ${ape_in_hook_component} withdraw_coins "${buyer_badge}:#$I#" >$OUTPUTFILE || ( cat $OUTPUTFILE ; exit 1 )
	export moved_coins=$(grep ResAddr: $OUTPUTFILE | cut -d ' ' -f 5 | sort | uniq | wc -l)
	echo "Done it again, the transaction moved ${moved_coins} different resources"
    	grep 'Transaction Cost: ' $OUTPUTFILE
    fi
done

