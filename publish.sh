#!/usr/bin/env bash

set -e

_DIR=$(dirname $(realpath "$0"))

cd $_DIR/sdb_macro

cargo +nightly publish

cd $_DIR/sdb

cargo +nightly publish
