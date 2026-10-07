#!/bin/sh
# SPDX-License-Identifier: LGPL-3.0-only
set -eu
MCU="$1"
OUT="$2"
BUILD="$(mktemp -d)"
DIR="$(dirname "$0")"
javac -nowarn -d "$BUILD" $(find "$DIR/stubs" "$MCU/java" -name '*.java' ! -name '*Test*.java') "$DIR/Gen.java"
java -cp "$BUILD" Gen "$OUT"
