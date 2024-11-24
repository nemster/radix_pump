# Timer blueprint

This blueprint can schedule the execution of hooks of the RadixPump system.  

The Dca hook is ment to be invoked by the Timer.  

## Transaction manifests

### Instantiate

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<RADIX_PUMP_COMPONENT>")
    "get_badges"
;
TAKE_ALL_FROM_WORKTOP
    Address("<PROXY_BADGE_ADDRESS>")
    Bucket("proxy_badge")
;
TAKE_ALL_FROM_WORKTOP
    Address("<HOOK_BADGE_ADDRESS>")
    Bucket("hook_badge")
;
CALL_FUNCTION
    Address("")
    "Timer"
    "new"
    Address("<OWNER_BADGE_ADDRESS>")
    Address("<RADIX_PUMP_COMPONENT>")
    Bucket("proxy_badge")
    Bucket("hook_badge")
    <MAX_HOURLY_FREQUENCY>u8
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<RADIX_PUMP_COMPONENT>` is the address of the RadixPump component.  
`<PROXY_BADGE_ADDRESS>` is the resource address of the proxy badge minted by the RadixPump component.  
`<HOOK_BADGE_ADDRESS>` is the resource address of the hook badge minted by the RadixPump component.  
`<MAX_HOURLY_FREQUENCY>` is the maximum hourly frequency that users can schedule their tasks (from 1 to 60).  

The function returns an alarm clock badge that can be used to invoke the `alarm_clock` method.  

### register_hook

The component owner can call this method to allow users to call a hook.

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<TIMER_COMPONENT>)
    "register_hook"
    "<HOOK_NAME>"
    Address("<HOOK_COMPONENT>")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<TIMER_COMPONENT>` is the address of the Timer component.  
`<HOOK_NAME>` is the name that will be used to refer to this hook.  
`<HOOK_COMPONENT>` is the component address of the hook.  

### unregister_hook

The component owner can call this method to disallow the use of a previously registered hook.

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<TIMER_COMPONENT>)
    "unregister_hook"
    "<HOOK_NAME>"
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<TIMER_COMPONENT>` is the address of the Timer component.  
`<HOOK_NAME>` is the name that was assigned to the hook by the `register_hook` method.  

### get_alarm_clock_badge

The component owner can call this method to get an additional alarm clock badge.

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<TIMER_COMPONENT>)
    "get_alarm_clock_badge"
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the account containing the owner badge.  
`<OWNER_BADGE_ADDRESS>` is the resource address of a badge that was specified when creating the component.  
`<TIMER_COMPONENT>` is the address of the Timer component.  

### alarm_clock

This method is ment to be programmatically called by a software according the schedule of a task defined by the user who created the task.  
The Timer component makes no check that the schedule is respected (because of the random delay) so it's responsibility of the calling software to ensure it.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<ALARM_CLOCK_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<TIMER_COMPONENT>)
    "alarm_clock"
    <TASK_ID>u64
;
```

`<ACCOUNT_ADDRESS>` is the account containing the alarm clock badge.  
`<ALARM_CLOCK_BADGE_ADDRESS>` is the resource address of the alarm clock badge.  
`<TIMER_COMPONENT>` is the address of the Timer component.  
`<TASK_ID>` is the numeric id of the scheduled task to execute.  

### new_task

Users can call this method to schedule their tasks

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3")
    Decimal("<XRD_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3")
    Bucket("xrd_bucket")
;
CALL_METHOD
    Address("<TIMER_COMPONENT>)
    "new_task"
    "<MINUTE>"
    "<HOUR>"
    "<DAY_OF_MONTH>"
    "<MONTH>"
    "<DAY_OF_WEEK>"
    "<RANDOM_DELAY>"
    "<HOOK_NAME>"
    Address("<COIN_ADDRESS>")
    Bucket("xrd_bucket")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the account of the user.  
`<XRD_AMOUNT>` is the amount of XRD to deposit to pay network fees for the scheduled tasks.  
`<TIMER_COMPONENT>` is the address of the Timer component.  
`<MINUTE> <HOUR> <DAY_OF_MONTH> <MONTH> <DAY_OF_WEEK>` is the specification of the task schedule. It must have the same format as the Unix crontab: https://en.wikipedia.org/wiki/Cron. Frequecy must not be bigger than the `<MAX_HOURLY_FREQUENCY>` specified in the component instantiation.  
`<RANDOM_DELAY>` Random delay, in seconds, to avoid being front run.  
`<HOOK_NAME>` The name under which the hook was registered.  
`<COIN_ADDRESS>` The resource address of the coin to be passed in the hook invocation.  

The user receives a timer badge that contains all of the info about the scheduled task.  

### change_schedule

Users can invoke this method to update the schedule of a previously created task.  

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
    Address("<TIMER_COMPONENT>)
    "change_schedule"
    Proof("timer_badge_proof")
    "<MINUTE>"
    "<HOUR>"
    "<DAY_OF_MONTH>"
    "<MONTH>"
    "<DAY_OF_WEEK>"
    "<RANDOM_DELAY>"
;
```

`<ACCOUNT_ADDRESS>` is the account of the user.  
`<TIMER_BADGE_ADDRESS>` is the resource address of the timer badge minted by the `new_task` method.  
`<TIMER_BADGE_ID>` is numeric id of the timer badge minted by the `new_task` method.  
`<TIMER_COMPONENT>` is the address of the Timer component.  
`<MINUTE> <HOUR> <DAY_OF_MONTH> <MONTH> <DAY_OF_WEEK>` is the specification of the new task schedule. It must have the same format as the Unix crontab: https://en.wikipedia.org/wiki/Cron. Frequecy must not be bigger than the `<MAX_HOURLY_FREQUENCY>` specified in the component instantiation.  
`<RANDOM_DELAY>` Random delay, in seconds, to avoid being front run.  

### add_gas

Users can invoke this method to deposit additional XRD to pay the network fees of their tasks.  

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
    Address("resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3")
    Decimal("<XRD_AMOUNT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3")
    Bucket("xrd_bucket")
;
CALL_METHOD
    Address("<TIMER_COMPONENT>)
    "add_gas"
    Proof("timer_badge_proof")
    Bucket("xrd_bucket")
;
```

`<ACCOUNT_ADDRESS>` is the account of the user.  
`<TIMER_BADGE_ADDRESS>` is the resource address of the timer badge minted by the `new_task` method.  
`<TIMER_BADGE_ID>` is numeric id of the timer badge minted by the `new_task` method.  
`<TIMER_COMPONENT>` is the address of the Timer component.  
`<XRD_AMOUNT>` is the amount of XRD to deposit to pay network fees for the scheduled tasks.  

### remove_task

Users can invoke this method to delete a scheduled task and withdraw the remaining deposited XRD.  

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
    Address("<TIMER_COMPONENT>)
    "remove_task"
    Proof("timer_badge_proof")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>` is the account of the user.  
`<TIMER_BADGE_ADDRESS>` is the resource address of the timer badge minted by the `new_task` method.  
`<TIMER_BADGE_ID>` is numeric id of the timer badge minted by the `new_task` method.  
`<TIMER_COMPONENT>` is the address of the Timer component.  
