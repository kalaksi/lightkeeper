# LightkeeperRM

**LightkeeperRM (Remote Management) is a modular drop-in replacement for maintaining servers over SSH with shell commands.**
No additional daemons or other software is needed on target hosts. LightkeeperRM will only run standard Linux commands already available on the host.
You can see executed commands through debug log and on target hosts's logs (depending on setup), so it's easy to audit and debug.
Lightkeeper simplifies general maintenance tasks such as upgrades, ad-hoc monitoring, debugging and configuration.
  
**LightkeeperRM aims to be customizable**, modular and easily extendable, so that it can be modified for different needs.  
  
**User-interface is compact** and aims to keep only the most essential information visible. Clicking and navigating between different views is kept to a minimum.  
In case you find the GUI insufficient and need to dig deeper, you can always use a button, or hotkey, for launching a terminal that logs you in through SSH.

**Extensive keyboard shortcuts**.
  
*NOTE: this is currently a pre-release and still has bugs and is missing some features. Documentation is also not yet complete.*

<br />
<br />
<p align="center">
    <img src="doc/images/LightkeeperRM-overview.png" width="75%" />
    <br />
    <i>Overview of LightkeeperRM's GUI.</i>
</p>
<br />

## Some features
- Status summary in host table for quick status view
- Log viewer with regex search and hotkeys similar to less/vim.
- Text file editor for editing remote files (with built-in editor or using CLI over SSH).
- Follow console output for longer running commands such as container builds and package updates.

# Installing
## Flatpak
It is recommended to download the app from Flathub: https://flathub.org/apps/io.github.kalaksi.Lightkeeper  
It's sandboxed and uses the minimum amount of permissions required.

The alternative is building from source.

# Building from source
## Flatpak
```
flatpak-builder build --user --force-clean flatpak/io.github.kalaksi.Lightkeeper-local.yml
# If you want to install also:
flatpak-builder --user --install --force-clean build flatpak/io.github.kalaksi.Lightkeeper-local.yml
```

## Regular
Dependencies are:
- Qt 5.15
- liboping
- libdbus
- qmltermwidget

Corresponding Ubuntu 22.04 packages are:
- qtdeclarative5-dev
- liboping0
- libdbus-1-3, libdbus-1-dev
- qml-module-qmltermwidget

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

# Server OS support
The (current) goal is to support:
- Debian
- Ubuntu
- RHEL
- CentOS
- NixOS
- Linux in general (basic functionality for most distributions)


# Configuration
**NOTE: There is currently an assumption that your user can run sudo without password (or has root privileges) on the target host since some commands need higher privileges.**  
  
Configuration can now be done using the graphical UI, but configuring can always be done directly through configuration files, too.

## Configuration files
Example configuration files `config.example.yml`, `hosts.example.yml` and `groups.example.yml` can be found in the root of this repository.

When running without flatpak, the default configuration directory is usually `~/.config/lightkeeper` and cache directory `~/.cache/lightkeeper`.
With flatpak, it's the usual app specific directory: `~/.var/app/io.github.kalaksi.Lightkeeper/config` and `~/.var/app/io.github.kalaksi.Lightkeeper/cache`.
You can use a custom configuration directory with the `-c`/`--config-dir` option.

# Debug logging
Log levels are controlled with environment variable `RUST_LOG`, so use `RUST_LOG=debug`.


# Testing
`test`-directory contains Vagrantfiles for virtual machines and also matching LightkeeperRM configurations.  
Use `--config-dir` to load the test configuration. For example, `./target/debug/lightkeeper --config-dir test` if building from source.


# License
## Lightkeeper
Copyright © 2023 kalaksi@users.noreply.github.com.

This software is licensed under GNU General Public License 3.

**NOTE: If you need a non-GPL license, get in contact.**

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
