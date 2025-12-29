#!/bin/bash
export COPYFILE_DISABLE=1
export COPY_EXTENDED_ATTRIBUTES_DISABLE=1

find src-tauri/target -name "._*" -type f -delete 2>/dev/null || true

node node_modules/@tauri-apps/cli/tauri.js build
