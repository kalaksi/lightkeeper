# Flatpak packaging for Lightkeeper

Local `flatpak-builder` workflows. Manifest and related files live in this directory.

## Prerequisites

If you are missing the runtime, platform, WebEngine base app, and Rust SDK extension:

```bash
flatpak install --user runtime/org.kde.Sdk/x86_64/6.10
flatpak install --user org.kde.Platform/x86_64/6.10
flatpak install --user io.qt.qtwebengine.BaseApp/x86_64/6.10
flatpak install --user runtime/org.freedesktop.Sdk.Extension.rust-stable/x86_64/25.08
```

## Building

From the repository root:

```bash
flatpak-builder build --user --force-clean flatpak/io.github.kalaksi.Lightkeeper-local.yml
```

## Installing the build locally

From the repository root:

```bash
flatpak-builder --user --install --force-clean build flatpak/io.github.kalaksi.Lightkeeper-local.yml
```
