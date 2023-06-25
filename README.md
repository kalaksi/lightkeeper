# LightkeeperRM

**LightkeeperRM (Remote Management) is a drop-in replacement for maintaining servers over SSH with shell commands.**  
No additional daemons or other software is needed on target servers. LightkeeperRM will only run standard Linux commands already available on the target server.
You can see executed commands through debug log and, of course, on target server's logs, so it's easy to audit and debug.  
  
**LightkeeperRM aims to be customizable**, modular and easily extendable, so that it can be modified for different needs.  
  
**User-interface is compact** and aims to keep only the most essential information visible. Clicking and navigating between different views is kept to a minimum.  
In case you find the GUI insufficient and need to dig deeper, you can always find a convenience button for launching a terminal that logs you in through SSH.
You can, for example, launch a terminal with a shell inside a Docker container, with a single click.  

<br />
<br />
<p align="center">
    <img src="doc/images/LightkeeperRM-overview.png" width="75%" />
    <br />
    <i>Overview of LightkeeperRM's GUI.</i>
</p>
<br />


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


# Configuration

## File locations
When running without flatpak, the default configuration directory is `~/.config/lightkeeper` and cache directory `~/.cache/lightkeeper`.  
With flatpak, it's the usual app specific directory: `~/.var/app/io.github.kalaksi.Lightkeeper/config` and `~/.var/app/io.github.kalaksi.Lightkeeper/cache`.

You can point to a custom configuration directory with the `-c`/`--config-dir` options.

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
