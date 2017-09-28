#!/bin/bash

set -e

cd "$(dirname "$0")/.."

capnpc \
    --import-path=$JAVA_SCHEMA_PATH \
    --output=rust:libsolver/src \
    --src-prefix=common \
    common/api.capnp
