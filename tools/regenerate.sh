#!/usr/bin/bash -e

function regenerate() {
    echo "##################### Filename: $filename"
    if [ "$filename" == "examples/all-en.yaml" ]; then
        echo "skipping $filename"
        return;
    fi
    rm -f demo.epub
    cargo run -- --output demo.epub --local pages/ $filename > /dev/null 2> /dev/null

    name=$(basename $filename | cut -f1 -d'.')
    echo "********************* Name: $name"
    cd expected/$name
    unzip -o ../../demo.epub > /dev/null 2> /dev/null
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



