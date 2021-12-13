#!/bin/bash
set -e

cargo run --bin server ${1:-0} ${2:-1} ${3:-BOOT}
