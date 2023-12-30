#!/usr/bin/env -S just --justfile
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set dotenv-load := true
export CARGO_TERM_COLOR := "always"

# Show available commands
default:
    @just --list --justfile {{justfile()}}

# Run tests using cargo test
test:
    cargo nextest run
