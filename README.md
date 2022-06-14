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
