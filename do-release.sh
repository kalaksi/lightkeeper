#!/usr/bin/env bash
set -euo pipefail

new_version=$1
version_only=${new_version#*v}
minor_version=${new_version%.*}
new_date=$(date +%Y-%m-%d)

cd "$(dirname "$0")"

if [ ! -z "$(git status -s)" ]; then
    echo "Uncommitted changes, aborting"
fi

echo -e "\n* Updating Cargo.toml..."
sed -i 's|^version = ".*|version = "'$version_only'"|' Cargo.toml

# Test build and update cargo.lock
cargo build

pushd flatpak &>/dev/null

echo -e "\n* Updating flatpak cargo-source and metainfo files..."
python3 ./flatpak-cargo-generator/flatpak-cargo-generator.py ../Cargo.lock -o cargo-sources.json
sed -i '/<releases>$/a \    <release version="'$version_only'" date="'$new_date'">'"\n      <description>\n      </description>\n    </release>" io.github.kalaksi.Lightkeeper.metainfo.xml

read -p "Edit metainfo now by pressing enter" _temp
$EDITOR io.github.kalaksi.Lightkeeper.metainfo.xml
xmllint io.github.kalaksi.Lightkeeper.metainfo.xml >/dev/null

echo -e "\n* Committing changes..."
git commit -a -m "Prepare for version $new_version"
git push

echo -e "\n* Creating a git tag..."
git tag -a "$new_version" -m "Version $new_version"
git push origin "$new_version"

echo -e "\n* Updating io.github.kalaksi.Lightkeeper.yml..."
sed -ri 's|(\s+url: https://github.com/kalaksi/lightkeeper/archive/refs/tags/).*|\1'$new_version'.tar.gz|' io.github.kalaksi.Lightkeeper.yml
new_checksum=$(wget -qO- "https://github.com/kalaksi/lightkeeper/archive/refs/tags/$new_version.tar.gz"|sha256sum|cut -f 1 -d ' ')
sed -ri '/url: https:\/\/github.com\/kalaksi\/lightkeeper\/archive/{n;s/(\s+sha256: ).*/\1'$new_checksum'/}' io.github.kalaksi.Lightkeeper.yml
git commit -a -m "Update flatpak manifest for $new_version"
git push

popd &>/dev/null

if [ -d ../io.github.kalaksi.Lightkeeper ]; then
    echo -e "\n* Updating flatpak-repo"
    cp -v flatpak/cargo-sources.json ../io.github.kalaksi.Lightkeeper/
    cp -v flatpak/io.github.kalaksi.Lightkeeper.yml ../io.github.kalaksi.Lightkeeper/
    cd ../io.github.kalaksi.Lightkeeper

    current_branch=$(git branch --show-current)
    if [ $current_branch != $minor_version ]; then
        # Will fail if branch already exists.
        git checkout -b $minor_version
    fi
    git commit -a -m "Update to version $version_only"
    cd -
fi
