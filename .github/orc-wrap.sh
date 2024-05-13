#!/bin/sh
# Determine the network from the version string;
# if it's unusual (i.e. neither mainnet nor testnet), call the orc tool specially.

VERSION="$(cargo metadata --format-version 1 --no-deps | jq -r '.packages | .[0] | .version')"
PRE="$(echo "$VERSION" | cut -s -d- -f2-)"

if [ -z "$PRE" ] || (echo "$PRE" | grep -q "testnet"); then
  orc "$@"
else
  RUNTIMEID="$(grep -A 5 "\\[package.metadata.orc.$PRE\\]" Cargo.toml | grep "runtime-id" | head -n 1 | grep -o '"[0-9a-fA-F]*"' | tr -d '"')"
  orc "$@" --runtime-id "$RUNTIMEID"
fi
