#!/usr/bin/env bash
set -eu
find src -name '*.rs' -print0 | while IFS= read -rd '' file; do
    if ! grep -q " Copyright (C) " "$file"; then
        cat "license-header.txt" "$file" > "$file.tmp"
        mv "$file.tmp" "$file"
    fi
done

