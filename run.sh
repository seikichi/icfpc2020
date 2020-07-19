#!/bin/sh

echo "$(date -u --rfc-3339=ns): Start" >&2

REPO_TOP="$(cd $(dirname "$0"); pwd)"
"$REPO_TOP/target/release/ships" "$@" || echo "run error code: $?"
