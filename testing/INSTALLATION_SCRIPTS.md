# Installation Scripts Documentation

This document details the installation scripts needed for setting up COLMAP and Brush for testing with Hylaean Splat.

## COLMAP Installation Script (`install_colmap.sh`)

### Purpose
Automated installation of COLMAP with multiple fallback methods to ensure compatibility across different systems.

### Installation Methods

#### Method 1: System Package Manager
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install colmap

# macOS
brew install colmap

# Arch Linux
sudo pacman -S colmap
```

#### Method 2: Conda Installation
```bash
conda install -c conda-forge colmap
```

#### Method 3: Build from Source
```bash
# Prerequisites
sudo apt install cmake build-essential libboost-all-dev libeigen3-dev
sudo apt install libfreeimage-dev libmetis-dev libgoogle-glog-dev
sudo apt install libgflags-dev libglew-dev qtbase5-dev libqt5opengl5-dev
sudo apt install libcgal-dev libceres-dev

# Clone and build
git clone https://github.com/colmap/colmap.git
cd colmap
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make -j$(nproc)
sudo make install
```

#### Method 4: Docker Installation
```bash
# Pull official COLMAP Docker image
docker pull colmap/colmap:latest

# Create wrapper script for Docker execution
cat > ~/.local/bin/colmap << 'EOF'
#!/bin/bash
docker run --rm -v "$PWD:/workspace" -w /workspace colmap/colmap:latest "$@"
EOF
chmod +x ~/.local/bin/colmap
```

### Installation Script Logic
1. **Detect Operating System**: Determine Linux distribution or macOS
2. **Check Existing Installation**: Verify if COLMAP is already installed
3. **Try Package Manager**: Attempt system package manager first
4. **Fallback to Conda**: If package manager fails, try conda
5. **Source Build**: As last resort, build from source
6. **Verification**: Test installation with `colmap --help`
7. **Local Installation**: Install to `./tools/colmap` for project isolation

### Environment Variables
- `COLMAP_INSTALL_DIR`: Custom installation directory
- `COLMAP_BUILD_TYPE`: Release/Debug build type
- `COLMAP_CUDA_SUPPORT`: Enable CUDA support (true/false)

## Brush Installation Script (`install_brush.sh`)

### Purpose
Automated installation of Brush with Rust toolchain setup and GPU support configuration.

### Installation Steps

#### Step 1: Rust Toolchain Setup
```bash
# Install rustup if not present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install nightly toolchain for latest features
rustup toolchain install nightly
rustup default nightly
```

#### Step 2: GPU Dependencies
```bash
# CUDA Support (NVIDIA)
if command -v nvidia-smi &> /dev/null; then
    echo "NVIDIA GPU detected, installing CUDA dependencies..."
    # Install CUDA toolkit if not present
    # Configure environment variables
fi

# ROCm Support (AMD)
if command -v rocm-smi &> /dev/null; then
    echo "AMD GPU detected, installing ROCm dependencies..."
    # Install ROCm toolkit if not present
fi
```

#### Step 3: Clone and Build
```bash
# Clone Brush repository
git clone https://github.com/ArthurBrussee/brush.git ./tools/brush
cd ./tools/brush

# Configure build
export RUSTFLAGS="-C target-cpu=native"
export CARGO_INCREMENTAL=0

# Build with optimizations
cargo build --release --features gpu
```

#### Step 4: Installation Verification
```bash
# Test basic functionality
./target/release/brush --help

# Test GPU availability
./target/release/brush --list-devices
```

### Build Configuration
- **Release Mode**: Optimized for performance
- **GPU Features**: Enable CUDA or ROCm support
- **Native CPU**: Optimize for host CPU architecture
- **Link Time Optimization**: Enable for maximum performance

### Environment Variables
- `BRUSH_INSTALL_DIR`: Custom installation directory
- `BRUSH_GPU_BACKEND`: cuda/rocm/cpu
- `BRUSH_BUILD_FEATURES`: Additional cargo features

## Setup Environment Script (`setup_test_env.sh`)

### Purpose
Complete environment setup including tool installation, test data download, and configuration.

### Setup Steps

#### 1. Initialize Hylaean Splat
```bash
# Build Hylaean Splat
cargo build --release

# Initialize configuration
./target/release/hylaeansplat init --force

# Create testing directories
mkdir -p testing/{datasets,outputs,validation}
```

#### 2. Install Tools
```bash
# Install COLMAP
./testing/scripts/install_colmap.sh

# Install Brush
./testing/scripts/install_brush.sh

# Verify installations
./target/release/hylaeansplat tool discover
./target/release/hylaeansplat list --detailed
```

#### 3. Download Test Datasets
```bash
# Small dataset (synthetic cube)
wget -O testing/datasets/small.zip "https://example.com/datasets/small.zip"
unzip testing/datasets/small.zip -d testing/datasets/small/

# Medium dataset (room scan)
wget -O testing/datasets/medium.zip "https://example.com/datasets/medium.zip"
unzip testing/datasets/medium.zip -d testing/datasets/medium/

# Challenging dataset (outdoor scene)
wget -O testing/datasets/challenging.zip "https://example.com/datasets/challenging.zip"
unzip testing/datasets/challenging.zip -d testing/datasets/challenging/
```

#### 4. Configuration Setup
```bash
# Create test configuration
cat > testing/config.toml << 'EOF'
[test_config]
timeout_minutes = 60
quality_threshold = 0.8
performance_target_fps = 30

[colmap_config]
feature_extractor_quality = "high"
matcher_type = "exhaustive"
dense_reconstruction = true

[brush_config]
training_iterations = 5000
learning_rate = 0.01
batch_size = 1
EOF
```

## Full Pipeline Script (`run_full_pipeline.sh`)

### Purpose
End-to-end testing of the complete COLMAP → Brush pipeline with validation.

### Pipeline Steps

#### 1. Data Preparation
```bash
DATASET_SIZE=${1:-small}
INPUT_DIR="testing/datasets/$DATASET_SIZE"
OUTPUT_DIR="testing/outputs/$DATASET_SIZE"

# Validate input images
python3 testing/validation/validate_images.py "$INPUT_DIR"
```

#### 2. COLMAP Pipeline
```bash
# Initialize output directory
mkdir -p "$OUTPUT_DIR/colmap"

# Run COLMAP full pipeline
./target/release/hylaeansplat tool run colmap full_pipeline \
    "$INPUT_DIR" \
    "$OUTPUT_DIR/colmap"

# Validate COLMAP output
python3 testing/validation/validate_colmap.py "$OUTPUT_DIR/colmap"
```

#### 3. Format Conversion
```bash
# Convert COLMAP sparse reconstruction to Brush format
./target/release/hylaeansplat convert \
    --input "$OUTPUT_DIR/colmap/sparse/0" \
    --output "$OUTPUT_DIR/brush_input" \
    --output-format brush
```

#### 4. Brush Training
```bash
# Train Gaussian Splat model
./target/release/hylaeansplat tool run brush_app train \
    "$OUTPUT_DIR/brush_input" \
    "$OUTPUT_DIR/brush_model"

# Validate training results
python3 testing/validation/validate_brush.py "$OUTPUT_DIR/brush_model"
```

#### 5. Rendering and Validation
```bash
# Render novel views
./target/release/hylaeansplat tool run brush_app render \
    "$OUTPUT_DIR/brush_model" \
    "$OUTPUT_DIR/renders"

# Calculate quality metrics
python3 testing/validation/calculate_metrics.py \
    "$OUTPUT_DIR/renders" \
    "$INPUT_DIR/ground_truth"
```

### Performance Monitoring
- **GPU Usage**: Monitor with `nvidia-smi`
- **Memory Usage**: Track with `htop`
- **Disk I/O**: Monitor with `iotop`
- **Pipeline Timing**: Log each step duration

### Error Handling
- **Timeout Protection**: Kill processes after timeout
- **Cleanup**: Remove partial outputs on failure
- **Logging**: Comprehensive error logging
- **Recovery**: Automatic retry with different parameters

## Test Data Download Script (`download_test_data.sh`)

### Purpose
Download and prepare test datasets for comprehensive pipeline testing.

### Dataset Sources

#### Small Dataset (Synthetic Cube)
- **Source**: Procedurally generated
- **Images**: 12 views of a textured cube
- **Resolution**: 512x512
- **Format**: PNG
- **Size**: ~5MB

#### Medium Dataset (Room Scan)
- **Source**: Real indoor environment
- **Images**: 80 photos of a furnished room
- **Resolution**: 1920x1080
- **Format**: JPG
- **Size**: ~200MB

#### Challenging Dataset (Outdoor Scene)
- **Source**: Complex outdoor environment
- **Images**: 150 photos with challenging conditions
- **Resolution**: 4000x3000
- **Format**: RAW + JPG
- **Size**: ~2GB

### Download and Preparation
```bash
# Create dataset directories
mkdir -p testing/datasets/{small,medium,challenging}

# Download with progress bars
wget --progress=bar:force "https://datasets.hylaeansplat.com/small.tar.gz"
wget --progress=bar:force "https://datasets.hylaeansplat.com/medium.tar.gz"
wget --progress=bar:force "https://datasets.hylaeansplat.com/challenging.tar.gz"

# Extract and organize
tar -xzf small.tar.gz -C testing/datasets/small/
tar -xzf medium.tar.gz -C testing/datasets/medium/
tar -xzf challenging.tar.gz -C testing/datasets/challenging/

# Cleanup archives
rm *.tar.gz
```

### Data Validation
- **Image Format Check**: Verify all images are valid
- **Resolution Consistency**: Check image dimensions
- **Metadata Extraction**: Extract EXIF data
- **Duplicate Detection**: Remove duplicate images

## Next Steps

To implement these scripts, we need to:

1. **Switch to Code Mode**: Create the actual shell scripts
2. **Create Python Validation Scripts**: Implement quality checks
3. **Add Dataset Integration**: Connect to real test datasets
4. **Implement Error Handling**: Robust error management
5. **Add Performance Monitoring**: Resource usage tracking

The testing framework will provide:
- **Automated Setup**: One-command environment preparation
- **Comprehensive Testing**: Multiple dataset scenarios
- **Quality Validation**: Automated output verification
- **Performance Benchmarking**: Speed and quality metrics
- **Error Recovery**: Graceful failure handling