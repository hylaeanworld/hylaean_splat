# Hylaean Splat

**Operating system for 3D Gaussian splatting tools**

Hylaean Splat is a unified interface and management system for various 3D Gaussian splatting (3DGS) tools and frameworks. It provides format conversion, tool discovery, intelligent recommendations, and workflow orchestration for the 3DGS ecosystem.

## Features

### 🔧 Tool Management
- **Auto-discovery**: Automatically detect installed 3DGS tools on your system
- **Unified Interface**: Common CLI for all supported tools
- **Installation Management**: Automated installation and updates
- **Version Tracking**: Monitor tool versions and dependencies

### 🔄 Format Conversion
- **Point Cloud Formats**: Convert between PLY, PCD, XYZ, and LAZ
- **Camera Parameters**: Convert between COLMAP, NeRF, OpenCV, and Blender formats
- **Intelligent Detection**: Automatic format detection from file extensions and content

### 🤖 Agentic Intelligence
- **Repository Monitoring**: Track 3DGS repositories on GitHub
- **Paper Tracking**: Monitor arXiv for new 3DGS research
- **Smart Recommendations**: AI-powered tool suggestions based on your workflow
- **Installation Scripts**: Auto-generate installation scripts for new tools

### 🛠️ Supported Tools

#### Core 3DGS Tools
- **3D Gaussian Splatting** (Original) - Static scene reconstruction
- **SeaSplat** - Underwater scene reconstruction
- **SkySplat Blender** - Blender integration for rendering
- **Dynamic 3DGS** - Dynamic scene reconstruction
- **4D Gaussians** - Temporal 3DGS with time dimension
- **Brush** - Rust-based high-performance renderer
- **COLMAP** - Structure-from-motion preprocessing

## Installation

### Prerequisites
- Rust (latest stable version)
- Git

### Build from Source
```bash
git clone https://github.com/kyjohnso/hylaean_splat.git
cd hylaean_splat
cargo build --release
```

### Install Binary
```bash
cargo install --path .
```

## Quick Start

### 1. Initialize Hylaean Splat
```bash
hylaean init
```

### 2. Discover Installed Tools
```bash
hylaean tool discover
```

### 3. List Available Tools
```bash
hylaean list --detailed
```

### 4. Convert Point Cloud Formats
```bash
hylaean convert --input data.ply --output data.xyz --output-format xyz
```

### 5. Start the Agentic Monitor
```bash
hylaean agent start
```

## Usage

### Tool Management

#### Discover Tools
```bash
# Auto-discover tools in common locations
hylaean tool discover

# Search specific directory
hylaean tool discover --path /path/to/tools
```

#### Install Tools
```bash
# Install from known repository
hylaean tool install gaussian_splatting

# Install to specific location
hylaean tool install brush_app --path ./my_tools
```

#### Run Tools
```bash
# Train with 3D Gaussian Splatting
hylaean tool run gaussian_splatting train /path/to/data /path/to/output

# Render with Brush
hylaean tool run brush_app render /path/to/model /path/to/output
```

### Format Conversion

#### Point Cloud Conversion
```bash
# PLY to XYZ
hylaean convert -i model.ply -o points.xyz --output-format xyz

# PCD to PLY
hylaean convert -i cloud.pcd -o cloud.ply --output-format ply
```

#### Camera Parameter Conversion
```bash
# COLMAP to NeRF
hylaean convert -i colmap_sparse -o transforms.json --output-format nerf

# NeRF to COLMAP
hylaean convert -i transforms.json -o colmap_out --output-format colmap
```

### Agentic Features

#### Start Monitoring
```bash
# Start in foreground
hylaean agent start

# Start as daemon
hylaean agent start --daemon
```

#### Get Recommendations
```bash
# General recommendations
hylaean agent recommend

# Specific use case
hylaean agent recommend --use-case "dynamic scenes"
```

#### Generate Installation Scripts
```bash
# Generate script for a tool
hylaean agent generate brush_app --output install_brush.sh
```

### Workflows

#### Complete COLMAP Pipeline
```bash
# Run full COLMAP pipeline
hylaean tool run colmap full_pipeline /path/to/images /path/to/output
```

#### 3DGS Training Pipeline
```bash
# 1. Preprocess with COLMAP
hylaean tool run colmap full_pipeline ./images ./colmap_output

# 2. Train with 3D Gaussian Splatting
hylaean tool run gaussian_splatting train ./colmap_output ./gs_output

# 3. Render results
hylaean tool run gaussian_splatting render ./gs_output ./renders
```

## Configuration

Hylaean Splat stores configuration in `~/.hylaean_splat/`:

```
~/.hylaean_splat/
├── config.toml          # Main configuration
├── database/            # Tool registry and metadata
├── tools/              # Installed tools
└── cache/              # Conversion cache
```

### Configuration Options

Edit `~/.hylaean_splat/config.toml`:

```toml
[agent_config]
enabled = true
update_interval_hours = 24
github_token = "your_github_token"  # Optional, for higher API limits
arxiv_search_terms = [
    "3d gaussian splatting",
    "gaussian splatting",
    "neural radiance field"
]
max_repos_to_track = 1000
auto_install_recommendations = false

[format_config]
default_point_cloud_format = "ply"
default_camera_format = "colmap"
conversion_cache_size_mb = 1024
preserve_metadata = true
```

## Development

### Project Structure
```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library root
├── cli/                 # Command-line interface
├── core/                # Core functionality
│   ├── tool_manager.rs  # Tool discovery and management
│   ├── data_manager.rs  # Format conversion
│   └── agent.rs         # Agentic intelligence
├── integrations/        # Tool integrations
│   ├── gaussian_splatting.rs
│   ├── brush_app.rs
│   ├── colmap.rs
│   └── ...
├── formats/             # Format handling
│   ├── point_cloud.rs
│   └── camera_params.rs
├── agentic/             # Agentic components
├── config.rs            # Configuration management
└── errors.rs            # Error handling
```

### Adding New Tools

To add support for a new 3DGS tool:

1. Create a new integration module in `src/integrations/`
2. Implement the `Integration` trait
3. Add the tool to the discovery system in `integrations/mod.rs`
4. Update the tool templates in `tool_manager.rs`

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Roadmap

### Phase 1: Foundation ✅
- [x] Core Rust architecture
- [x] CLI interface
- [x] Tool discovery and registry
- [x] Format conversion (point clouds)
- [x] Basic agentic component

### Phase 2: Integration 🚧
- [x] 3D Gaussian Splatting integration
- [x] COLMAP integration
- [x] Brush App integration
- [ ] Complete format conversion system
- [ ] Advanced agentic features

### Phase 3: Intelligence 📋
- [ ] ML-based recommendations
- [ ] Workflow optimization
- [ ] Performance benchmarking
- [ ] Quality metrics

### Phase 4: Interfaces 📋
- [ ] Web interface (WASM)
- [ ] Desktop application (Tauri)
- [ ] Advanced visualizations
- [ ] Collaborative features

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original 3D Gaussian Splatting paper and implementation
- All the amazing 3DGS research community
- Contributors to the various integrated tools

## Support

For questions, issues, or feature requests:
- Open an issue on GitHub
- Check the documentation
- Join our community discussions

---

**Hylaean Splat** - Unifying the 3D Gaussian splatting ecosystem