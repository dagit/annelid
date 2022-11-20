#!/bin/bash
HERE="$(cd $(dirname "${BASH_SOURCE[0]}") && pwd)"
echo HERE=$HERE

set -e

# This isn't actually meant to be run on linux, the linux commands are for
# testing and developing the script on linux. So they just echo the commands.

function install_name {
  if [ $(uname) == "Linux" ]
  then
    echo install_name_tool $*
  elif [ $(uname) == "Darwin" ]
  then
    #echo install_name_tool $*
    install_name_tool $*
  fi
}

function get_deps {
  if [ $(uname) == "Linux" ]
  then
    ldd "$1" | grep "=>" | awk '{ print $3 }' | xargs
  elif [ $(uname) == "Darwin" ]
  then
    #echo otool -L "$1"
    otool -L "$1" | grep "/*.*dylib" -o | grep -v '/usr/lib' | grep -v '/System/Library' | xargs
  fi
}

function copy_file {
  if [ $(uname) == "Linux" ]
  then
    echo cp $*
  elif [ $(uname) == "Darwin" ]
  then
    #echo cp $*
    ls -lh "$1" "$2"
    cp $*
  fi
}

function copy_dep {
  local source="$1"
  local target="$2/$(dirname $source)"
  mkdir -p "$target"
  copy_file "$source" "$target"
  install_name -change "$source" "@executable_path/../Resources/libs/$source" "$3"

  deps=$(get_deps "$source")
  for d in $deps
  do
    if [ ! -f "$2/$d" ]
    then
      copy_dep "$d" "$2" "$2/$d"
    fi
  done
}

binary=$HERE/../target/release/bundle/osx/Annelid.app/Contents/MacOS/annelid
dest=$HERE/../target/release/bundle/osx/Annelid.app/Contents/Resources/libs
deps=$(get_deps "$binary")

for d in $deps
do
  copy_dep "$d" "$dest" "$binary"
done

