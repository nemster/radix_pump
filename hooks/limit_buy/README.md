# Limit buy hook

This blueprint implements a limit buy order system as a hook for RadixPump.  
This hook can both be invoked by RadixPump when a Sell operation happens on a pool or by the Timer.  

## Known bugs and limitations

Pending order are kept in a Vec; this limits the maximum number of pending orders that can be stored without the transaction costs grow too much. This limit is set to 500 per coin.  
The number of matched orders per operation is limited too to limit the transaction costs; this is set to 30.  

## Transaction manifests

### Instantiate

Call this function to create a LimitBuyHook component.  

```
CALL_FUNCTION
    Address("")
    "LimitBuyHook"
    "new"
    Address("<OWNER_BADGE_ADDRESS>")
    Address("<PROXY_BADGE_ADDRESS>")
    Address("<BASE_COIN_ADDRESS>")
    Address("<RADIX_PUMP_COMPONENT>")
;
```

`<OWNER_BADGE_ADDRESS>` is the resource address of the component owner badge.  
`<PROXY_BADGE_ADDRESS>` is the resource address of the proxy badge minted by the RadixPump component.  
`<BASE_COIN_ADDRESS>` is the resource address of the coin (probably XRD) that will be used to buy coins from the component.  
`<RADIX_PUMP_COMPONENT>` is the address of the RadixPump component.  

### new_order

Users can call this method to create a new limit order.  
If the order can be filled or partially filled immediately, the method returns the bought coins.  
If the order can't be filled immediately, the method returns a `LimitBuyOrder` NFT that will be needed for future operations.  

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
    Address("<LIMIT_BUY_COMPONENT>")
    "new_order"
    Bucket("base_coin_bucket")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the account of the user creating the limit buy order.  
`<BASE_COIN_ADDRESS>` is the base coin address specified in the component creation (probably XRD).  
`<BASE_COIN_AMOUNT>` is the base coin amount that will be used to buy coins.  
`<LIMIT_BUY_COMPONENT>` is the LimitBuyHook component address.  

### withdraw

Users can invoke this method to withdraw their bought coins or cancel one or more pending order.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw_non_fungibles"
    Address("<LIMIT_BUY_NFT_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<LIMIT_BUY_NFT_ID>#"), NonFungibleLocalId("#<LIMIT_BUY_NFT_ID>#")...)
;
TAKE_ALL_FROM_WORKTOP
    Address("<LIMIT_BUY_NFT_ADDRESS>")
    Bucket("order_bucket")
;
CALL_METHOD
    Address("<LIMIT_BUY_COMPONENT>")
    "withdraw"
    Bucket("order_bucket")
    <COINS_ONLY>
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the account of the user who owns the limit buy order NFT(s).  
`<LIMIT_BUY_NFT_ADDRESS>` is the resource address of the limit buy order NFTs.  
`<LIMIT_BUY_NFT_ID>` is the numeric id of one of the limit buy order NFTs to cancel or withdraw.  
`<LIMIT_BUY_COMPONENT>` is the LimitBuyHook component address.  
`<COINS_ONLY>` is a boolean value. If true only the bought coins are withdrawn, if the order is not filled it will stay in place. If false the order is canceled and both coins and base coins are withdrawn.  

