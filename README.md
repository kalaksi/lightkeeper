# Design goals for Lightkeeper

## General
- Make customizations easy so that it can be modified to different needs.
- Make it easily extendable.
- Avoid sending too many commands to keep target host logs cleaner and traffic minimal.

## UI
- Aim for compact style. Avoid including too much information to keep the UI simple.
- Keep the amount of navigating and clicking to a minimum.

# Dependencies

## Requirements
- Currently only Linux is supported
- Qt 5.15

## Ubuntu 22.04 packages
- liboping0
- qtdeclarative5-dev

# Post-install

If you're using the ping monitor, you need to give the lighthouse binary more networking privileges:
```
$ setcap cap_net_raw+ep $MY_BINARY
```
