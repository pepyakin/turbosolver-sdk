#!/bin/bash

set -e

cd "$(dirname "$0")/.."

if [ -z "$JAVA_SCHEMA_PATH" ]; then
    echo "You must provide JAVA_SCHEMA_PATH. See https://dwrensha.github.io/capnproto-java/index.html"
    exit 1
fi

capnpc \
    --import-path=$JAVA_SCHEMA_PATH \
    --output=java:android-demo/app/src/main/java/me/pepyakin/turbosolver/capnp \
    --output=rust:libsolver/src \
    --src-prefix=common \
    common/api.capnp
