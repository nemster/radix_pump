# ATH Club hook

This blueprint implements a RadixPump hook that reward users with an NFT when they buy an ATH.  
The ATH Club NFT contains information about the coin and price bought and if the ATH is still valid or has been passed (obsoleted) by a new one.  

Coin creators have to enable this hook for the Buy operation for it to work for their coins.  

## Transaction manifests

### Instantiate

New component creation

```
CALL_FUNCTION
    Address("")
    "AthClubHook"
    "new"
    Address("<OWNER_BADGE_ADDRESS>")
    Address("<PROXY_BADGE_ADDRESS>")
    Address("<COIN_CREATOR_BADGE_ADDRESS>")
    "<DEFAULT_IMAGE_URL>"
;
```

`<OWNER_BADGE_ADDRESS>` is the resource address of the owner badge of this component
`<PROXY_BADGE_ADDRESS>` is the resource address of badge RadixPump will use to authenticate against this hook
`<COIN_CREATOR_BADGE_ADDRESS>` is the resource address of the creator badges minted by RadixPump
`<DEFAULT_IMAGE_URL>` Is the default image to put in the NFT if a coin has no `icon_url` metadata (should never happen but better safe than sorry)

### init_coin

If the ATH Club hook is enabled for a non fresh launched coin, the creator may want to initialize it with past ATH value.  
This is not mandatory: it is possible to just the let the hook consider the first buy operation as the first ATH.  

This method will panic if an ATH Club NFT has already been minted for the coin.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<CREATOR_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<CREATOR_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("creator_proof")
;
CALL_METHOD
    Address("<ATH_CLUB_COMPONENT>")
    "init_coin"
    Proof("creator_proof")
    Decimal("<ATH_PRICE>")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the coin creator badge.  
`<CREATOR_BADGE_ADDRESS>` is the badge receaved when creating the coin.  
`<CREATOR_BADGE_ID>` is the numeric ID of the badge received when creating the coin.  
`<ATH_CLUB_COMPONENT>` is the address of the Ath Club component.  
