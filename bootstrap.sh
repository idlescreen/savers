#!/usr/bin/env bash
set -euo pipefail

if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update && sudo apt-get install -y libdbus-1-dev libwayland-dev libxkbcommon-dev libssl-dev pkg-config libudev-dev
fi

if [ ! -d "../runtime" ] && [ "$(basename "$PWD")" != "runtime" ]; then git clone https://github.com/idlescreen/runtime ../runtime; fi

# Path deps resolve via the repo-local `runtime/` — Cargo.toml declares
# `idle-api = { path = "runtime/idle-api" }`, so the link MUST be named
# `runtime`, not something else. CI checks out the engine there; locally it is
# a gitignored symlink to the sibling clone. (.gitignore already lists it.)
if [ ! -e runtime ] && [ -d ../runtime ]; then ln -s ../runtime runtime; fi

if ! command -v rustup >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi
fi

rustup show

cargo test --workspace
