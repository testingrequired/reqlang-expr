#!/usr/bin/env bash

cargo llvm-cov nextest --lcov --output-path target/llvm-cov/lcov.info --ignore-filename-regex 'main|cliutil'
cargo llvm-cov report --html --open --output-dir target/llvm-cov