#!/bin/bash
set -e

cargo run --bin monitor ${1:-1}