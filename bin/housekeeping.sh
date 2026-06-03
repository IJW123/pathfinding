#!/usr/bin/env bash

# Strict mode
set -euo pipefail

echo "🧹 Running housekeeping checks..."

echo "📝 Formatting code..."
cargo fmt --quiet

# Advisory pass: surface pedantic/style suggestions and unwrap/expect/allow usage.
# These are guidance ("avoid if possible"), not gates — clippy exits 0 on warnings.
echo "🔍 Clippy (advisory)..."
cargo clippy --workspace --all-targets --all-features -- \
  -W clippy::pedantic \
  -W clippy::style \
  -W clippy::unwrap_used \
  -W clippy::expect_used \
  -W clippy::allow_attributes

# Gate: any standard (default-level) warning fails the build. Keeps the tree clean
# without tripping on the 75 advisory pedantic nits above.
echo "🚦 Clippy gate (fail on any standard warning)..."
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "✅ Housekeeping complete!"
