#!/usr/bin/bash

echo $1
echo $2
curl "https://$1.wikipedia.org/w/api.php?action=parse&prop=wikitext&redirects=true&format=json&page=$2" -o out.json
