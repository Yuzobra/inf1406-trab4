#!/bin/bash
set -e

cargo run --bin client ${1:-INSERT} ${2:-CHAVE} ${3:-9999} ${4:-0} ${5:--1} ${6:--1}