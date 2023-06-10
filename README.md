# Design goals

## General
- Make customizations easy so that it can be modified to different needs.
- Make it easily extendable.
- Avoid sending too many commands to keep target host logs cleaner and traffic minimal.

## UI
- Aim for compact style. Avoid including too much information to keep the UI simple.
- Keep the amount of navigating and clicking to a minimum.

# Installing
Currently only Linux is supported.

## Flatpak
It is recommended to download the app from Flathub.

# Building from source
## Flatpak
```
cd flatpak
flatpak-builder build --force-clean io.github.kalaksi.Lightkeeper.yml
# If you want to install also:
flatpak-builder --user --install --force-clean build io.github.kalaksi.Lightkeeper.yml
```

## Regular
Dependencies are:
- Qt 5.15
- liboping

Corresponding Ubuntu 22.04 packages are:
- qtdeclarative5-dev
- liboping0

Building:
```
cargo build
```

Running:
```
./target/debug/lightkeeper
```

### Post-install

If you're using the ping monitor, you need to give Lightkeeper binary more networking privileges:
```
$ setcap cap_net_raw+ep $MY_BINARY
```


# License
## Lightkeeper
This software is licensed under GNU General Public License 3.

## Crate dependencies
All crate dependencies contain permissive licenses (mostly MIT license).  
You can check the licenses with:
```
cargo tree --format "{p} {l}" --prefix none
```

Or, to quickly see what different licenses are being used by printing only unique license strings:
```
cargo tree --format "{l}" --prefix none | sort | uniq
```

## Liboping
Liboping 1.10 (https://noping.cc/) is redistributed inside the flatpak package.
It's a separate C library dependency needed by oping-crate and is distributed under LGPL-2.1 license.
