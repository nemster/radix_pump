# RadixPump blueprint

This blueprint implements a marketplace where people can create, buy and sell coins.

## Transaction manifests

### Instantiate (Stokenet)

Use this function to create a RadixPump component in Stokenet

```
CALL_FUNCTION
    Address("package_tdx_2_1p5nuq8rud5h3dw6gae8qmg4asftg8tsyrg75sa7hn8c93y7ep6ld90")
    "RadixPump"
    "new"
    Address("<OWNER_BADGE_ADDRESS>")
    Address("<BASE_COIN_ADDRESS>")
    Decimal("<MINIMUM_DEPOSIT>")
    Decimal("<COINS_SUPPLY>")
    Decimal("<CREATOR_COINS_AMOUNT>")
    Decimal("<CREATION_FEE_PERCENTAGE>")
    Decimal("<BUY_SELL_FEE_PERCENTAGE>")
;
```

`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that can be later used to withdraw fees and other reserved operations.  
`<BASE_COIN_ADDRESS>` is the resource address of the coin (probably XRD) that will be used to buy coins from the component.  
`<MINIMUM_DEPOSIT>` is the minimum amount of base coins that a new coin creator must deposit.  
`<COINS_SUPPLY>` is the fixed supply of all of the coins that will be created.  
`<CREATOR_COINS_AMOUNT>` how many coins are released to the creator when he creates a new coin.  
`<CREATION_FEE_PERCENTAGE>` is the percentage (expressed as a number from o to 100) of base coins paid by the token creators.  
`<BUY_SELL_FEE_PERCENTAGE>` is the percentage (expressed as a number from o to 100) of base coins paid by buyers and sellers.  

### Forbid symbols

Only the component owner can call this method.  
Use this method to avoid people create coins with the same symbol of an already existing coin. The component avoids to create multiple coins with the same symbol but it's up to the component owner avoiding that users create coins with the same symbol of another project.  
This method can be called more than once to add new symbols to the already forbidden ones.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "forbid_symbols"
    Array<String>("<SYMBOL>", "<SYMBOL>"...)
;
```
`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<COMPONENT_ADDRESS>` is the address of the component created with the `new` function.  
`<SYMBOL>` is one of the symbols that need to be excluded. The symbols are converted to uppercase. An example list can be `Array<String>("XRD", "WXBTC", "XUSDC", "FLOOP", "CAVIAR", "XETH", "EARLY", "DFP2", "HUG", "OCI", "IDA", "WEFT", "WOWO", "XUSDT", "FOTON", "CAVIAR", "DAN", "DGC", "ASTRL", "DPH", "GAB", "FOMO", "CASSIE", "HIT", "JWLXRD", "LSUSP", "BOBBY", "DEXTR", "ICE")`  

### Forbid names

Only the component owner can call this method.  
Use this method to avoid people create coins with the same name of an already existing coin. The component avoids to create multiple coins with the same name but it's up to the component owner avoiding that users create coins with the same name of another project.  
This method can be called more than once to add new names to the already forbidden ones.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "forbid_names"
    Array<String>("<NAME>", "<NAME>"...)
;
```
`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<COMPONENT_ADDRESS>` is the address of the component created with the `new` function.  
`<NAME>` is one of the names that need to be excluded. The comparison is not case sensitive.  

### Create new coin

A user can create a new coin using this method.  
A Radix network transaction that calls this method adds a small royalty that goes to the package owner (about $0.05 in XRD).  

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
    Address("<COMPONENT_ADDRESS>")
    "create_new_coin"
    Bucket("base_coin_bucket")
    "<COIN_SYMBOL>"
    "<COIN_NAME>"
    "<COIN_ICON_URL>"
    "<COIN_DESCRIPTION>"
;
```

`<ACCOUNT_ADDRESS>` is the account of the user creating the new coin.  
`<BASE_COIN_ADDRESS>` is the base coin address specified in the component creation (probably XRD).  
`<BASE_COIN_AMOUNT>` is the base coin amount used to initialize the pool. It must be no less than the `<MINIMUM_DEPOSIT>` specifiled during the component creation. Not all of the amount goes into the pool: a percentage of `<CREATION_FEE_PERCENTAGE>` of this amount is credited to the component owner who can withdraw it later.  
`<COMPONENT_ADDRESS>` is the address of the component created with the `new` function.  
`<COIN_SYMBOL>` is the symbol to assign to the new coin. This is converted to uppercase and checked against all of the previously created coins' symbols and all of the symbols forbidden by the component owner.  
`<COIN_NAME>` is the name to assign to the new coin. This is checked against all of the previously created coins' names and all of the names forbidden by the component owner.  
`<COIN_ICON_URL>` is the URL of the image to assign as icon to the new coin; it must be a valid URL.  
`<COIN_DESCRIPTION>` is a descriptive text that is added to the coin metadata (can be empty).  

The coin creator receives a creator badge that can be later used to:  
- add new metadata to the coin  
- change existing coin metadata (symbol and name excluded)  
- lock coin metadata  
- burn his coins  
- allow third parties to burn their coins  

A `NewCoinEvent` event is issued. It contains the resource address of the new coin, the initial coin price and the number of coins currently in the pool.  

### Buy coins

A user can buy an existing coin using this method paying with base coins.  
A Radix network transaction that calls this method adds a very small royalty that goes to the package owner (about $0.005 in XRD).  

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
    Address("<COMPONENT_ADDRESS>")
    "buy"
    Address("<COIN_ADDRESS>")
    Bucket("base_coin_bucket")
;
```

`<ACCOUNT_ADDRESS>` is the account of the user buying the coin.  
`<BASE_COIN_ADDRESS>` is the base coin address specified in the component creation (probably XRD).  
`<BASE_COIN_AMOUNT>` is the base coin amount to buy the coin. A percentage of `<BUY_SELL_FEE_PERCENTAGE>` of this amount is credited to the component owner who can withdraw it later.  
`<COMPONENT_ADDRESS>` is the address of the component created with the `new` function.  
`<COIN_ADDRESS>` is the resource address of the coin the user wants to buy.  

This method returns a bucket of the requested coin.

A `BuyEvent` event is issued. It contains the resource address of the bought coin, the bought amount, the new price and the number of coins currently in the pool.  

### Sell coins

A user can sell coins using this method to receive base coins back.  
A Radix network transaction that calls this method adds a very small royalty that goes to the package owner (about $0.005 in XRD).  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("<COIN_ADDRESS>")
    Decimal("<COIN_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<COIN_ADDRESS>")
    Bucket("coin_bucket")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "sell"
    Bucket("coin_bucket")
;
```

`<ACCOUNT_ADDRESS>` is the account of the user selling the coin.  
`<COIN_ADDRESS>` is the coin the user wants to sell.  
`<BASE_COIN_AMOUNT>` is the coin amount the user wants to sell.  
`<COMPONENT_ADDRESS>` is the address of the component created with the `new` function.  

This method returns a bucket of base coin.
A percentage of `<BUY_SELL_FEE_PERCENTAGE>` is subtracted from the proceeds of coin sales and is credited to the component owner who can withdraw it later.  

A `SellEvent` event is issued. It contains the resource address of the sold coin, the sold amount, the new price and the number of coins currently in the pool.  

### Get fees

The component owner can use this method to claim the fees paid by creators, buyers and sellers.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "get_fees"
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<COMPONENT_ADDRESS>` is the address of the component created with the `new` function.  

The fees are always base coins (probably XRD).  

### Update fees

The component owner can use this method to update the fees for creators, buyers and sellers.

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "update_fees"
    Decimal("<CREATION_FEE_PERCENTAGE>")
    Decimal("<BUY_SELL_FEE_PERCENTAGE>")
;
```
