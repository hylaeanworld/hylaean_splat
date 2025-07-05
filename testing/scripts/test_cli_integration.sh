#!/bin/bash
set -e

echo "=== Testing Hylaean Splat CLI Integration ==="

# Build the project
echo "Building Hylaean Splat..."
cargo build --release

# Initialize configuration
echo "Initializing Hylaean Splat..."
./target/release/hylaeansplat init --force

# Test basic CLI functionality
echo "Testing CLI help..."
./target/release/hylaeansplat --help

echo "Testing tool discovery..."
./target/release/hylaeansplat tool discover

echo "Listing available tools..."
./target/release/hylaeansplat list --detailed

# Test if COLMAP is available
echo "Testing COLMAP integration..."
if command -v colmap &> /dev/null; then
    echo "✓ COLMAP is installed and available"
    
    # Test COLMAP version through our CLI
    ./target/release/hylaeansplat tool run colmap --version || echo "COLMAP version check failed (this might be expected)"
else
    echo "✗ COLMAP not found in PATH"
    echo "Install COLMAP first with: sudo apt install colmap (Ubuntu) or brew install colmap (macOS)"
fi

# Test if Rust/Cargo is available for Brush
echo "Testing Brush dependencies..."
if command -v cargo &> /dev/null; then
    echo "✓ Rust/Cargo is available for Brush"
else
    echo "✗ Rust/Cargo not found"
    echo "Install Rust first with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi

echo "=== CLI Integration Test Complete ==="
echo "If no errors appeared above, your CLI integration is working!"