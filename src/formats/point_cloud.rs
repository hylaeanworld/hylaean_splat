//! Point cloud format handling and conversion

use crate::errors::{Result, HylaeanError};
use crate::formats::{DataFormat, PointCloudFormat, FormatConverter};
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use log::{info, debug};

pub struct PointCloudConverter;

impl FormatConverter for PointCloudConverter {
    fn can_convert(&self, from: &DataFormat, to: &DataFormat) -> bool {
        matches!(
            (from, to),
            (DataFormat::PointCloud(_), DataFormat::PointCloud(_))
        )
    }
    
    fn convert(&self, input_path: &Path, output_path: &Path, from: &DataFormat, to: &DataFormat) -> Result<()> {
        match (from, to) {
            (DataFormat::PointCloud(from_fmt), DataFormat::PointCloud(to_fmt)) => {
                self.convert_point_cloud(input_path, output_path, from_fmt, to_fmt)
            }
            _ => Err(HylaeanError::ConversionFailed {
                source_format: format!("{:?}", from),
                target_format: format!("{:?}", to),
            }),
        }
    }
}

impl PointCloudConverter {
    pub fn new() -> Self {
        Self
    }
    
    fn convert_point_cloud(
        &self,
        input_path: &Path,
        output_path: &Path,
        from_format: &PointCloudFormat,
        to_format: &PointCloudFormat,
    ) -> Result<()> {
        info!("Converting point cloud: {:?} -> {:?}", from_format, to_format);
        
        match (from_format, to_format) {
            (PointCloudFormat::PLY, PointCloudFormat::XYZ) => {
                self.ply_to_xyz(input_path, output_path)
            }
            (PointCloudFormat::XYZ, PointCloudFormat::PLY) => {
                self.xyz_to_ply(input_path, output_path)
            }
            (PointCloudFormat::PLY, PointCloudFormat::PCD) => {
                self.ply_to_pcd(input_path, output_path)
            }
            (PointCloudFormat::PCD, PointCloudFormat::PLY) => {
                self.pcd_to_ply(input_path, output_path)
            }
            (PointCloudFormat::XYZ, PointCloudFormat::PCD) => {
                self.xyz_to_pcd(input_path, output_path)
            }
            (PointCloudFormat::PCD, PointCloudFormat::XYZ) => {
                self.pcd_to_xyz(input_path, output_path)
            }
            _ => Err(HylaeanError::ConversionFailed {
                source_format: format!("{:?}", from_format),
                target_format: format!("{:?}", to_format),
            }),
        }
    }
    
    fn ply_to_xyz(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        debug!("Converting PLY to XYZ: {} -> {}", input_path.display(), output_path.display());
        
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
        
        info!("Converted {} vertices from PLY to XYZ", processed_vertices);
        Ok(())
    }
    
    fn xyz_to_ply(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        debug!("Converting XYZ to PLY: {} -> {}", input_path.display(), output_path.display());
        
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
        
        info!("Converted {} vertices from XYZ to PLY", vertex_count);
        Ok(())
    }
    
    fn ply_to_pcd(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        debug!("Converting PLY to PCD: {} -> {}", input_path.display(), output_path.display());
        
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
        
        info!("Converted {} vertices from PLY to PCD", processed_vertices);
        Ok(())
    }
    
    fn pcd_to_ply(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        debug!("Converting PCD to PLY: {} -> {}", input_path.display(), output_path.display());
        
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
        
        info!("Converted {} vertices from PCD to PLY", vertex_count);
        Ok(())
    }
    
    fn xyz_to_pcd(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        debug!("Converting XYZ to PCD: {} -> {}", input_path.display(), output_path.display());
        
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
        
        info!("Converted {} vertices from XYZ to PCD", vertex_count);
        Ok(())
    }
    
    fn pcd_to_xyz(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        debug!("Converting PCD to XYZ: {} -> {}", input_path.display(), output_path.display());
        
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
}