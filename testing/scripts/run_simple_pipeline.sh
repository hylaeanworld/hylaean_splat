#!/bin/bash
set -e

# Simple pipeline test for COLMAP + Brush integration
# Usage: ./run_simple_pipeline.sh <input_images_dir> [output_dir]

if [ $# -lt 1 ]; then
    echo "Usage: $0 <input_images_dir> [output_dir]"
    echo "Example: $0 ./my_test_images ./output"
    echo ""
    echo "Input directory should contain 10-20 JPG/PNG images of the same object from different angles"
    exit 1
fi

INPUT_DIR="$1"
OUTPUT_DIR="${2:-./testing/outputs/simple_test}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "=== Hylaean Splat Simple Pipeline Test ==="
echo "Input: $INPUT_DIR"
echo "Output: $OUTPUT_DIR"
echo "Timestamp: $TIMESTAMP"

# Validate input directory
if [ ! -d "$INPUT_DIR" ]; then
    echo "Error: Input directory '$INPUT_DIR' does not exist"
    exit 1
fi

# Count images
IMAGE_COUNT=$(find "$INPUT_DIR" -name "*.jpg" -o -name "*.jpeg" -o -name "*.png" | wc -l)
echo "Found $IMAGE_COUNT images"

if [ $IMAGE_COUNT -lt 8 ]; then
    echo "Warning: Less than 8 images found. COLMAP may fail to reconstruct."
    echo "Recommended: 10-20 images with good overlap"
fi

# Create output directories
mkdir -p "$OUTPUT_DIR"/{colmap,brush_model,renders}

# Build Hylaean Splat
echo "Building Hylaean Splat..."
cargo build --release

# Initialize if needed
echo "Initializing Hylaean Splat..."
./target/release/hylaean init --force

echo "=== Step 1: COLMAP Structure-from-Motion ==="
echo "Running COLMAP full pipeline..."

# Run COLMAP full pipeline
if ./target/release/hylaean tool run colmap full_pipeline "$INPUT_DIR" "$OUTPUT_DIR/colmap"; then
    echo "✓ COLMAP reconstruction completed successfully"
    
    # Check if reconstruction was successful
    if [ -f "$OUTPUT_DIR/colmap/sparse/0/cameras.bin" ] && [ -f "$OUTPUT_DIR/colmap/sparse/0/images.bin" ] && [ -f "$OUTPUT_DIR/colmap/sparse/0/points3D.bin" ]; then
        echo "✓ COLMAP sparse reconstruction files found"
        
        # Count reconstructed points
        if command -v colmap &> /dev/null; then
            POINT_COUNT=$(colmap model_analyzer \
                --path "$OUTPUT_DIR/colmap/sparse/0" \
                --database_path "$OUTPUT_DIR/colmap/database.db" 2>/dev/null | grep "Points" | head -1 | awk '{print $2}' || echo "unknown")
            echo "Reconstructed points: $POINT_COUNT"
        fi
    else
        echo "✗ COLMAP reconstruction failed - sparse files not found"
        exit 1
    fi
else
    echo "✗ COLMAP pipeline failed"
    exit 1
fi

echo "=== Step 2: Brush Gaussian Splatting Training ==="
echo "Training Gaussian Splat model with Brush..."

# Check if Brush is available or can be built
if ! command -v brush &> /dev/null; then
    echo "Brush executable not found in PATH, checking for local installation..."
    
    if [ -d "./tools/brush" ]; then
        echo "Found Brush source at ./tools/brush"
        cd ./tools/brush
        if [ ! -f "target/release/brush" ]; then
            echo "Building Brush..."
            cargo build --release
        fi
        cd - > /dev/null
        BRUSH_EXEC="./tools/brush/target/release/brush"
    else
        echo "Brush not found. You can install it with:"
        echo "  ./target/release/hylaean tool install brush_app --path ./tools"
        echo "Or install manually from: https://github.com/ArthurBrussee/brush"
        echo "Skipping Brush training step..."
        exit 0
    fi
else
    BRUSH_EXEC="brush"
fi

# Try to run Brush training via our CLI
if ./target/release/hylaean tool run brush_app train "$OUTPUT_DIR/colmap" "$OUTPUT_DIR/brush_model"; then
    echo "✓ Brush training completed successfully"
    
    echo "=== Step 3: Rendering Test Views ==="
    echo "Rendering novel views..."
    
    if ./target/release/hylaean tool run brush_app render "$OUTPUT_DIR/brush_model" "$OUTPUT_DIR/renders"; then
        echo "✓ Rendering completed successfully"
        
        # Count rendered images
        RENDER_COUNT=$(find "$OUTPUT_DIR/renders" -name "*.png" | wc -l)
        echo "Generated $RENDER_COUNT rendered views"
    else
        echo "✗ Rendering failed, but training was successful"
    fi
else
    echo "Note: Brush integration may need additional setup"
    echo "COLMAP reconstruction completed successfully and can be used with other tools"
fi

echo "=== Pipeline Test Complete ==="
echo "Results saved to: $OUTPUT_DIR"
echo ""
echo "Output structure:"
echo "  $OUTPUT_DIR/colmap/          - COLMAP reconstruction"
echo "  $OUTPUT_DIR/brush_model/     - Trained Gaussian Splat model"
echo "  $OUTPUT_DIR/renders/         - Rendered novel views"
echo ""
echo "You can view the COLMAP reconstruction with:"
echo "  colmap gui --database_path $OUTPUT_DIR/colmap/database.db --image_path $INPUT_DIR"