#!/bin/bash

# Release script for local_search

set -e

# Check if version is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.1"
    exit 1
fi

VERSION=$1

# Validate version format
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in format X.Y.Z"
    exit 1
fi

echo "Preparing release v$VERSION..."

# Update version in Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Run tests
echo "Running tests..."
cargo test
cargo test --features cli

# Run clippy and fmt
echo "Running clippy..."
cargo clippy -- -D warnings

echo "Running rustfmt..."
cargo fmt --all -- --check

# Build release
echo "Building release..."
cargo build --release
cargo build --release --features cli

# Generate docs
echo "Generating documentation..."
cargo doc --no-deps

# Commit and tag
echo "Creating git commit and tag..."
git add Cargo.toml
git commit -m "Release v$VERSION"
git tag "v$VERSION"

echo "Release v$VERSION prepared!"
echo "Run 'git push origin main --tags' to publish the release"
echo "Don't forget to:"
echo "  1. Create a GitHub release at https://github.com/YOUR_USERNAME/local_search/releases/new"
echo "  2. Set up CARGO_REGISTRY_TOKEN secret in GitHub for automatic publishing"