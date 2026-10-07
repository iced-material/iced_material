#!/bin/sh
# SPDX-License-Identifier: LGPL-3.0-only
set -eu
OUT="$1"
shift
BASE=https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web
mkdir -p "$OUT"
for name in "$@"; do
    curl -sfL -o "$OUT/$name.svg" "$BASE/$name/materialsymbolsoutlined/${name}_24px.svg"
    curl -sfL -o "$OUT/${name}_fill1.svg" "$BASE/$name/materialsymbolsoutlined/${name}_fill1_24px.svg"
done
