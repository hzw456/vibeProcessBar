#!/bin/bash

# Vibe Process Bar Build Script for macOS
# This script works around the macOS resource fork issue

echo "=== Vibe Process Bar Build Script ==="
echo ""

# Set environment variables to disable resource forks
export COPYFILE_DISABLE=1
export COPY_EXTENDED_ATTRIBUTES_DISABLE=1

# Navigate to project directory
cd /Users/zwhao/nfs/project/vibeProcessBar

# Stop any running build processes
echo "Stopping any running build processes..."
pkill -9 cargo 2>/dev/null
pkill -9 rustc 2>/dev/null
sleep 2

# Clean target directory
echo "Cleaning target directory..."
rm -rf src-tauri/target
sleep 2

# Build the project
echo "Building project..."
cd src-tauri

# Try building, and if it fails due to resource forks, clean and retry
for attempt in 1 2 3; do
    echo "Attempt $attempt..."
    
    # Clean any resource files that might have been created
    find . -name "._*" -type f -delete 2>/dev/null
    
    if cargo build 2>&1 | tee /tmp/tauri-build.log; then
        echo "Build successful!"
        
        # Check if the binary was created
        if [ -f target/debug/vibe-process-bar ]; then
            echo ""
            echo "=== Build Complete ==="
            echo "Binary created at: target/debug/vibe-process-bar"
            echo ""
            echo "To run the app:"
            echo "  ./target/debug/vibe-process-bar"
            exit 0
        fi
    else
        echo "Build failed, checking for resource fork issue..."
        
        # Check if the error is due to resource forks
        if grep -q "stream did not contain valid UTF-8" /tmp/tauri-build.log; then
            echo "Detected macOS resource fork issue. Cleaning and retrying..."
            
            # Clean target and resource files
            rm -rf target
            find . -name "._*" -type f -delete 2>/dev/null
            rm -rf ~/.cargo/registry/src/*/tauri-*/scripts/bundle.global.js 2>/dev/null
            
            sleep 2
        else
            echo "Build failed with a different error. See /tmp/tauri-build.log for details."
            tail -30 /tmp/tauri-build.log
            exit 1
        fi
    fi
done

echo "Build failed after 3 attempts. Please try manually cleaning the target directory."
echo "You can also try: rm -rf src-tauri/target && cd src-tauri && cargo build"
