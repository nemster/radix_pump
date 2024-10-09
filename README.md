# RadixPump blueprint

This blueprint implements a marketplace where people can create, buy, sell and take flash loans of coins.

## Fair launch

To create a new coin you have to deposit an amount of a base coin. It's up to the component owner, during instatiation, to decide which base coin to use (XRD or another one) and the minimum deposit amount needed to crate a new coin.  
When a new coin is created the creator gets a share of it at about the same price as the earliest buyers of the coin.  

## Modified constant product formula

When a new coin is created most of its supply goes into a pool.  
The classic constant product formula isn't very suited for handling an unbalanced pool containing the whole supply of a coin and a small amount of the base coin. A user could easily buy great part of the supply for few base coins.  
This is why a modified version of the constant product formula is used. This modified version progressively approaches the classic one as the pool gets more balanced.  

## Liquidation mode

It may happen that a project fails or is rugged by its creator.
The liquidation mode is a guarantee that a partial reimbursement is possible for the holders of a failed project or a rugged coin.  

Please note: the creator of a coin has no way to withdraw the liquidity out of the pool, he can only sell his coins, just like anyone else. So a partial reimbursement will always be possible for all of the holders.  

Both the creator of a coin and the component owner can turn a coin into liquidation mode, when this happens:
- it's no longer possible to buy or borrow the coin  
- the base coins in the pool are divided pro-rata among the coin holders  

There's no going back from the liquidation mode.  

## Flash loans

Coins created in RadixPump can be borrowed by users.  
The user must return a fee in base coin together with the borrowed coins in the same transaction or it fails:  
`get_flash_loan` -> do something with the coins -> `return_flash_loan`  
 
The component owner gets his own fee percentage while the coin creator can set a fee percentage that goes to the pool. Both fees are paid in base coins.  

## Pool fees

A coin creator can set fees on buy, sell and flash loan operations for his coin.  

The component owner can set the upper limit for buy/sell and flash loan fees; by default this limit is 10%. the limit also applies retroactively to coins already created.  

No one can retrieve pool fees, the paid base coins just get into the pool itself. The effect is a coin price increase.  

## Known limitations

To avoid math overflows the supply of the created coins can't be bigger than 10^20.  

## Verifiable Scrypto build

Compiled with `radixdlt/scrypto-builder:v1.2.0`  

This is the SHA256 of the package files:  

## Transaction manifests

### Instantiate (Stokenet)

Use this function to create a RadixPump component in Stokenet

```
CALL_FUNCTION
    Address("")
    "RadixPump"
    "new"
    Address("<OWNER_BADGE_ADDRESS>")
    Address("<BASE_COIN_ADDRESS>")
    Decimal("<MINIMUM_DEPOSIT>")
    Decimal("<CREATION_FEE_PERCENTAGE>")
    Decimal("<BUY_SELL_FEE_PERCENTAGE>")
    Decimal("<FLASH_LOAN_FEE_PERCENTAGE>")
;
```

`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that can be later used to withdraw fees and other reserved operations.  
`<BASE_COIN_ADDRESS>` is the resource address of the coin (probably XRD) that will be used to buy coins from the component.  
`<MINIMUM_DEPOSIT>` is the minimum amount of base coins that a new coin creator must deposit.  
`<CREATION_FEE_PERCENTAGE>` is the percentage (expressed as a number from 0 to 100) of base coins paid by the token creators to the component owner.  
`<BUY_SELL_FEE_PERCENTAGE>` is the percentage (expressed as a number from 0 to 100) of base coins paid by buyers and sellers to the component owner.  
`<FLASH_LOAN_FEE_PERCENTAGE>` is the percentage (expressed as a number from 0 to 100) of base coins paid by flash borrowers to the component owner.  

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
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
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
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
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
    Decimal("<COIN_SUPPLY>")
    Decimal("<BUY_POOL_FEE_PERCENTAGE>")
    Decimal("<SELL_POOL_FEE_PERCENTAGE>")
    Decimal("<FLASH_LOAN_POOL_FEE_PERCENTAGE>")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the account of the user creating the new coin.  
`<BASE_COIN_ADDRESS>` is the base coin address specified in the component creation (probably XRD).  
`<BASE_COIN_AMOUNT>` is the base coin amount used to initialize the pool. It must be no less than the `<MINIMUM_DEPOSIT>` specifiled during the component creation. Not all of the amount goes into the pool: a percentage of `<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<COIN_SYMBOL>` is the symbol to assign to the new coin. This is converted to uppercase and checked against all of the previously created coins' symbols and all of the symbols forbidden by the component owner.  
`<COIN_NAME>` is the name to assign to the new coin. This is checked against all of the previously created coins' names and all of the names forbidden by the component owner.  
`<COIN_ICON_URL>` is the URL of the image to assign as icon to the new coin; it must be a valid URL.  
`<COIN_DESCRIPTION>` is a descriptive text that is added to the coin metadata (can be empty).  
`<COIN_SUPPLY>` is the initial supply of the new coin. It is not be possible to incease the supply later but it can be reduced by burning coins.  
`<BUY_POOL_FEE_PERCENTAGE>`  is the percentage (expressed as a number from 0 to 100) of base coins paid by buyers to the coin pool. The component owner can se a upper limit to this parameter (by default 10%).  
`<SELL_POOL_FEE_PERCENTAGE>`  is the percentage (expressed as a number from 0 to 100) of base coins paid by sellers to the coin pool. The component owner can se a upper limit to this parameter (by default 10%).  
`<FLASH_LOAN_POOL_FEE_PERCENTAGE>`  is the percentage (expressed as a number from 0 to 100) of base coins paid by flash borrowers to the coin pool. The component owner can se a upper limit to this parameter (by default 10%).  

The coin creator receives a creator badge NFT that shows in the wallet a numeric ID and the resource address of the new created coin.  
This badge can be later used to:  
- add new metadata to the coin  
- change existing coin metadata (symbol and name excluded)  
- lock coin metadata  
- burn the coins he holds  
- allow third parties to burn the coins they hold  
- start liquidation mode for the pool  

The coin creator also receives a number of coins as if he bought them from the pool with his initial deposit. The price is comparable to the one that will be paid by the first buyers.  

A `NewCoinEvent` event is issued. It contains the resource address of the new coin, the initial coin price, the amount held by the creator and the number of coins currently in the pool.  

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
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the account of the user buying the coin.  
`<BASE_COIN_ADDRESS>` is the base coin address specified in the component creation (probably XRD).  
`<BASE_COIN_AMOUNT>` is the base coin amount to buy the coin. A percentage of `<BUY_SELL_FEE_PERCENTAGE>` of this amount is credited to the component owner who can withdraw it later.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
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
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the account of the user selling the coin.  
`<COIN_ADDRESS>` is the coin the user wants to sell.  
`<BASE_COIN_AMOUNT>` is the coin amount the user wants to sell.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  

This method returns a bucket of base coin.
A percentage of `<BUY_SELL_FEE_PERCENTAGE>` is subtracted from the proceeds of coin sales and is credited to the component owner who can withdraw it later.  

A `SellEvent` event is issued. It contains the resource address of the sold coin, the sold amount, the new price and the number of coins currently in the pool.  

### Get fees

The component owner can use this method to claim the fees paid by creators, buyers, sellers and borrowers.  

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
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  

The fees are always base coins (probably XRD).  

### Update fees

The component owner can use this method to update the fees for creators, buyers, sellers and flash borrowers.

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
    Decimal("<FLASH_LOAN_FEE_PERCENTAGE>")
    Decimal("<MAX_BUY_SELL_POOL_FEE_PERCENTAGE>")
    Decimal("<MAX_FLASH_LOAN_POOL_FEE_PERCENTAGE>")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<CREATION_FEE_PERCENTAGE>` is the new percentage (expressed as a number from 0 to 100) of base coins paid by the token creators.  
`<BUY_SELL_FEE_PERCENTAGE>` is the new percentage (expressed as a number from 0 to 100) of base coins paid by buyers and sellers.  
`<FLASH_LOAN_FEE_PERCENTAGE>` is the percentage (expressed as a number from 0 to 100) of base coins paid by flash borrowers to the component owner.  
`<MAX_BUY_SELL_POOL_FEE_PERCENTAGE>` is the upper limit to the `buy_sell_pool_fee_percentage` a coin creator can set (by default 100). If the component owner changes this value it is also applied retroactively to the existing pools.  
`<MAX_FLASH_LOAN_POOL_FEE_PERCENTAGE>` is the upper limit to the `flash_loan_pool_fee_percentage` a coin creator can set (by default 100). If the component owner changes this value it is also applied retroactively to the existing pools.  

### Owner initiated liquidation mode

The component owner can set liquidation mode for any coin.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "owner_set_liquidation_mode"
    Address("<COIN_ADDRESS>")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<COIN_ADDRESS>` is the coin the component owner wants to put in liquidation mode.  

A `LiquidationEvent` containing the resource address of the liquidating coin is issued.  

### Creator initiated liquidation mode

The creator of a coin can set liquidation mode for it.

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
    Address("<COMPONENT_ADDRESS>")
    "creator_set_liquidation_mode"
    Proof("creator_proof")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<CREATOR_BADGE_ADDRESS>` is the badge receaved when creating the coin.  
`<CREATOR_BADGE_ID>` is the numeric ID of the badge received when creating the coin.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  

A `LiquidationEvent` containing the resource address of the liquidating coin is issued.  

### Get a flash loan

Get a flash loan of a coin created in RadixPump

```
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "get_flash_loan"
    Address("<COIN_ADDRESS>")
    Decimal("<LOAN_AMOUNT>")
;
```

`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<COIN_ADDRESS>` is the resource address of the coin the user wants to borrow.  
`<LOAN_AMOUNT>` is the requested loan amount.  

Together with the coin bucket a transient NFT is returned, this NFT can't be deposited anywhere, it can only be burned by the `return_flash_loan` method. Not burning it will cause the transaction to fail.  

### Return a flash loan

```
TAKE_ALL_FROM_WORKTOP
    Address("<TRANSIENT_NFT_ADDRESS>")
    Bucket("transient_nft_bucket")
;
TAKE_FROM_WORKTOP
    Address("<BASE_COIN_ADDRESS>")
    Decimal("<FEES>")
    Bucket("base_coin_bucket")
;
TAKE_FROM_WORKTOP
    Address("<COIN_ADDRESS>")
    Decimal("<LOAN_AMOUNT>")
    Bucket("coin_bucket")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "return_flash_loan"
    Bucket("transient_nft_bucket")
    Bucket("base_coin_bucket")
    Bucket("coin_bucket")
;
```
`<TRANSIENT_NFT_ADDRESS>` is the address of the transient NFT returned by the `get_flash_loan`. This is known at the component intantiation and never changes.  
`<BASE_COIN_ADDRESS>` is the base coin address specified in the component creation (probably XRD).  
`<FEES>` is the total fees the user must pay to the component owner and the pool.  
`<COIN_ADDRESS>` is the resource address of the coin the user borrowed.  
`<LOAN_AMOUNT>` is the requested loan amount.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  

### Update pool fee percentage

A coin creator can modify the pool fees specified at coin creation time.  

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
    Address("<COMPONENT_ADDRESS>")
    "update_pool_fee_percentage"
    Proof("creator_proof")
    Decimal("<BUY_FEE_PERCENTAGE>")
    Decimal("<SELL_FEE_PERCENTAGE>")
    Decimal("<FLASH_LOAN_FEE_PERCENTAGE>")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<CREATOR_BADGE_ADDRESS>` is the badge receaved when creating the coin.  
`<CREATOR_BADGE_ID>` is the numeric ID of the badge received when creating the coin.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<BUY_FEE_PERCENTAGE>` is the new percentage (expressed as a number from 0 to 100) of base coins paid by buyers and sellers. The upper limit for this parameter can be changed by the componet owner.  
`<SELL_FEE_PERCENTAGE>` is the new percentage (expressed as a number from 0 to 100) of base coins paid by buyers and sellers. The upper limit for this parameter can be changed by the componet owner.  
`<FLASH_LOAN_FEE_PERCENTAGE>` is the percentage (expressed as a number from 0 to 100) of base coins paid by flash borrowers to the component owner.  

### Get pool information

This method can be useful for third parties code that needs to interact with a RadixPump component.  

```
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "get_pool_info"
    Address("<COIN_ADDRESS>")
;
```

`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<COIN_ADDRESS>` is the resource address of the coin the user wants to receve information about.  

The method returns these information:  
- the amount of base coins in the coin pool.  
- the amount of coins in the pool.  
- the price of the last buy or sell operation.  
- total (component owner + pool) buy fee percentage.  
- total (component owner + pool) sell fee percentage.  
- total (component owner + pool) flash loan fee percentage.  
- the pool mode (Normal or Liquidation).  
- the resource address of the transient NFT used in flash loans (it's the same for all of the coins).  

## Copyright

See LICENSE
