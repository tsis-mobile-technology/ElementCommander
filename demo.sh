#!/bin/bash
# Demo setup script for hermes_tail

set -e

echo "📁 Setting up test directory structure..."

# Create test directories
TEST_DIR="/tmp/hermes_demo"
mkdir -p "$TEST_DIR/project/src"
mkdir -p "$TEST_DIR/project/docs"
mkdir -p "$TEST_DIR/archive"
mkdir -p "$TEST_DIR/media"

# Create sample files
echo "Creating sample files..."
touch "$TEST_DIR/project/README.md"
touch "$TEST_DIR/project/Cargo.toml"
touch "$TEST_DIR/project/src/main.rs"
touch "$TEST_DIR/project/src/lib.rs"
touch "$TEST_DIR/project/docs/guide.md"
touch "$TEST_DIR/project/docs/api.md"

# Add some content
echo "# My Project" > "$TEST_DIR/project/README.md"
echo "fn main() { println!(\"Hello!\"); }" > "$TEST_DIR/project/src/main.rs"

for i in {1..5}; do
    dd if=/dev/zero bs=1K count=$((RANDOM % 100 + 10)) of="$TEST_DIR/archive/file_$i.bin" 2>/dev/null
done

touch "$TEST_DIR/media/image_1.jpg"
touch "$TEST_DIR/media/image_2.jpg"
touch "$TEST_DIR/media/video.mp4"

echo ""
echo "✅ Test directory created at: $TEST_DIR"
echo ""
echo "📋 Directory structure:"
ls -lR "$TEST_DIR" | head -30
echo ""
echo "🚀 To run hermes_tail:"
echo "   cargo run"
echo "   or"
echo "   ./target/release/hermes_tail"
echo ""
echo "📝 Keyboard controls:"
echo "   ↑↓     - Navigate"
echo "   Tab    - Switch panel"
echo "   Enter  - Open directory"
echo "   Bksp   - Go parent"
echo "   Insert - Select/Deselect"
echo "   Ctrl+A - Select all"
echo "   q/Esc  - Exit"
echo ""
echo "💡 Navigate to $TEST_DIR to test the file manager!"
