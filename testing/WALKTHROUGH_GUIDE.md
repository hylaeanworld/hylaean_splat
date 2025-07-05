# Testing Framework Walkthrough Guide

This guide walks you through implementing and running the testing framework for your COLMAP and Brush integrations.

## Prerequisites Check

Before starting, let's verify your current setup:

### 1. Build Status
```bash
# Ensure Hylaean Splat builds successfully
cargo build --release
```

### 2. Current CLI Status
```bash
# Test basic CLI functionality
./target/release/hylaean --help
./target/release/hylaean init --force
./target/release/hylaean list
```

### 3. Integration Status
```bash
# Check if tools are discoverable
./target/release/hylaean tool discover
```

## Step-by-Step Implementation

### Phase 1: Fix Critical CLI Integration (Required First)

**Problem**: The CLI can't execute your COLMAP/Brush integrations because [`tool_manager.rs`](src/core/tool_manager.rs:339) `run_tool` method isn't connected.

**Files to Fix**:
- [`src/core/tool_manager.rs`](src/core/tool_manager.rs) - Line 339: Connect `run_tool` to integrations
- [`src/integrations/mod.rs`](src/integrations/mod.rs) - Export integration modules
- [`src/core/mod.rs`](src/core/mod.rs) - Update imports if needed

**Expected Fix**:
```rust
// In src/core/tool_manager.rs, replace the TODO at line 339
pub async fn run_tool(&mut self, name: String, args: Vec<String>) -> Result<()> {
    match name.as_str() {
        "colmap" => {
            let colmap = crate::integrations::colmap::Colmap::new();
            if args.len() >= 2 {
                colmap.run_command(&args[0], &args[1..].to_vec())
            } else {
                Err(HylaeanError::ToolExecutionFailed {
                    tool: name,
                    message: "Insufficient arguments".to_string(),
                })
            }
        }
        "brush_app" => {
            let brush = crate::integrations::brush_app::BrushApp::new();
            if args.len() >= 2 {
                brush.run_command(&args[0], &args[1..].to_vec())
            } else {
                Err(HylaeanError::ToolExecutionFailed {
                    tool: name,
                    message: "Insufficient arguments".to_string(),
                })
            }
        }
        _ => Err(HylaeanError::ToolNotFound { name })
    }
}
```

### Phase 2: Create Installation Scripts

**Files to Create**:
- `testing/scripts/install_colmap.sh` - COLMAP installation
- `testing/scripts/install_brush.sh` - Brush installation  
- `testing/scripts/setup_test_env.sh` - Environment setup
- `testing/scripts/run_full_pipeline.sh` - Pipeline execution

**Installation Script Structure**:
```bash
#!/bin/bash
# install_colmap.sh
set -e

echo "Installing COLMAP..."

# Try different installation methods
if command -v apt-get &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y colmap
elif command -v brew &> /dev/null; then
    brew install colmap
elif command -v conda &> /dev/null; then
    conda install -c conda-forge colmap
else
    echo "Building COLMAP from source..."
    # Source build process
fi

# Verify installation
if command -v colmap &> /dev/null; then
    echo "COLMAP installed successfully: $(colmap --version)"
else
    echo "COLMAP installation failed"
    exit 1
fi
```

### Phase 3: Create Test Data

**Quick Test Setup**:
```bash
# Create minimal test dataset
mkdir -p testing/datasets/small/images

# Method 1: Use your own images
# Copy 10-15 photos of a simple object (book, mug, etc.) taken from different angles
# Make sure images have good overlap and are reasonably sharp

# Method 2: Download sample dataset
# We'll create a script to download a standard test dataset
```

**Test Image Requirements**:
- **Count**: 10-15 images minimum
- **Subject**: Simple object with good texture
- **Overlap**: Each image should share 60%+ content with adjacent images
- **Format**: JPG or PNG
- **Quality**: Sharp, well-lit images

### Phase 4: Basic Pipeline Test

**Test Command Sequence**:
```bash
# 1. Initialize Hylaean
./target/release/hylaean init --force

# 2. Discover tools (should find COLMAP if installed)
./target/release/hylaean tool discover

# 3. Test COLMAP pipeline
./target/release/hylaean tool run colmap full_pipeline \
    testing/datasets/small/images \
    testing/outputs/small/colmap

# 4. Test Brush training (if COLMAP succeeded)
./target/release/hylaean tool run brush_app train \
    testing/outputs/small/colmap \
    testing/outputs/small/brush_model

# 5. Test rendering
./target/release/hylaean tool run brush_app render \
    testing/outputs/small/brush_model \
    testing/outputs/small/renders
```

### Phase 5: Validation Scripts

**Python Validation Scripts**:
- `testing/validation/validate_images.py` - Check input image quality
- `testing/validation/validate_colmap.py` - Verify COLMAP reconstruction
- `testing/validation/validate_brush.py` - Check Brush training results

**Basic Validation**:
```bash
# Check if outputs exist and are valid
ls -la testing/outputs/small/colmap/
ls -la testing/outputs/small/brush_model/
ls -la testing/outputs/small/renders/
```

## Manual Testing Walkthrough

### Step 1: Prepare Test Images
```bash
# Create test directory
mkdir -p testing/datasets/small/images

# Add your test images (10-15 photos of the same object from different angles)
# Images should be named consistently: IMG_0001.jpg, IMG_0002.jpg, etc.
```

### Step 2: Test COLMAP Integration
```bash
# Build and initialize
cargo build --release
./target/release/hylaean init --force

# Test COLMAP directly
./target/release/hylaean tool run colmap full_pipeline \
    testing/datasets/small/images \
    testing/outputs/small/colmap

# Expected output:
# - testing/outputs/small/colmap/database.db
# - testing/outputs/small/colmap/sparse/0/ (with cameras.bin, images.bin, points3D.bin)
```

### Step 3: Test Brush Integration
```bash
# Test Brush training
./target/release/hylaean tool run brush_app train \
    testing/outputs/small/colmap \
    testing/outputs/small/brush_model

# Expected output:
# - testing/outputs/small/brush_model/ (with trained model files)
```

### Step 4: Test Rendering
```bash
# Test rendering
./target/release/hylaean tool run brush_app render \
    testing/outputs/small/brush_model \
    testing/outputs/small/renders

# Expected output:
# - testing/outputs/small/renders/ (with rendered images)
```

## Troubleshooting Common Issues

### Issue 1: "Tool not found" error
**Cause**: COLMAP or Brush not installed or not in PATH
**Solution**: 
```bash
# Check if tools are available
which colmap
which brush

# Install missing tools
./testing/scripts/install_colmap.sh
./testing/scripts/install_brush.sh
```

### Issue 2: "Insufficient arguments" error
**Cause**: CLI integration not properly connected
**Solution**: Fix the `run_tool` method in [`tool_manager.rs`](src/core/tool_manager.rs)

### Issue 3: COLMAP fails with "No images found"
**Cause**: Images not in expected location or format
**Solution**: 
```bash
# Check image directory
ls -la testing/datasets/small/images/
file testing/datasets/small/images/*

# Ensure images are valid JPG/PNG files
```

### Issue 4: Brush training fails
**Cause**: Invalid COLMAP output or missing dependencies
**Solution**:
```bash
# Verify COLMAP output exists
ls -la testing/outputs/small/colmap/sparse/0/

# Check Brush installation
brush --help
```

## Expected Results

### Successful COLMAP Pipeline
```
testing/outputs/small/colmap/
├── database.db          # Feature database
└── sparse/
    └── 0/
        ├── cameras.bin   # Camera parameters
        ├── images.bin    # Image poses
        └── points3D.bin  # 3D points
```

### Successful Brush Training
```
testing/outputs/small/brush_model/
├── model.ply           # Trained Gaussian splat model
├── training_log.json   # Training metrics
└── config.json         # Model configuration
```

### Successful Rendering
```
testing/outputs/small/renders/
├── render_001.png      # Novel view renders
├── render_002.png
└── ...
```

## Next Steps After Manual Testing

1. **Automate with Scripts**: Create the installation and pipeline scripts
2. **Add Validation**: Implement Python validation scripts
3. **Expand Datasets**: Add medium and challenging test datasets
4. **Performance Monitoring**: Add timing and quality metrics
5. **CI Integration**: Set up automated testing

## Getting Help

If you encounter issues:

1. **Check Logs**: Look for error messages in the terminal output
2. **Verify Dependencies**: Ensure COLMAP and Rust are properly installed
3. **Test Individually**: Run COLMAP and Brush separately to isolate issues
4. **Check File Permissions**: Ensure output directories are writable

## Ready to Implement?

The critical first step is fixing the CLI integration in [`tool_manager.rs`](src/core/tool_manager.rs). Once that's working, you can test the complete pipeline with a simple folder of images.

Would you like me to switch to Code mode and implement the necessary fixes to get your testing framework working?