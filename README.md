# RadixPump blueprint

This blueprint implements a marketplace where people can create, buy, sell and take flash loans of coins.

## Launch types

RadixPump supports two coin lauch types: FairLaunch and QuickLaunch.  

### FairLaunch

A FairLaunch happens in multiple steps:  
- The creator creates the coin calling the `new_fair_launch` method specifying the launch price and percentage of locket tokens he wants to reserve for himself. No coin is minted at this stage.  
- The creator launches the coin calling the `launch` method specifying when the launch will end and when his tokens will be completely unlocked. Every user can now buy the coin at the same price, no one can sell.  
- The creator closes the launch phase calling the `terminate_launch` method and receives the proceeds of the coin sale (fees excluded). Now the price can move and users can buy and sell the coin at market level, the unlock of the creator coins starts and the supply stops.  
- Once the launch phase is completed the creator can call the `unlock` whenever he wants; he receives a number of his locked coins proportional to the time passed since the end of the sale.  

The supply is unknown until `terminate_launch` is called: new coins are minted when they are bought during the Launching phase. When launch ends the maximum supply is fixed.  

### QuickLaunch

To create a new coin you have to deposit an amount of a base coin. It's up to the component owner to decide which base coin to use (XRD or another one) and the minimum deposit amount needed to create a new coin.  
When a new coin is created the creator gets a share of it at about the same price as the earliest buyers of the coin while the largest part on the supply stays into the pool for users to buy it. 

The classic constant product formula isn't very suited for handling the pool generated by the QuickLaunch: an unbalanced pool containing the whole supply of a coin and a small amount of the base coin.  
A user could easily buy great part of the supply for few base coins.  
This is why a modified version of the constant product formula is used. This modified version just ignores part of the coins in the pool so that bot sides of the pool have equal value.  
Everytime a user buys the coin, the number of ignored coins decreases. 

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

The component owner can set the upper limit for buy/sell and flash loan fees; by default this limit is 10%.

No one can retrieve pool fees, the paid base coins just get into the pool itself. The effect is a coin price increase.  

## Hooks

Hooks are external component authomatically called by RadixPump when certain operations are performed.  

The component owner can make hooks available by calling the `register_hook` method, he must specify the operations this hook can be attached to.  
The available operations are `PostFairLaunch`, `PostTerminateFairLaunch`, `PostQuickLaunch`, `PostBuy`, `PostSell` and `PostReturnFlashLoan`. I avoided `Pre` hooks to prevent frontrunning and sandwitch attacks.  

Once an hook is registered the component owner can attach it to one or more operation globally (i.e. for all pools) via the `owner_enable_hook` method.
A coin owner can attach a registered hook to operations happening on his coin.

Hooks can be used to extend RadixPump features in any way; just few examples:
- make an airdrop to the 100 first buyers on my coin  
- authomatically buy the next 10 coins quick launched  
- authomatically buy the dips  
...  
A simple hook that just emits an event is provided as example; when developing a new hook make sure it has a `hook` method with the same arguments and return type as the provided example.

An hook can never steal the buckets intended for the user; it can only add new bucket towards him.  

RadixPump uses a proof of a badge when calling an hook, so the hook can be sure about the caller.  

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
`<MINIMUM_DEPOSIT>` is the minimum amount of base coins that a new coin creator must deposit when doing a QuickLaunch.  
`<CREATION_FEE_PERCENTAGE>` is the percentage (expressed as a number from 0 to 100) of base coins paid by the token creators to the component owner.  
`<BUY_SELL_FEE_PERCENTAGE>` is the percentage (expressed as a number from 0 to 100) of base coins paid by buyers and sellers to the component owner.  
`<FLASH_LOAN_FEE_PERCENTAGE>` is the percentage (expressed as a number from 0 to 100) of base coins paid by flash borrowers to the component owner.  

### forbid_symbols

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

### forbid_names

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

### new_fair_launch

A user can create a new coin and launch it using this method.  
A Radix network transaction that calls this method adds a small royalty that goes to the package owner (about $0.05 in XRD).  

```
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "new_fair_launch"
    "<COIN_SYMBOL>"
    "<COIN_NAME>"
    "<COIN_ICON_URL>"
    "<COIN_DESCRIPTION>"
    "<COIN_INFO_URL>"
    Decimal("<LAUNCH_PRICE>")
    Decimal("<CREATOR_LOCKED_PERCENTAGE>")
    Decimal("<BUY_POOL_FEE_PERCENTAGE>")
    Decimal("<SELL_POOL_FEE_PERCENTAGE>")
    Decimal("<FLASH_LOAN_POOL_FEE_PERCENTAGE>")
;
```

`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<COIN_SYMBOL>` is the symbol to assign to the new coin. This is converted to uppercase and checked against all of the previously created coins' symbols and all of the symbols forbidden by the component owner.  
`<COIN_NAME>` is the name to assign to the new coin. This is checked against all of the previously created coins' names and all of the names forbidden by the component owner.  
`<COIN_ICON_URL>` is the URL of the image to assign as icon to the new coin; it must be a valid URL.  
`<COIN_DESCRIPTION>` is a descriptive text that is added to the coin metadata (it can be an empty string).  
`<COIN_INFO_URL>` is the URL of the website of the coin (it can be an empty string).  
`<LAUNCH_PRICE>` is the price that will be constant during the launch phase.  
`<CREATOR_LOCKED_PERCENTAGE>` percentage of minted coins reserved to the creator (initially locked).  
`<BUY_POOL_FEE_PERCENTAGE>`  is the percentage (expressed as a number from 0 to 100) of base coins paid by buyers to the coin pool. The component owner can se a upper limit to this parameter (by default 10%).  
`<SELL_POOL_FEE_PERCENTAGE>`  is the percentage (expressed as a number from 0 to 100) of base coins paid by sellers to the coin pool. The component owner can se a upper limit to this parameter (by default 10%).  
`<FLASH_LOAN_POOL_FEE_PERCENTAGE>`  is the percentage (expressed as a number from 0 to 100) of base coins paid by flash borrowers to the coin pool. The component owner can se a upper limit to this parameter (by default 10%).  

The name and symbol of the coin are reserved no coin is minted at this stage.  

The coin creator receives a creator badge NFT that shows in the wallet a numeric ID, the resource address, name and symbol of the new created coin.  
This badge allows the creator to:  
- add new metadata to the coin  
- change existing coin metadata (symbol and name excluded)  
- lock coin metadata  
- burn the coins he holds  
- allow third parties to burn the coins they hold  
- start liquidation mode for the pool  
- modify pool fees  
- start and stop launch phase  
- get his coins according to the unlock schedule  

### new_quick_launch

A user can create a new coin and launch it using this method.  
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
    "new_quick_launch"
    Bucket("base_coin_bucket")
    "<COIN_SYMBOL>"
    "<COIN_NAME>"
    "<COIN_ICON_URL>"
    "<COIN_DESCRIPTION>"
    "<COIN_INFO_URL>"
    Decimal("<COIN_SUPPLY>")
    Decimal("<PRICE>")
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
`<COIN_INFO_URL>` is the URL of the website of the coin (it can be an empty string).  
`<COIN_SUPPLY>` is the initial supply of the new coin. It is not be possible to incease the supply later but it can be reduced by burning coins.  
`<PRICE>` is the initial price of the coin. The creator himself receives coins bought at this price with his base coin deposit.  
`<BUY_POOL_FEE_PERCENTAGE>`  is the percentage (expressed as a number from 0 to 100) of base coins paid by buyers to the coin pool. The component owner can se a upper limit to this parameter (by default 10%).  
`<SELL_POOL_FEE_PERCENTAGE>`  is the percentage (expressed as a number from 0 to 100) of base coins paid by sellers to the coin pool. The component owner can se a upper limit to this parameter (by default 10%).  
`<FLASH_LOAN_POOL_FEE_PERCENTAGE>`  is the percentage (expressed as a number from 0 to 100) of base coins paid by flash borrowers to the coin pool. The component owner can se a upper limit to this parameter (by default 10%).  

The coin creator receives a creator badge NFT that shows in the wallet a numeric ID, the resource address, name and symbol of the new created coin.  
This badge allows the creator to:  
- add new metadata to the coin  
- change existing coin metadata (symbol and name excluded)  
- lock coin metadata  
- burn the coins he holds  
- allow third parties to burn the coins they hold  
- start liquidation mode for the pool  
- modify pool fees  

The coin creator also receives a number of coins as if he bought them from the pool with his initial deposit. The price is the same that will be paid by the first buyer.  

A `QuickLaunchEvent` event is issued. It contains the resource address of the new coin, the initial coin price, the amount held by the creator and the number of coins currently in the pool.  

### buy

A user can buy an existing coin calling this method and paying with base coins.  
A Radix network transaction that calls this method adds a very small royalty that goes to the package owner (about $0.005 in XRD).  
It is not allowed to buy coins in liquidation mode.

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

A `BuyEvent` event is issued. It contains the resource address of the bought coin, the pool mode (Launching or Normal), the bought amount, the new price, the number of coins currently in the pool and the fees paid to the pool.  

### sell

A user can sell coins using this method to receive base coins back.  
A Radix network transaction that calls this method adds a very small royalty that goes to the package owner (about $0.005 in XRD).  
It is not allowed to sell coins in launching mode.

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

A `SellEvent` event is issued. It contains the resource address of the sold coin, the pool mode (Normal or Liquidation), the sold amount, the new price, the number of coins currently in the pool and the fees paid to the pool.  

### get_fees

The component owner can call this method to claim the fees paid by creators, buyers, sellers and borrowers.  

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

### update_fees

The component owner can use this method to update the fees for creators, buyers, sellers and flash borrowers and limit the fees of the pools.

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
`<MAX_BUY_SELL_POOL_FEE_PERCENTAGE>` is the upper limit to the `buy_sell_pool_fee_percentage` a coin creator can set (by default 10).  
`<MAX_FLASH_LOAN_POOL_FEE_PERCENTAGE>` is the upper limit to the `flash_loan_pool_fee_percentage` a coin creator can set (by default 10).  

### owner_set_liquidation_mode

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

### creator_set_liquidation_mode

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

### get_flash_loan

Get a flash loan of a coin created in RadixPump
A Radix network transaction that calls this method adds a very small royalty that goes to the package owner (about $0.002 in XRD).  

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

### return_flash_loan

Return the flash loan received with `get_flash_loan` and burn the transient NFT.
It is possible to return a flash loan only in Normal mode.

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
`<TRANSIENT_NFT_ADDRESS>` is the address of the transient NFT returned by the `get_flash_loan`. This is known at the component instantiation and never changes.  
`<BASE_COIN_ADDRESS>` is the base coin address specified in the component creation (probably XRD).  
`<FEES>` is the total fees the user must pay to the component owner and the pool.  
`<COIN_ADDRESS>` is the resource address of the coin the user borrowed.  
`<LOAN_AMOUNT>` is the requested loan amount.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  

A `FlashLoanEvent` event is issued. It contains the resource address of the borrowed coin, the amount returned and the fees paid to the pool.  

### update_pool_fee_percentage

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
    Decimal("<BUY_POOL_FEE_PERCENTAGE>")
    Decimal("<SELL_POOL_FEE_PERCENTAGE>")
    Decimal("<FLASH_LOAN_POOL_FEE_PERCENTAGE>")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<CREATOR_BADGE_ADDRESS>` is the badge receaved when creating the coin.  
`<CREATOR_BADGE_ID>` is the numeric ID of the badge received when creating the coin.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<BUY_POOL_FEE_PERCENTAGE>` is the new percentage (expressed as a number from 0 to 100) of base coins paid by buyers and sellers to the pool. The upper limit for this parameter can be changed by the componet owner.  
`<SELL_POOL_FEE_PERCENTAGE>` is the new percentage (expressed as a number from 0 to 100) of base coins paid by buyers and sellers to the pool. The upper limit for this parameter can be changed by the componet owner.  
`<FLASH_LOAN_POOL_FEE_PERCENTAGE>` is the percentage (expressed as a number from 0 to 100) of base coins paid by flash borrowers to the pool. The upper limit for this parameter can be changed by the componet owner.  

### get_pool_info

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
- the pool mode (WaitingForLaunch, Launching, Normal or Liquidation).  
- the end of the launch period (FairLaunch only)
- the end of the lock period (FairLaunch only)
- the creator allocation initially locked (FairLaunch only)
- the currently claimed creator allocation  (FairLaunch only)
- the resource address of the transient NFT used in flash loans (it's the same for all of the coins).  

### update_time_limits

The component owner can call this method to set the lower limits for the timings of fair launches.

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "update_time_limits"
    <MIN_LAUNCH_DURATION>i64
    <MIN_LOCK_DURATION>i64
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<MIN_LAUNCH_DURATION>` is the minimum possible duration of a launch phase expressed in seconds (by default one week).  
`<MIN_LOCK_DURATION>` is the minimum possible duration of the lock period for creator coins (by default 60 days).  

### launch

The creator of a coin can call this method to start the launching phase of his fair launched coin.  
The minimum possible values for the arguments of this method depends on the values the component owner specified in `update_time_limits`.

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
    "launch"
    Proof("creator_proof")
    <END_LAUNCH_TIME>i64
    <UNLOCKING_TIME>i64
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<CREATOR_BADGE_ADDRESS>` is the badge receaved when creating the coin.  
`<CREATOR_BADGE_ID>` is the numeric ID of the badge received when creating the coin.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<END_LAUNCH_TIME>` is the earliest time (expressed in seconds since Unix epoch) when the creator can close the launch phase.  
`<UNLOCKING_TIME>` is the time (expressed in seconds since Unix epoch) when all creator coins will be claimable.  

### terminate_launch

The creator of a coin can call this method to end the launching phase of his fair launched coin, it can't happen before the time specified in the `launch` call.

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
    "terminate_launch"
    Proof("creator_proof")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<CREATOR_BADGE_ADDRESS>` is the badge receaved when creating the coin.  
`<CREATOR_BADGE_ID>` is the numeric ID of the badge received when creating the coin.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  

This call returns the proceeds of the launch sale and starts the unlocking period.  

### unlock

Allows the creator of a fair launched coin to withdraw his previously locked coins.  
This method can only be called in Normal mode. If a pool gets into Liquidation mode it will never be possible to withdraw the creator's coin.  

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
    "unlock"
    Proof("creator_proof")
    Some(Decimal("<AMOUNT>"))
    <SELL>
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<CREATOR_BADGE_ADDRESS>` is the badge receaved when creating the coin.  
`<CREATOR_BADGE_ID>` is the numeric ID of the badge received when creating the coin.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<AMOUNT>` is the amount of coins the creator wants to withdraw. Instead of `Some(Decimal("<AMOUNT>"))` it is possible to specify `None` to withdraw all of the available coins.  
`<SELL>` is a boolean specifying if the withdrawed coins must be sold for base coins.

Depending on the vaule of `<SELL>` a bucket of coins or a bucket of base coins is returned.
If `<SELL>` is true, a `SellEvent` is issued. It contains the resource address of the sold coin, the pool mode (Normal), the sold amount, the new price, the number of coins currently in the pool and the fees paid to the pool.  

### register_hook

The component owner can call this method to make an hook available to the creators and to himself.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "register_hook"
    "<HOOK_NAME>"
    Array<String>("<OPERATION>", "<OPERATION>", ...)
    Address("<HOOK_ADDRESS>")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<HOOK_NAME>` is the name that will be used to refer to this hook.  
`<OPERATION>` is one of the operations the hooks can be attached to. Available operations are `PostFairLaunch`, `PostTerminateFairLaunch`, `PostQuickLaunch`, `PostBuy`, `PostSell` and `PostReturnFlashLoan`.  
`<HOOK_ADDRESS>` is the component address of the hook.  

### unregister_hook

The component owner can use this method to remove an hook that was previously registered or to make it not available for one or more operations.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "unregister_hook"
    "<HOOK_NAME>"
    Some(Array<String>("<OPERATION>", "<OPERATION>", ...))
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<HOOK_NAME>` is the name of a previously registered hook.  
`<OPERATION>` is one of the operations the hooks can no longer be attached to. If instead of this argument `None` is passed, RadixPump will completely forgot about the hook.  

### owner_enable_hook

The component owner can use this method to attach an hook to one or more operations globally (all the pools).  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "owner_enable_hook"
    "<HOOK_NAME>"
    Array<String>("<OPERATION>", "<OPERATION>", ...)
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<HOOK_NAME>` is the name of a previously registered hook.  
`<OPERATION>` is one of the operations the hooks gets attached to.  

A `HookEnabledEvent` is issued; it contains the hook name, the hook address and the list of operations it has been attached to.  

### owner_disable_hook

The component owner can use this method to detach an hook from one or more operations globally.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "owner_disable_hook"
    "<HOOK_NAME>"
    Array<String>("<OPERATION>", "<OPERATION>", ...)
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<HOOK_NAME>` is the name of a previously registered hook.  
`<OPERATION>` is one of the operations the hooks gets attached to.  

A `HookDisabledEvent` is issued; it contains the hook name, the hook address and the list of operations it has been detached from.  

### creator_enable_hook

A coin creator can call this method to attach a previously registered hook to one or more operations on his pool.  

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
    "creator_enable_hook"
    Proof("creator_proof")
    "<HOOK_NAME>"
    Array<String>("<OPERATION>", "<OPERATION>", ...)
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<CREATOR_BADGE_ADDRESS>` is the badge receaved when creating the coin.  
`<CREATOR_BADGE_ID>` is the numeric ID of the badge received when creating the coin.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<HOOK_NAME>` is the name of a previously registered hook.  
`<OPERATION>` is one of the operations the hooks gets attached to.  

A `HookEnabledEvent` is issued; it contains the coin resource address, the hook name, the hook address and the list of operations it has been attached to.  

### creator_disable_hook

A coin creator can call this method to detach a previously attached hook from one or more operations on his pool.  

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
    "creator_disable_hook"
    Proof("creator_proof")
    "<HOOK_NAME>"
    Array<String>("<OPERATION>", "<OPERATION>", ...)
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<CREATOR_BADGE_ADDRESS>` is the badge receaved when creating the coin.  
`<CREATOR_BADGE_ID>` is the numeric ID of the badge received when creating the coin.  
`<COMPONENT_ADDRESS>` is the address of the RadixPump component.  
`<HOOK_NAME>` is the name of a previously attached hook.  
`<OPERATION>` is one of the operations the hooks gets detached from.  

A `HookDisabledEvent` is issued; it contains the coin resource address, the hook name, the hook address and the list of operations it has been detached from.  

## Copyright

See LICENSE
