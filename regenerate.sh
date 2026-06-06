#!/usr/bin/bash

if [ "$1" == "" ]; then
    echo Usage: $0 examples/planets.yaml
    exit 1
fi

if [ ! -f "$1" ]; then
    echo File $1 does not exist.
    exit 2
fi

echo $1

cargo run -- --output demo.epub --local pages/  $1

name=$(basename $1 | cut -f1 -d'.')
#echo $name
cd expected/$name
unzip -o ../../demo.epub
cd -

