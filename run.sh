#!/bin/sh

REPO_TOP="$(cd $(dirname "$0"); pwd)"
"$REPO_TOP/target/release/app" "$@" || echo "run error code: $?"
