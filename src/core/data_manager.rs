use crate::errors::{Result, HylaeanError};
use log::{info, warn, error, debug};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFormat {
    PointCloud(PointCloudFormat),
    CameraParameters(CameraFormat),
    Dataset(DatasetFormat),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PointCloudFormat {
    PLY,
    PCD,
    XYZ,
    LAZ,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraFormat {
    COLMAP,
    NeRF,
    OpenCV,
    Blender,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatasetFormat {
    NeRFSynthetic,
    LLFF,
    TanksAndTemples,
    Custom(String),
}

pub struct DataManager {
    conversion_cache: PathBuf,
}

impl DataManager {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("./cache"))
            .join("hylaean_splat")
            .join("conversions");
        
        std::fs::create_dir_all(&cache_dir)?;
        
        Ok(Self {
            conversion_cache: cache_dir,
        })
    }
    
    pub async fn convert_file(
        &mut self,
        input: String,
        output: String,
        input_format: Option<String>,
        output_format: String,
    ) -> Result<()> {
        let input_path = PathBuf::from(input);
        let output_path = PathBuf::from(output);
        
        let input_fmt = if let Some(fmt) = input_format {
            self.parse_format(&fmt)?
        } else {
            self.detect_format(&input_path)?
        };
        
        let output_fmt = self.parse_format(&output_format)?;
        
        info!("Converting {:?} to {:?}", input_fmt, output_fmt);
        
        match (&input_fmt, &output_fmt) {
            (DataFormat::PointCloud(input_pc), DataFormat::PointCloud(output_pc)) => {
                self.convert_point_cloud(&input_path, &output_path, input_pc, output_pc).await?;
            }
            (DataFormat::CameraParameters(input_cam), DataFormat::CameraParameters(output_cam)) => {
                self.convert_camera_params(&input_path, &output_path, input_cam, output_cam).await?;
            }
            _ => {
                return Err(HylaeanError::ConversionFailed {
                    source_format: format!("{:?}", input_fmt),
                    target_format: format!("{:?}", output_fmt),
                });
            }
        }
        
        info!("Conversion completed: {} -> {}", input_path.display(), output_path.display());
        Ok(())
    }
    
    fn parse_format(&self, format_str: &str) -> Result<DataFormat> {
        match format_str.to_lowercase().as_str() {
            "ply" => Ok(DataFormat::PointCloud(PointCloudFormat::PLY)),
            "pcd" => Ok(DataFormat::PointCloud(PointCloudFormat::PCD)),
            "xyz" => Ok(DataFormat::PointCloud(PointCloudFormat::XYZ)),
            "laz" => Ok(DataFormat::PointCloud(PointCloudFormat::LAZ)),
            "colmap" => Ok(DataFormat::CameraParameters(CameraFormat::COLMAP)),
            "nerf" => Ok(DataFormat::CameraParameters(CameraFormat::NeRF)),
            "opencv" => Ok(DataFormat::CameraParameters(CameraFormat::OpenCV)),
            "blender" => Ok(DataFormat::CameraParameters(CameraFormat::Blender)),
            _ => Err(HylaeanError::UnsupportedFormat {
                format: format_str.to_string(),
            }),
        }
    }
    
    fn detect_format(&self, path: &Path) -> Result<DataFormat> {
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "ply" => Ok(DataFormat::PointCloud(PointCloudFormat::PLY)),
            "pcd" => Ok(DataFormat::PointCloud(PointCloudFormat::PCD)),
            "xyz" => Ok(DataFormat::PointCloud(PointCloudFormat::XYZ)),
            "laz" => Ok(DataFormat::PointCloud(PointCloudFormat::LAZ)),
            "txt" => {
                // Could be camera parameters or point cloud
                if self.looks_like_camera_params(path)? {
                    Ok(DataFormat::CameraParameters(CameraFormat::COLMAP))
                } else {
                    Ok(DataFormat::PointCloud(PointCloudFormat::XYZ))
                }
            }
            "json" => Ok(DataFormat::CameraParameters(CameraFormat::NeRF)),
            _ => Err(HylaeanError::UnsupportedFormat {
                format: extension.to_string(),
            }),
        }
    }
    
    fn looks_like_camera_params(&self, path: &Path) -> Result<bool> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines().take(5) {
            let line = line?;
            if line.contains("# Camera list") || line.contains("# Image list") {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    async fn convert_point_cloud(
        &self,
        input_path: &Path,
        output_path: &Path,
        input_format: &PointCloudFormat,
        output_format: &PointCloudFormat,
    ) -> Result<()> {
        match (input_format, output_format) {
            (PointCloudFormat::PLY, PointCloudFormat::XYZ) => {
                self.ply_to_xyz(input_path, output_path).await?;
            }
            (PointCloudFormat::XYZ, PointCloudFormat::PLY) => {
                self.xyz_to_ply(input_path, output_path).await?;
            }
            (PointCloudFormat::PLY, PointCloudFormat::PCD) => {
                self.ply_to_pcd(input_path, output_path).await?;
            }
            (PointCloudFormat::PCD, PointCloudFormat::PLY) => {
                self.pcd_to_ply(input_path, output_path).await?;
            }
            (PointCloudFormat::XYZ, PointCloudFormat::PCD) => {
                self.xyz_to_pcd(input_path, output_path).await?;
            }
            (PointCloudFormat::PCD, PointCloudFormat::XYZ) => {
                self.pcd_to_xyz(input_path, output_path).await?;
            }
            _ => {
                return Err(HylaeanError::ConversionFailed {
                    source_format: format!("{:?}", input_format),
                    target_format: format!("{:?}", output_format),
                });
            }
        }
        
        Ok(())
    }
    
    async fn ply_to_xyz(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        info!("Converting PLY to XYZ: {} -> {}", input_path.display(), output_path.display());
        
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut output_file = File::create(output_path)?;
        
        let mut in_vertex_data = false;
        let mut vertex_count = 0;
        let mut processed_vertices = 0;
        
        for line in reader.lines() {
            let line = line?;
            
            if line.starts_with("element vertex") {
                vertex_count = line.split_whitespace()
                    .nth(2)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                continue;
            }
            
            if line == "end_header" {
                in_vertex_data = true;
                continue;
            }
            
            if in_vertex_data && processed_vertices < vertex_count {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    writeln!(output_file, "{} {} {}", parts[0], parts[1], parts[2])?;
                    processed_vertices += 1;
                }
            }
        }
        
        info!("Converted {} vertices", processed_vertices);
        Ok(())
    }
    
    async fn xyz_to_ply(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        info!("Converting XYZ to PLY: {} -> {}", input_path.display(), output_path.display());
        
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        
        // First pass: count vertices
        let vertex_count = reader.lines().count();
        
        // Second pass: write PLY
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut output_file = File::create(output_path)?;
        
        // Write PLY header
        writeln!(output_file, "ply")?;
        writeln!(output_file, "format ascii 1.0")?;
        writeln!(output_file, "element vertex {}", vertex_count)?;
        writeln!(output_file, "property float x")?;
        writeln!(output_file, "property float y")?;
        writeln!(output_file, "property float z")?;
        writeln!(output_file, "end_header")?;
        
        // Write vertex data
        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                writeln!(output_file, "{}", line)?;
            }
        }
        
        info!("Converted {} vertices", vertex_count);
        Ok(())
    }
    
    async fn ply_to_pcd(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        info!("Converting PLY to PCD: {} -> {}", input_path.display(), output_path.display());
        
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut output_file = File::create(output_path)?;
        
        let mut in_vertex_data = false;
        let mut vertex_count = 0;
        let mut processed_vertices = 0;
        let mut vertices = Vec::new();
        
        // Read PLY and collect vertices
        for line in reader.lines() {
            let line = line?;
            
            if line.starts_with("element vertex") {
                vertex_count = line.split_whitespace()
                    .nth(2)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                continue;
            }
            
            if line == "end_header" {
                in_vertex_data = true;
                continue;
            }
            
            if in_vertex_data && processed_vertices < vertex_count {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    vertices.push(line.clone());
                    processed_vertices += 1;
                }
            }
        }
        
        // Write PCD header
        writeln!(output_file, "# .PCD v0.7 - Point Cloud Data file format")?;
        writeln!(output_file, "VERSION 0.7")?;
        writeln!(output_file, "FIELDS x y z")?;
        writeln!(output_file, "SIZE 4 4 4")?;
        writeln!(output_file, "TYPE F F F")?;
        writeln!(output_file, "COUNT 1 1 1")?;
        writeln!(output_file, "WIDTH {}", vertices.len())?;
        writeln!(output_file, "HEIGHT 1")?;
        writeln!(output_file, "VIEWPOINT 0 0 0 1 0 0 0")?;
        writeln!(output_file, "POINTS {}", vertices.len())?;
        writeln!(output_file, "DATA ascii")?;
        
        // Write vertex data
        for vertex in vertices {
            writeln!(output_file, "{}", vertex)?;
        }
        
        info!("Converted {} vertices", processed_vertices);
        Ok(())
    }
    
    async fn pcd_to_ply(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        info!("Converting PCD to PLY: {} -> {}", input_path.display(), output_path.display());
        
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut output_file = File::create(output_path)?;
        
        let mut in_data = false;
        let mut vertex_count = 0;
        let mut vertices = Vec::new();
        
        // Read PCD and collect vertices
        for line in reader.lines() {
            let line = line?;
            
            if line.starts_with("POINTS") {
                vertex_count = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                continue;
            }
            
            if line == "DATA ascii" {
                in_data = true;
                continue;
            }
            
            if in_data && !line.trim().is_empty() {
                vertices.push(line.clone());
            }
        }
        
        // Write PLY header
        writeln!(output_file, "ply")?;
        writeln!(output_file, "format ascii 1.0")?;
        writeln!(output_file, "element vertex {}", vertices.len())?;
        writeln!(output_file, "property float x")?;
        writeln!(output_file, "property float y")?;
        writeln!(output_file, "property float z")?;
        writeln!(output_file, "end_header")?;
        
        // Write vertex data
        for vertex in vertices {
            writeln!(output_file, "{}", vertex)?;
        }
        
        info!("Converted {} vertices", vertex_count);
        Ok(())
    }
    
    async fn xyz_to_pcd(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        info!("Converting XYZ to PCD: {} -> {}", input_path.display(), output_path.display());
        
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        
        // First pass: count vertices
        let vertex_count = reader.lines().count();
        
        // Second pass: write PCD
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut output_file = File::create(output_path)?;
        
        // Write PCD header
        writeln!(output_file, "# .PCD v0.7 - Point Cloud Data file format")?;
        writeln!(output_file, "VERSION 0.7")?;
        writeln!(output_file, "FIELDS x y z")?;
        writeln!(output_file, "SIZE 4 4 4")?;
        writeln!(output_file, "TYPE F F F")?;
        writeln!(output_file, "COUNT 1 1 1")?;
        writeln!(output_file, "WIDTH {}", vertex_count)?;
        writeln!(output_file, "HEIGHT 1")?;
        writeln!(output_file, "VIEWPOINT 0 0 0 1 0 0 0")?;
        writeln!(output_file, "POINTS {}", vertex_count)?;
        writeln!(output_file, "DATA ascii")?;
        
        // Write vertex data
        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                writeln!(output_file, "{}", line)?;
            }
        }
        
        info!("Converted {} vertices", vertex_count);
        Ok(())
    }
    
    async fn pcd_to_xyz(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        info!("Converting PCD to XYZ: {} -> {}", input_path.display(), output_path.display());
        
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut output_file = File::create(output_path)?;
        
        let mut in_data = false;
        
        for line in reader.lines() {
            let line = line?;
            
            if line == "DATA ascii" {
                in_data = true;
                continue;
            }
            
            if in_data && !line.trim().is_empty() {
                writeln!(output_file, "{}", line)?;
            }
        }
        
        info!("PCD to XYZ conversion completed");
        Ok(())
    }
    
    async fn convert_camera_params(
        &self,
        input_path: &Path,
        output_path: &Path,
        input_format: &CameraFormat,
        output_format: &CameraFormat,
    ) -> Result<()> {
        info!("Converting camera parameters: {:?} -> {:?}", input_format, output_format);
        
        match (input_format, output_format) {
            (CameraFormat::COLMAP, CameraFormat::NeRF) => {
                self.colmap_to_nerf(input_path, output_path).await?;
            }
            (CameraFormat::NeRF, CameraFormat::COLMAP) => {
                self.nerf_to_colmap(input_path, output_path).await?;
            }
            _ => {
                return Err(HylaeanError::ConversionFailed {
                    source_format: format!("{:?}", input_format),
                    target_format: format!("{:?}", output_format),
                });
            }
        }
        
        Ok(())
    }
    
    async fn colmap_to_nerf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        info!("Converting COLMAP to NeRF format");
        // TODO: Implement COLMAP to NeRF conversion
        warn!("COLMAP to NeRF conversion not yet implemented");
        Ok(())
    }
    
    async fn nerf_to_colmap(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        info!("Converting NeRF to COLMAP format");
        // TODO: Implement NeRF to COLMAP conversion
        warn!("NeRF to COLMAP conversion not yet implemented");
        Ok(())
    }
}