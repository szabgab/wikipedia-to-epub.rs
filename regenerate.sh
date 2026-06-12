#!/usr/bin/bash -e

function regenerate() {
    echo $filename;
    rm -f demo.epub
    cargo run -- --output demo.epub --local pages/ $filename

    name=$(basename $filename | cut -f1 -d'.')
    echo $name
    cd expected/$name
    unzip -o ../../demo.epub
    cd -
}

if [ "$1" == "" ]; then
    echo "Usage: $0 [all|examples/planets.yaml]"
    exit 1
fi


if [ "$1" == "all" ]; then
    for filename in examples/*.yaml;
        do regenerate
    done
else
    if [ ! -f "$1" ]; then
        echo File $1 does not exist.
        exit 2
    fi
    filename=$1
    regenerate
fi



