#!/bin/bash
set -e

echo "Building teacher-server..."
cd teacher-server
cargo build --release

echo "Creating data directory..."
mkdir -p data

echo "Build completed successfully!"
