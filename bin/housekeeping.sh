#!/usr/bin/env zsh

# Strict mode
set -euo pipefail

echo "🧹 Running housekeeping checks..."

echo "📝 Formatting code..."
cargo fmt --quiet

echo "🔍 Running clippy checks..."
cargo clippy --workspace --all-targets --all-features -- \
  -W clippy::pedantic \
  -W clippy::style \
  -W clippy::unwrap_used \
  -W clippy::expect_used \
  -W clippy::allow_attributes

echo "✅ Housekeeping complete!"