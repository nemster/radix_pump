# Liquidity providers rewards hook

This hook can be used by a coin creator to reward the liquidity providers of his pool.  
The reward can be any fungible coin.  

For the automatical rewards withdraw to work it is needed that both the `AddLiquidity` and `RemoveLiquidity` operations are intercepted by this hook so it is advisable to enable the hook for `AddLiquidity` globally as soon as possible and let coin creators enable/disable the hook for `RemoveLiquidity` when they start/end liquidity campaigns.  
When a user removes his liquidity from a pool (`RemoveLiquidity` operation) he automatically gets his rewards.  
If a `RemoveLiquidity` operation happens but the corresponding `AddLiquidity` has not been intercepted by the hook, the transaction doesn't fail (liquidity removal suceeds) but a `UnknownRewardAmountEvent` is emitted.  
Automatic withdrawal of rewards may also fail if the hook runs out of rewards; in this case a `OutOfFundsEvent` event is issued.  

Users can also withdraw their rewards without removing liquidity by invoking directly the `get_rewards` method.  
This way it works event if the `AddLiquidity` operation has not been processed by the hook.  

## Known bugs and limitations

Only one liquidity campaign at a time can be created per coin.  

## Transaction manifests

### Instantiation

Use this function to create a LpRewardsHook component.  

```
CALL_FUNCTION
    Address("")
    "LpRewardsHook"
    "new"
    Address("<OWNER_BADGE_ADDRESS>")
    Address("<PROXY_BADGE_ADDRESS>")
    Address("<COIN_CREATOR_BADGE_ADDRESS>")
    <GRACE_PERIOD>i64
;
```

`<OWNER_BADGE_ADDRESS>` is the resource address of the component owner badge.  
`<PROXY_BADGE_ADDRESS>` is the resource address of the proxy badge minted by the RadixPump component.  
`<COIN_CREATOR_BADGE_ADDRESS>` is the resource address of the coin creator badges minted by the RadixPump component.  
`<GRACE_PERIOD>` is the minimum time in seconds that must occur from the end of a campaign to its destruction by the coin creator; this is ment to give liquidity providers enough time to withdraw their rewards.  

### new_liquidity_campaign

A coin creator can invoke this method to create a new liquidity campaign for his pool.  

A `LiquidityCampaignCreationEvent` is emitted.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<COIN_CREATOR_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<COIN_CREATOR_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("coin_creator_proof")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("<REWARD_COIN_ADDRESS>")
    Decimal("<REWARD_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<REWARD_COIN_ADDRESS>")
    Bucket("rewards_bucket")
;
CALL_METHOD
    Address("<LP_HOOK_ADDRESS>")
    "new_liquidity_campaign"
    Proof("coin_creator_proof")
    <START_TIME>i64
    <END_TIME>i64
    Decimal("<DAILY_REWARD_PER_COIN>")
    Bucket("rewards_bucket")
;
```

`<ACCOUNT_ADDRESS>` is the account holding the coin creator badge.  
`<REWARD_COIN_ADDRESS>` is the resource address of the coin that will be given as a reward to the liquidity providers.  
`<REWARD_AMOUNT>' is the amount of reward coins that the creator wants to deposit in the hook (it can be increased later).  
`<COIN_CREATOR_BADGE_ADDRESS>` is the resource address of the coin creator badges minted by the RadixPump component.  
`<COIN_CREATOR_BADGE_ID>` is the numerid id of the coin creator badge minted by RadixPump.  
`<LP_HOOK_ADDRESS>` is the component address of the LP rewards hook.  
`<START_TIME>` is the start of the period since the rewards are computed; it can be both in the future or in the past.  
`<END_TIME>` is the end of the period of the computed rewards.  
`<DAILY_REWARD_PER_COIN>` how many reward coins each liquidity provider will receive daily for each coin he added to the pool.  

### update_liquidity_campaign

A coin creator can invoke this method to pospone the end of an existing campaign or add funds to it.  
It is not possible to modify the rewards rate or shorten the period.  

The call to the `wihdraw` method and the subsequent `TAKE_ALL_FROM_WORKTOP` can be omitted and `Some(Bucket("rewards_bucket")` replaced with `None` if the creator doesn't want to deposit additional rewards.  
In the same way, `Some(<END_TIME>i64)` can be replaced by `None` if the creator doesn't want to pospone the end of the campaign.  

A new `LiquidityCampaignCreationEvent` is emitted.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<COIN_CREATOR_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<COIN_CREATOR_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("coin_creator_proof")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("<REWARD_COIN_ADDRESS>")
    Decimal("<REWARD_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<REWARD_COIN_ADDRESS>")
    Bucket("rewards_bucket")
;
CALL_METHOD
    Address("<LP_HOOK_ADDRESS>")
    "update_liquidity_campaign"
    Some(<END_TIME>i64)
    Some(Bucket("rewards_bucket"))
;
```
`<ACCOUNT_ADDRESS>` is the account holding the coin creator badge.  
`<COIN_CREATOR_BADGE_ADDRESS>` is the resource address of the coin creator badges minted by the RadixPump component.  
`<COIN_CREATOR_BADGE_ID>` is the numerid id of the coin creator badge minted by RadixPump.  
`<REWARD_COIN_ADDRESS>` is the resource address of the coin that will be given as a reward to the liquidity providers; it must be the same of the previously deposited coins.  
`<REWARD_AMOUNT>' is the amount of additional reward coins that the creator wants to deposit in the hook.  
`<LP_HOOK_ADDRESS>` is the component address of the LP rewards hook.  
`<END_TIME>` is the new end of the period of the computed rewards (must be after the previously set value).  

### terminate_liquidity_campaign

A coin creator can call this method to close a terminated liquidity campaign and get any remaining funds back.  
Termination can't happen before `<END_TIME> + <GRACE_PERIOD>`.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<COIN_CREATOR_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<COIN_CREATOR_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("coin_creator_proof")
;
CALL_METHOD
    Address("<LP_HOOK_ADDRESS>")
    "terminate_liquidity_campaign"
    Proof("coin_creator_proof")
;
```

`<ACCOUNT_ADDRESS>` is the account holding the coin creator badge.  
`<COIN_CREATOR_BADGE_ADDRESS>` is the resource address of the coin creator badges minted by the RadixPump component.  
`<COIN_CREATOR_BADGE_ID>` is the numerid id of the coin creator badge minted by RadixPump.  
`<LP_HOOK_ADDRESS>` is the component address of the LP rewards hook.  

### get_rewards

A user can invoke this method to withdraw his rewards without removing his liquidity from a pool.  

It is possible to get rewards for multilple LP tokens at once as long as they all belong to the same pool.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<LP_NFT_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<LP_NFT_ID>#"), NonFungibleLocalId("#<LP_NFT_ID>#")...)
;
POP_FROM_AUTH_ZONE
    Proof("lp_proof")
;
CALL_METHOD
    Address("<LP_HOOK_ADDRESS>")
    Proof("lp_proof")
;
```

`<ACCOUNT_ADDRESS>` is the user account.  
`<LP_NFT_ADDRESS>` is the resource address of the LP NFT minted by RadixPump when depositing liquidity in a pool.  
`<LP_NFT_ID>` is numeric id of the LP NFT minted by RadixPump when depositing liquidity in a pool.  
`<LP_HOOK_ADDRESS>` is the component address of the LP rewards hook.  

