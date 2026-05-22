#!/usr/bin/bash

echo $1
curl "https://es.wikipedia.org/w/api.php?action=parse&prop=wikitext&redirects=true&format=json&page=$1" -o out.json
