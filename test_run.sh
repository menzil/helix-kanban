#!/bin/bash
# Test script to run the application briefly

cargo build --release && {
    echo "Build successful!"
    echo ""
    echo "Application compiled successfully with dialog system."
    echo ""
    echo "To test the application, run: cargo run"
    echo ""
    echo "Available keyboard shortcuts:"
    echo "  Space p n - Create new project (opens input dialog)"
    echo "  Space p o - Open project (opens selection dialog)"
    echo "  Space w v - Split horizontal"
    echo "  Space w s - Split vertical"
    echo "  j/k - Navigate tasks"
    echo "  h/l - Navigate columns"
    echo "  q - Quit"
}
