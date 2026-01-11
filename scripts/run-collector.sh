#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
exec cargo run --release --manifest-path "$repo_dir/Cargo.toml"
