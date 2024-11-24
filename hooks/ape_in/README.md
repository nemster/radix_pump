# Ape in hook

This hook allows users to be the first to buy the next quick coins launched.

For this hook to work it must be enabled globally for the `QuickLaunch` operation by the component owner.

## Known bugs & limitations

A user can't choose the number of launches he will take part into and the amount of base coins he will use in each launch. These numbers are decided once for all by the component owner when the component is instantiated.  
Uses can't change their minds and stop buying before the decided number of launches happens.  

## Transaction manifests

### Instantiate

```
CALL_FUNCTION
    Address("")
    "ApeInHook"
    "new"
    Address("<OWNER_BADGE_ADDRESS>")
    Address("<PROXY_BADGE_ADDRESS>")
    Address("<BASE_COIN_ADDRESS>")
    <LAUNCHES_PER_BUYER>u16
    Decimal("<BASE_COINS_PER_LAUNCH>)
;
```

`<OWNER_BADGE_ADDRESS>` is the resource address of the component owner badge.  
`<PROXY_BADGE_ADDRESS>` is the resource address of the badge that RadixPump uses to authenticate against pools and hooks.  
`<BASE_COIN_ADDRESS>` must be the same base coin address as RadixPump.  
`<LAUNCHES_PER_BUYER>` is the number of the quick launches a user of this hook will take part.  
`<BASE_COINS_PER_LAUNCH>` is the number of base coins a user will spend for each launch.  

### ape_in

A user can call this method to deposit his base coins to buy the next quick launches.  
The number of coins he must deposit is fixed (`<LAUNCHES_PER_BUYER>` * `<BASE_COINS_PER_LAUNCH>`), any additional amount will be returned.  
This method returns a buyer badge that can be used to withdraw the bought coins.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("<BASE_COIN_ADDRESS>")
    Decimal("<BASE_COIN_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<BASE_COIN_ADDRESS>")
    Bucket("base_coin_bucket")
;
CALL_METHOD
    Address("<APE_IN_HOOK_COMPONENT>")
    "ape_in"
    Bucket("base_coin_bucket")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the user account address.  
`<BASE_COIN_ADDRESS>` must be the same base coin address as RadixPump.  
`<BASE_COIN_AMOUNT>` is the amount of base coins to buy the next launches; it must be at least `<LAUNCHES_PER_BUYER>` * `<BASE_COINS_PER_LAUNCH>`.  
`<APE_IN_HOOK_COMPONENT>` is the componet address of the Ape in hook.  

### withdraw_coins

Users can use this method to withdraw the bought coins by passing the buyer badge to the component.  
It is not possible to pass multiple buyer badges in a single invokation.   
This method can be invoked for a partial withdraw befour the fixed number of launches happened; in this case the buyer badge is returned back to the user, otherways it is burned.  
If the number of coins to withdraw is higher than 80, a partial withdraw happens even if all of the launches happened and the buyer bucket is returned to the uses that can invoke the method again in a new transaction.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw_non_fungibles"
    Address("<BUYER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("<BUYER_BADGE_ID>"))
;
TAKE_ALL_FROM_WORKTOP
    Address("<BUYER_BADGE_ADDRESS>")
    Bucket("buyer_badge_bucket")
;
CALL_METHOD
    Address("<APE_IN_HOOK_COMPONENT>")
    "withdraw_coins"
    Bucket("buyer_badge_bucket")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the user account address.  
`<BUYER_BADGE_ADDRESS>` is the resorce address of the buyer badge received when invoking the `ape_in` method.  
`<BUYER_BADGE_ID>` is the numeric id of the user's buyer badge.  

