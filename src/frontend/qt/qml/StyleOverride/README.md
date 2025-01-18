For some reason, palette colors don't get applied correctly on KDE.
This directory therefore contains necessary patches to fix the colors. Usually they contain modified original implementation.

To make matters more weird, it seems that locally built flatpak applies palette correctly just by re-defining some components, but
the published flatpak doesn't seem to respect palette at all if not forcing the colors in components in StyleOverride.