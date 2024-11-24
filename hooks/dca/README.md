# Dca hook

This blueprint implements a RadixPump hook that can be used to DCA (dollar cost average) buy a coin; it is ment to be invoked by the Timer, not hooked to any RadixPump operation.  

## Known bugs and limitations

This hook doesn't mint a badge to identify users, it uses the timer badge for this purpouse; so, before interacting with this hook, users must create a task in the Timer referring to the coin they want to buy.  
This means that if the user deletes the Timer task he will be no longer able to withdraw his bought coins!  

There is no feedback when a buy operation succeeds; the only way the user has to know the state of his buy task is to simulate a `withdraw` operation (eventually abort it if not satisfied).  

## Transaction manifests

### Instantiation

Use this function to create a Dca component.  

```
CALL_FUNCTION
    Address("")
    "Dca"
    "new"
    Address("<OWNER_BADGE_ADDRESS>")
    Address("<PROXY_BADGE_ADDRESS>")
    Address("<TIMER_BADGE_ADDRESS>")
    Address("<BASE_COIN_ADDRESS>")
    Address("<RADIX_PUMP_COMPONENT>")
;
```

`<OWNER_BADGE_ADDRESS>` is the resource address of the component owner badge.  
`<PROXY_BADGE_ADDRESS>` is the resource address of the proxy badge minted by the RadixPump component.  
`<TIMER_BADGE_ADDRESS>` is the resource address of the timer badges minted by the Timer component.  
`<BASE_COIN_ADDRESS>` is the resource address of the base coin used by RadixPump.  
`<RADIX_PUMP_COMPONENT>` is the address of the RadixPump component.  

### new_task

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<TIMER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<TIMER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("timer_badge_proof")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("<COIN1_ADDRESS>")
    Decimal("<COIN1_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<COIN1_ADDRESS>")
    Bucket("coin1_bucket")
;
CALL_METHOD
    Address("<DCA_COMPONENT>")
    "new_task"
    Proof("timer_badge_proof")
    Bucket("coin1_bucket")
    Decimal("<COIN1_PER_BUY_OPERATION>")
    Decimal("<MAX_PRICE>")
    <MIN_INTERVAL_BUY_OPERATIONS>u32
;
```

`<ACCOUNT_ADDRESS>` is the user account address.  
`<TIMER_BADGE_ADDRESS>` is the resource address of the timer badge minted by the Timer component.  
`<TIMER_BADGE_ID>` is numeric id of the timer badge minted by the Timer component.  
`<COIN1_ADDRESS>` is the resource address of the coin the user wants to spend in order to buy the desired coin.  
`<COIN1_AMOUNT>` is the amount of coin1 the user wants to deposit in the component (it can be increased later).  
`<DCA_COMPONENT>` is the address of the Dca component.  
`<COIN1_PER_BUY_OPERATION>` is the amount of coin1 the user wants to spend in a single buy operation.  
`<MAX_PRICE>` is the maximum coin1/coin to buy price the user wants to buy.  
`<MIN_INTERVAL_BUY_OPERATIONS>` is the minimum time that must pass between one buy operation and the following one. The user can schedule the task aggressively in the Timer and limit the buy operation through this and the `<MAX_PRICE>` parameter to try to get a better price.  

### withdraw

Users can invoke this method to withdraw their bought coins of to stop the buy task and withdraw the provided coins too.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<TIMER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<TIMER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("timer_badge_proof")
;
CALL_METHOD
    Address("<DCA_COMPONENT>")
    "withdraw"
    Proof("timer_badge_proof")
    <BOUGHT_COINS_ONLY>
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the user account address.  
`<TIMER_BADGE_ADDRESS>` is the resource address of the timer badge minted by the Timer component.  
`<TIMER_BADGE_ID>` is numeric id of the timer badge minted by the Timer component.  
`<DCA_COMPONENT>` is the address of the Dca component.  
`<BOUGHT_COINS_ONLY>` whether to withdraw the bouth coins only (true) or the provided coin1 too and leave the component out of funds (false).  

### add_funds

Users can call this method to add funds to a previously created buy task.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<TIMER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<TIMER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("timer_badge_proof")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("<COIN1_ADDRESS>")
    Decimal("<COIN1_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<COIN1_ADDRESS>")
    Bucket("coin1_bucket")
;
CALL_METHOD
    Address("<DCA_COMPONENT>")
    "add_funds"
    Proof("timer_badge_proof")
    Bucket("coin1_bucket")
;
```

`<ACCOUNT_ADDRESS>` is the user account address.  
`<TIMER_BADGE_ADDRESS>` is the resource address of the timer badge minted by the Timer component.  
`<TIMER_BADGE_ID>` is numeric id of the timer badge minted by the Timer component.  
`<COIN1_ADDRESS>` is the resource address of the coin the user wants to spend in order to buy the desired coin.  
`<COIN1_AMOUNT>` is the additional amount of coin1 to add to the previously deposited ones.  
`<DCA_COMPONENT>` is the address of the Dca component.  

### update_task

A user can call this method to modify one or more parameters specified during the task creation.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<TIMER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<TIMER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("timer_badge_proof")
;
CALL_METHOD
    Address("<DCA_COMPONENT>")
    "update_task"
    Proof("timer_badge_proof")
    Decimal("<COIN1_PER_BUY_OPERATION>")
    Decimal("<MAX_PRICE>")
    <MIN_INTERVAL_BUY_OPERATIONS>u32
;
```

`<ACCOUNT_ADDRESS>` is the user account address.  
`<TIMER_BADGE_ADDRESS>` is the resource address of the timer badge minted by the Timer component.  
`<TIMER_BADGE_ID>` is numeric id of the timer badge minted by the Timer component.  
`<DCA_COMPONENT>` is the address of the Dca component.  
`<COIN1_PER_BUY_OPERATION>` is the amount of coin1 the user wants to spend in a single buy operation.  
`<MAX_PRICE>` is the maximum coin1/coin to buy price the user wants to buy.  
`<MIN_INTERVAL_BUY_OPERATIONS>` is the minimum time that must pass between one buy operation and the following one.  
