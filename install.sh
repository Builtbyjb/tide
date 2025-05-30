#!/usr/bin/env bash

set -e

# if os == mac
curl -LsSf -o tide https://github.com/builtbyjb/tide/releases/download/v0.1.0/tide-macos-arm64

# if os == linux
curl -LsSf -o tide https://github.com/builtbyjb/tide/releases/download/v0.1.0/tide-linux-x86_64

