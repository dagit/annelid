#!/bin/bash
HERE="$(cd $(dirname "${BASH_SOURCE[0]}") && pwd)"
echo HERE=$HERE

set -ex

function copy_dep {
  local d="$1"
  local target="$2/$(dirname $d)"
  mkdir -p "$target"
  cp "$d" "$target"
  install_name_tool -change "$d" "@executable_path/../Resources/libs/$d" "$3"

  deps=$(otool -L "$d" | grep "/*.*dylib" -o | grep -v '/usr/lib' | grep -v '/System/Library')
  for d in $deps
  do
    local target="$2/$(dirname $d)"
    if ! [[ -f $target ]]
    then
      copy_dep "$d" "$2" "$2/$d"
    fi
  done

}

binary=$HERE/../targets/release/bundle/osx/Annelid.app/Contents/MacOS/annelid
dest=$HERE/../targets/release/bundle/osx/Annelid.app/Contents/Resources/libs
deps=$(otool -L $binary | grep "/*.*dylib" -o | grep -v '/usr/lib' | grep -v '/System/Library' | xargs)

for d in $deps
do
  copy_dep "$d" "$dest" "$binary"
done

