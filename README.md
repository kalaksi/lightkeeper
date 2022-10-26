# Design goals

## General
- Make customizations easy so that the tool can be modified to different needs.
- Make the tool extendable.

## UI
- Keep the amount of navigating and clicking to a minimum.
- Avoid including too much non-essential information to keep the UI simple.

# Dependencies
## Requirements
- Qt 5.15

## Ubuntu 22.04 packages
- liboping0
- qtdeclarative5-dev

# Post-install

If you're using the ping monitor, you need to give the lighthouse binary more networking privileges:
```
$ setcap cap_net_raw+ep $MY_BINARY
```
