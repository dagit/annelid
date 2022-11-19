#!/bin/bash
HERE="$(cd $(dirname "${BASH_SOURCE[0]}") && pwd)"
echo HERE=$HERE

set -ex

function copy_dep {
  local source="$1"
  local target="$2/$(dirname $source)"
  mkdir -p "$target"
  cp "$source" "$target"
  install_name_tool -change "$source" "@executable_path/../Resources/libs/$source" "$3"

  deps=$(otool -L "$source" | grep "/*.*dylib" -o | grep -v '/usr/lib' | grep -v '/System/Library')
  for d in $deps
  do
    if ! [[ -f "$2/$d" ]]
    then
      copy_dep "$d" "$2" "$2/$d"
    fi
  done

}

binary=$HERE/../target/release/bundle/osx/Annelid.app/Contents/MacOS/annelid
dest=$HERE/../target/release/bundle/osx/Annelid.app/Contents/Resources/libs
deps=$(otool -L $binary | grep "/*.*dylib" -o | grep -v '/usr/lib' | grep -v '/System/Library' | xargs)

for d in $deps
do
  copy_dep "$d" "$dest" "$binary"
done

