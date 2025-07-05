//! Camera parameter format handling and conversion

use crate::errors::{Result, HylaeanError};
use crate::formats::{DataFormat, CameraFormat, FormatConverter};
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use serde::{Deserialize, Serialize};
use log::{info, debug, warn};

pub struct CameraParamsConverter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColmapCamera {
    pub camera_id: u32,
    pub model: String,
    pub width: u32,
    pub height: u32,
    pub params: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColmapImage {
    pub image_id: u32,
    pub qw: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
    pub tx: f64,
    pub ty: f64,
    pub tz: f64,
    pub camera_id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeRFCamera {
    pub camera_angle_x: f64,
    pub frames: Vec<NeRFFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeRFFrame {
    pub file_path: String,
    pub rotation: f64,
    pub transform_matrix: [[f64; 4]; 4],
}

impl FormatConverter for CameraParamsConverter {
    fn can_convert(&self, from: &DataFormat, to: &DataFormat) -> bool {
        matches!(
            (from, to),
            (DataFormat::CameraParameters(_), DataFormat::CameraParameters(_))
        )
    }
    
    fn convert(&self, input_path: &Path, output_path: &Path, from: &DataFormat, to: &DataFormat) -> Result<()> {
        match (from, to) {
            (DataFormat::CameraParameters(from_fmt), DataFormat::CameraParameters(to_fmt)) => {
                self.convert_camera_params(input_path, output_path, from_fmt, to_fmt)
            }
            _ => Err(HylaeanError::ConversionFailed {
                source_format: format!("{:?}", from),
                target_format: format!("{:?}", to),
            }),
        }
    }
}

impl CameraParamsConverter {
    pub fn new() -> Self {
        Self
    }
    
    fn convert_camera_params(
        &self,
        input_path: &Path,
        output_path: &Path,
        from_format: &CameraFormat,
        to_format: &CameraFormat,
    ) -> Result<()> {
        info!("Converting camera parameters: {:?} -> {:?}", from_format, to_format);
        
        match (from_format, to_format) {
            (CameraFormat::COLMAP, CameraFormat::NeRF) => {
                self.colmap_to_nerf(input_path, output_path)
            }
            (CameraFormat::NeRF, CameraFormat::COLMAP) => {
                self.nerf_to_colmap(input_path, output_path)
            }
            (CameraFormat::COLMAP, CameraFormat::OpenCV) => {
                self.colmap_to_opencv(input_path, output_path)
            }
            (CameraFormat::OpenCV, CameraFormat::COLMAP) => {
                self.opencv_to_colmap(input_path, output_path)
            }
            _ => {
                warn!("Conversion not implemented: {:?} -> {:?}", from_format, to_format);
                Err(HylaeanError::ConversionFailed {
                    source_format: format!("{:?}", from_format),
                    target_format: format!("{:?}", to_format),
                })
            }
        }
    }
    
    fn colmap_to_nerf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        debug!("Converting COLMAP to NeRF format: {} -> {}", input_path.display(), output_path.display());
        
        // Read COLMAP cameras and images
        let cameras = self.read_colmap_cameras(input_path)?;
        let images = self.read_colmap_images(input_path)?;
        
        // Convert to NeRF format
        let nerf_data = self.convert_colmap_to_nerf_data(&cameras, &images)?;
        
        // Write NeRF JSON
        let mut output_file = File::create(output_path)?;
        let json_string = serde_json::to_string_pretty(&nerf_data)?;
        output_file.write_all(json_string.as_bytes())?;
        
        info!("Converted COLMAP to NeRF format with {} frames", nerf_data.frames.len());
        Ok(())
    }
    
    fn nerf_to_colmap(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        debug!("Converting NeRF to COLMAP format: {} -> {}", input_path.display(), output_path.display());
        
        // Read NeRF JSON
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let nerf_data: NeRFCamera = serde_json::from_reader(reader)?;
        
        // Convert to COLMAP format
        let (cameras, images) = self.convert_nerf_to_colmap_data(&nerf_data)?;
        
        // Write COLMAP files
        self.write_colmap_cameras(output_path, &cameras)?;
        self.write_colmap_images(output_path, &images)?;
        
        info!("Converted NeRF to COLMAP format with {} cameras and {} images", cameras.len(), images.len());
        Ok(())
    }
    
    fn colmap_to_opencv(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        debug!("Converting COLMAP to OpenCV format: {} -> {}", input_path.display(), output_path.display());
        
        // This is a simplified conversion
        // In practice, you'd need to handle distortion parameters and coordinate system differences
        warn!("COLMAP to OpenCV conversion is simplified and may not preserve all parameters");
        
        let cameras = self.read_colmap_cameras(input_path)?;
        let images = self.read_colmap_images(input_path)?;
        
        // Write OpenCV format (simplified)
        let mut output_file = File::create(output_path)?;
        
        writeln!(output_file, "# OpenCV camera parameters converted from COLMAP")?;
        writeln!(output_file, "# Format: camera_id width height fx fy cx cy")?;
        
        for camera in cameras {
            if camera.params.len() >= 4 {
                writeln!(
                    output_file,
                    "{} {} {} {} {} {} {}",
                    camera.camera_id,
                    camera.width,
                    camera.height,
                    camera.params[0], // fx
                    camera.params[1], // fy
                    camera.params[2], // cx
                    camera.params[3], // cy
                )?;
            }
        }
        
        info!("Converted COLMAP to OpenCV format");
        Ok(())
    }
    
    fn opencv_to_colmap(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        debug!("Converting OpenCV to COLMAP format: {} -> {}", input_path.display(), output_path.display());
        
        warn!("OpenCV to COLMAP conversion not fully implemented");
        
        // This would need to read OpenCV format and convert to COLMAP
        // For now, just create a placeholder
        let mut output_file = File::create(output_path)?;
        writeln!(output_file, "# Camera list with one line of data per camera:")?;
        writeln!(output_file, "# CAMERA_ID, MODEL, WIDTH, HEIGHT, PARAMS[]")?;
        
        info!("OpenCV to COLMAP conversion placeholder created");
        Ok(())
    }
    
    fn read_colmap_cameras(&self, base_path: &Path) -> Result<Vec<ColmapCamera>> {
        let cameras_file = base_path.join("cameras.txt");
        if !cameras_file.exists() {
            return Err(HylaeanError::InvalidPath {
                path: cameras_file.display().to_string(),
            });
        }
        
        let file = File::open(&cameras_file)?;
        let reader = BufReader::new(file);
        let mut cameras = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let camera_id = parts[0].parse::<u32>().unwrap_or(0);
                let model = parts[1].to_string();
                let width = parts[2].parse::<u32>().unwrap_or(0);
                let height = parts[3].parse::<u32>().unwrap_or(0);
                let params: Vec<f64> = parts[4..].iter()
                    .filter_map(|s| s.parse::<f64>().ok())
                    .collect();
                
                cameras.push(ColmapCamera {
                    camera_id,
                    model,
                    width,
                    height,
                    params,
                });
            }
        }
        
        Ok(cameras)
    }
    
    fn read_colmap_images(&self, base_path: &Path) -> Result<Vec<ColmapImage>> {
        let images_file = base_path.join("images.txt");
        if !images_file.exists() {
            return Err(HylaeanError::InvalidPath {
                path: images_file.display().to_string(),
            });
        }
        
        let file = File::open(&images_file)?;
        let reader = BufReader::new(file);
        let mut images = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                let image_id = parts[0].parse::<u32>().unwrap_or(0);
                let qw = parts[1].parse::<f64>().unwrap_or(0.0);
                let qx = parts[2].parse::<f64>().unwrap_or(0.0);
                let qy = parts[3].parse::<f64>().unwrap_or(0.0);
                let qz = parts[4].parse::<f64>().unwrap_or(0.0);
                let tx = parts[5].parse::<f64>().unwrap_or(0.0);
                let ty = parts[6].parse::<f64>().unwrap_or(0.0);
                let tz = parts[7].parse::<f64>().unwrap_or(0.0);
                let camera_id = parts[8].parse::<u32>().unwrap_or(0);
                let name = parts[9].to_string();
                
                images.push(ColmapImage {
                    image_id,
                    qw,
                    qx,
                    qy,
                    qz,
                    tx,
                    ty,
                    tz,
                    camera_id,
                    name,
                });
            }
        }
        
        Ok(images)
    }
    
    fn convert_colmap_to_nerf_data(&self, cameras: &[ColmapCamera], images: &[ColmapImage]) -> Result<NeRFCamera> {
        let mut frames = Vec::new();
        
        // Get camera parameters (assuming single camera for simplicity)
        let camera = cameras.first().ok_or_else(|| HylaeanError::ConversionFailed {
            source_format: "COLMAP".to_string(),
            target_format: "NeRF".to_string(),
        })?;
        
        // Calculate field of view
        let fx = camera.params.get(0).unwrap_or(&500.0);
        let camera_angle_x = 2.0 * (camera.width as f64 / (2.0 * fx)).atan();
        
        for image in images {
            // Convert quaternion to rotation matrix
            let transform_matrix = self.quaternion_to_matrix(
                image.qw, image.qx, image.qy, image.qz,
                image.tx, image.ty, image.tz,
            );
            
            let frame = NeRFFrame {
                file_path: image.name.clone(),
                rotation: 0.0, // Simplified
                transform_matrix,
            };
            
            frames.push(frame);
        }
        
        Ok(NeRFCamera {
            camera_angle_x,
            frames,
        })
    }
    
    fn convert_nerf_to_colmap_data(&self, nerf_data: &NeRFCamera) -> Result<(Vec<ColmapCamera>, Vec<ColmapImage>)> {
        let mut cameras = Vec::new();
        let mut images = Vec::new();
        
        // Create a single camera (simplified)
        let fx = 500.0; // Default focal length
        let fy = 500.0;
        let cx = 320.0; // Default principal point
        let cy = 240.0;
        
        let camera = ColmapCamera {
            camera_id: 1,
            model: "PINHOLE".to_string(),
            width: 640,
            height: 480,
            params: vec![fx, fy, cx, cy],
        };
        cameras.push(camera);
        
        // Convert frames to images
        for (i, frame) in nerf_data.frames.iter().enumerate() {
            let (qw, qx, qy, qz, tx, ty, tz) = self.matrix_to_quaternion(&frame.transform_matrix);
            
            let image = ColmapImage {
                image_id: (i + 1) as u32,
                qw,
                qx,
                qy,
                qz,
                tx,
                ty,
                tz,
                camera_id: 1,
                name: frame.file_path.clone(),
            };
            images.push(image);
        }
        
        Ok((cameras, images))
    }
    
    fn quaternion_to_matrix(&self, qw: f64, qx: f64, qy: f64, qz: f64, tx: f64, ty: f64, tz: f64) -> [[f64; 4]; 4] {
        let mut matrix = [[0.0; 4]; 4];
        
        // Rotation part
        matrix[0][0] = 1.0 - 2.0 * (qy * qy + qz * qz);
        matrix[0][1] = 2.0 * (qx * qy - qz * qw);
        matrix[0][2] = 2.0 * (qx * qz + qy * qw);
        matrix[1][0] = 2.0 * (qx * qy + qz * qw);
        matrix[1][1] = 1.0 - 2.0 * (qx * qx + qz * qz);
        matrix[1][2] = 2.0 * (qy * qz - qx * qw);
        matrix[2][0] = 2.0 * (qx * qz - qy * qw);
        matrix[2][1] = 2.0 * (qy * qz + qx * qw);
        matrix[2][2] = 1.0 - 2.0 * (qx * qx + qy * qy);
        
        // Translation part
        matrix[0][3] = tx;
        matrix[1][3] = ty;
        matrix[2][3] = tz;
        matrix[3][3] = 1.0;
        
        matrix
    }
    
    fn matrix_to_quaternion(&self, matrix: &[[f64; 4]; 4]) -> (f64, f64, f64, f64, f64, f64, f64) {
        // Extract translation
        let tx = matrix[0][3];
        let ty = matrix[1][3];
        let tz = matrix[2][3];
        
        // Extract rotation (simplified quaternion extraction)
        let trace = matrix[0][0] + matrix[1][1] + matrix[2][2];
        let (qw, qx, qy, qz) = if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            (
                0.25 * s,
                (matrix[2][1] - matrix[1][2]) / s,
                (matrix[0][2] - matrix[2][0]) / s,
                (matrix[1][0] - matrix[0][1]) / s,
            )
        } else {
            (1.0, 0.0, 0.0, 0.0) // Default quaternion
        };
        
        (qw, qx, qy, qz, tx, ty, tz)
    }
    
    fn write_colmap_cameras(&self, base_path: &Path, cameras: &[ColmapCamera]) -> Result<()> {
        let cameras_file = base_path.join("cameras.txt");
        let mut file = File::create(&cameras_file)?;
        
        writeln!(file, "# Camera list with one line of data per camera:")?;
        writeln!(file, "# CAMERA_ID, MODEL, WIDTH, HEIGHT, PARAMS[]")?;
        
        for camera in cameras {
            write!(file, "{} {} {} {}", camera.camera_id, camera.model, camera.width, camera.height)?;
            for param in &camera.params {
                write!(file, " {}", param)?;
            }
            writeln!(file)?;
        }
        
        Ok(())
    }
    
    fn write_colmap_images(&self, base_path: &Path, images: &[ColmapImage]) -> Result<()> {
        let images_file = base_path.join("images.txt");
        let mut file = File::create(&images_file)?;
        
        writeln!(file, "# Image list with two lines of data per image:")?;
        writeln!(file, "# IMAGE_ID, QW, QX, QY, QZ, TX, TY, TZ, CAMERA_ID, NAME")?;
        
        for image in images {
            writeln!(
                file,
                "{} {} {} {} {} {} {} {} {} {}",
                image.image_id,
                image.qw,
                image.qx,
                image.qy,
                image.qz,
                image.tx,
                image.ty,
                image.tz,
                image.camera_id,
                image.name
            )?;
            writeln!(file)?; // Empty line for points (not implemented)
        }
        
        Ok(())
    }
}