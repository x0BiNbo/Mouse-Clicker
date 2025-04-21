use crate::modules::error::{AppError, Result};
use image::{DynamicImage, GenericImageView, RgbaImage};
use std::path::PathBuf;
use std::fs;
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose};
use windows::Win32::Graphics::Gdi;
use windows::Win32::Foundation::HWND;
use std::mem::size_of;

/// Represents a target image that can be searched for on the screen
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TargetImage {
    /// Unique identifier for the target
    pub id: String,
    /// User-friendly name for the target
    pub name: String,
    /// Base64 encoded image data
    pub image_data: String,
    /// Confidence threshold for matching (0.0 to 1.0)
    pub threshold: f32,
    /// Optional click offset from the center of the matched image
    pub click_offset: Option<(i32, i32)>,
}

/// Manages a collection of target images
#[derive(Clone, Debug)]
pub struct ImageLibrary {
    /// Directory where target images are stored
    targets_dir: PathBuf,
    /// Collection of loaded target images
    targets: Vec<TargetImage>,
}

impl ImageLibrary {
    /// Create a new image library
    pub fn new(targets_dir: &str) -> Self {
        let targets_dir = PathBuf::from(targets_dir);
        println!("Creating image library with targets directory: {:?}", targets_dir);

        // Create the directory if it doesn't exist
        if !targets_dir.exists() {
            println!("Targets directory does not exist, creating it");
            fs::create_dir_all(&targets_dir).expect("Failed to create targets directory");
        } else {
            println!("Targets directory already exists");

            // List files in the directory
            match fs::read_dir(&targets_dir) {
                Ok(entries) => {
                    println!("Files in targets directory:");
                    for entry in entries {
                        if let Ok(entry) = entry {
                            println!("  {:?}", entry.path());
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read targets directory: {}", e);
                }
            }
        }

        Self {
            targets_dir,
            targets: Vec::new(),
        }
    }

    /// Load all target images from the targets directory
    pub fn load_targets(&mut self) -> Result<()> {
        println!("Loading targets from directory: {:?}", self.targets_dir);
        self.targets.clear();

        // Read all JSON files in the targets directory
        for entry in fs::read_dir(&self.targets_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                println!("Found target file: {:?}", path);
                let json_content = fs::read_to_string(&path)?;
                println!("JSON content length: {}", json_content.len());

                match serde_json::from_str::<TargetImage>(&json_content) {
                    Ok(target) => {
                        println!("Loaded target: id={}, name={}, image_data_length={}",
                            target.id, target.name, target.image_data.len());
                        self.targets.push(target);
                    },
                    Err(e) => {
                        eprintln!("Failed to parse target JSON: {}", e);
                    }
                }
            }
        }

        println!("Loaded {} targets", self.targets.len());
        Ok(())
    }

    /// Save a target image to the targets directory
    pub fn save_target(&self, target: &TargetImage) -> Result<()> {
        println!("Saving target: id={}, name={}, image_data_length={}",
            target.id, target.name, target.image_data.len());

        let file_path = self.targets_dir.join(format!("{}.json", target.id));
        println!("Target file path: {:?}", file_path);

        let json_content = serde_json::to_string_pretty(target)?;
        println!("JSON content length: {}", json_content.len());

        fs::write(&file_path, &json_content)?;
        println!("Target saved successfully");

        // Verify the file was written correctly
        if let Ok(content) = fs::read_to_string(&file_path) {
            println!("Verified file content length: {}", content.len());
        } else {
            eprintln!("Failed to verify file content");
        }

        Ok(())
    }

    /// Create a new target image from a screenshot region
    pub fn create_target_from_screenshot(
        &self,
        id: &str,
        name: &str,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        threshold: f32,
        click_offset: Option<(i32, i32)>,
    ) -> Result<TargetImage> {
        println!("Creating target from screenshot: {}x{} at ({}, {})", width, height, x, y);

        // Take a screenshot of the specified region
        let screenshot = capture_screen_area(x, y, width, height)?;
        println!("Captured screenshot: {}x{}", screenshot.width(), screenshot.height());

        let image = DynamicImage::ImageRgba8(screenshot);

        // Convert to base64
        let mut buffer = Vec::new();
        image.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;
        let base64_data = general_purpose::STANDARD.encode(&buffer);
        println!("Encoded image to base64, length: {}", base64_data.len());

        // Create the target
        let target = TargetImage {
            id: id.to_string(),
            name: name.to_string(),
            image_data: base64_data,
            threshold,
            click_offset,
        };

        println!("Created target: id={}, name={}, image_data_length={}", id, name, target.image_data.len());

        Ok(target)
    }

    /// Find a target image on the screen
    pub fn find_on_screen(&self, target_id: &str) -> Result<Option<(i32, i32)>> {
        // Find the target by ID
        let target = self.targets.iter()
            .find(|t| t.id == target_id)
            .ok_or_else(|| AppError::ParseError(format!("Target not found: {}", target_id)))?;

        // Take a screenshot of the entire screen
        let screenshot = capture_screen()?;

        // Convert to DynamicImage
        let screen_image = DynamicImage::ImageRgba8(screenshot);

        // Decode the target image from base64
        let target_data = general_purpose::STANDARD.decode(&target.image_data)?;
        let target_image = image::load_from_memory(&target_data)?;

        // Find the target in the screenshot
        match find_template(&screen_image, &target_image, target.threshold) {
            Some((x, y)) => {
                // Apply click offset if specified
                let (x, y) = if let Some((offset_x, offset_y)) = target.click_offset {
                    (x + offset_x, y + offset_y)
                } else {
                    // Default to center of the matched image
                    let (width, height) = target_image.dimensions();
                    (x + (width / 2) as i32, y + (height / 2) as i32)
                };

                Ok(Some((x, y)))
            },
            None => Ok(None),
        }
    }

    /// Get all loaded targets
    pub fn get_targets(&self) -> &[TargetImage] {
        &self.targets
    }

    /// Delete a target by ID
    pub fn delete_target(&mut self, target_id: &str) -> Result<()> {
        let file_path = self.targets_dir.join(format!("{}.json", target_id));

        if file_path.exists() {
            fs::remove_file(file_path)?;
            self.targets.retain(|t| t.id != target_id);
            Ok(())
        } else {
            Err(AppError::ParseError(format!("Target not found: {}", target_id)))
        }
    }
}

/// Find a template image within a larger image using template matching
fn find_template(
    screen: &DynamicImage,
    template: &DynamicImage,
    threshold: f32,
) -> Option<(i32, i32)> {
    // Convert images to grayscale for faster processing
    let screen_gray = screen.to_luma8();
    let template_gray = template.to_luma8();

    let (screen_width, screen_height) = screen_gray.dimensions();
    let (template_width, template_height) = template_gray.dimensions();

    // Ensure template is smaller than screen
    if template_width > screen_width || template_height > screen_height {
        return None;
    }

    // Multi-scale template matching for better accuracy
    let scales = [1.0, 0.9, 0.8, 1.1, 1.2];
    let mut global_best_match = (0.0, 0, 0, 1.0); // (correlation, x, y, scale)

    for &scale in &scales {
        // Skip if scaling would make the template larger than the screen
        let scaled_width = (template_width as f32 * scale) as u32;
        let scaled_height = (template_height as f32 * scale) as u32;

        if scaled_width > screen_width || scaled_height > screen_height || scaled_width < 10 || scaled_height < 10 {
            continue;
        }

        // Resize the template if scale is not 1.0
        let template_to_use = if (scale - 1.0).abs() > 0.01 {
            template.resize(scaled_width, scaled_height, image::imageops::FilterType::Lanczos3).to_luma8()
        } else {
            template_gray.clone()
        };

        // Simple normalized cross-correlation template matching
        let mut best_match = (0.0, 0, 0);

        // Step size for faster scanning (check every 2nd pixel first)
        let initial_step = 2;
        let mut found_match = false;

        // First pass: scan with step size
        for y in (0..=(screen_height - scaled_height)).step_by(initial_step) {
            for x in (0..=(screen_width - scaled_width)).step_by(initial_step) {
                // Calculate correlation values
                let mut template_sum_squared = 0.0;
                let mut screen_sum_squared = 0.0;
                let mut cross_correlation = 0.0;

                // Calculate normalized cross-correlation for this position
                // Use sampling to speed up initial pass
                for ty in (0..scaled_height).step_by(initial_step) {
                    for tx in (0..scaled_width).step_by(initial_step) {
                        let template_pixel = template_to_use.get_pixel(tx, ty)[0] as f32;
                        let screen_pixel = screen_gray.get_pixel(x + tx, y + ty)[0] as f32;

                        cross_correlation += template_pixel * screen_pixel;
                        template_sum_squared += template_pixel * template_pixel;
                        screen_sum_squared += screen_pixel * screen_pixel;
                    }
                }

                // Normalize the correlation
                let denominator = (template_sum_squared * screen_sum_squared).sqrt();
                let correlation = if denominator > 0.0 {
                    cross_correlation / denominator
                } else {
                    0.0
                };

                // Update best match if this is better
                if correlation > best_match.0 {
                    best_match = (correlation, x as i32, y as i32);
                    if correlation >= threshold * 0.8 { // Lower threshold for first pass
                        found_match = true;
                    }
                }
            }
        }

        // If we found a good match in the first pass, refine it
        if found_match {
            // Define a search area around the best match
            let search_radius = initial_step * 2;
            let search_radius_i32 = search_radius as i32;
            let start_x = (best_match.1 - search_radius_i32).max(0) as u32;
            let start_y = (best_match.2 - search_radius_i32).max(0) as u32;
            let end_x = (best_match.1 + search_radius_i32).min((screen_width - scaled_width) as i32) as u32;
            let end_y = (best_match.2 + search_radius_i32).min((screen_height - scaled_height) as i32) as u32;

            // Second pass: detailed scan in the refined area
            for y in start_y..=end_y {
                for x in start_x..=end_x {
                    // Calculate correlation values
                    let mut template_sum_squared = 0.0;
                    let mut screen_sum_squared = 0.0;
                    let mut cross_correlation = 0.0;

                    // Calculate normalized cross-correlation for this position
                    for ty in 0..scaled_height {
                        for tx in 0..scaled_width {
                            let template_pixel = template_to_use.get_pixel(tx, ty)[0] as f32;
                            let screen_pixel = screen_gray.get_pixel(x + tx, y + ty)[0] as f32;

                            cross_correlation += template_pixel * screen_pixel;
                            template_sum_squared += template_pixel * template_pixel;
                            screen_sum_squared += screen_pixel * screen_pixel;
                        }
                    }

                    // Normalize the correlation
                    let denominator = (template_sum_squared * screen_sum_squared).sqrt();
                    let correlation = if denominator > 0.0 {
                        cross_correlation / denominator
                    } else {
                        0.0
                    };

                    // Update best match if this is better
                    if correlation > best_match.0 {
                        best_match = (correlation, x as i32, y as i32);
                    }
                }
            }
        }

        // Update global best match if this scale is better
        if best_match.0 > global_best_match.0 {
            global_best_match = (best_match.0, best_match.1, best_match.2, scale);
        }
    }

    // Return the best match if it exceeds the threshold
    if global_best_match.0 >= threshold {
        Some((global_best_match.1, global_best_match.2))
    } else {
        None
    }
}

/// Convert a base64 encoded image to a DynamicImage
pub fn base64_to_image(base64_data: &str) -> Result<DynamicImage> {
    println!("Converting base64 to image, data length: {}", base64_data.len());

    // Check if the base64 data is valid
    if base64_data.is_empty() {
        return Err(AppError::ParseError("Empty base64 data".to_string()));
    }

    // Decode the base64 data
    let image_data = match general_purpose::STANDARD.decode(base64_data) {
        Ok(data) => {
            println!("Successfully decoded base64 data, length: {}", data.len());
            data
        },
        Err(e) => {
            eprintln!("Failed to decode base64 data: {}", e);
            return Err(AppError::ParseError(format!("Failed to decode base64: {}", e)));
        }
    };

    // Load the image from memory
    match image::load_from_memory(&image_data) {
        Ok(image) => {
            println!("Successfully loaded image from memory, dimensions: {}x{}", image.width(), image.height());
            Ok(image)
        },
        Err(e) => {
            eprintln!("Failed to load image from memory: {}", e);
            Err(AppError::ParseError(format!("Failed to load image: {}", e)))
        }
    }
}

/// Convert a DynamicImage to base64 encoded string
pub fn image_to_base64(image: &DynamicImage) -> Result<String> {
    let mut buffer = Vec::new();
    image.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;
    let base64_data = general_purpose::STANDARD.encode(&buffer);
    Ok(base64_data)
}

/// Capture a screenshot of the entire screen using the Windows API
pub fn capture_screen() -> Result<RgbaImage> {
    unsafe {
        // Get the device context for the entire screen
        let screen_dc = Gdi::GetDC(HWND(0));
        if screen_dc.is_invalid() {
            return Err(AppError::ParseError("Failed to get screen DC".to_string()));
        }

        // Get screen dimensions
        let screen_width = Gdi::GetDeviceCaps(screen_dc, Gdi::HORZRES);
        let screen_height = Gdi::GetDeviceCaps(screen_dc, Gdi::VERTRES);

        // Create a compatible DC for the screen
        let compatible_dc = Gdi::CreateCompatibleDC(screen_dc);
        if compatible_dc.is_invalid() {
            Gdi::ReleaseDC(HWND(0), screen_dc);
            return Err(AppError::ParseError("Failed to create compatible DC".to_string()));
        }

        // Create a compatible bitmap
        let bitmap = Gdi::CreateCompatibleBitmap(screen_dc, screen_width, screen_height);
        if bitmap.is_invalid() {
            Gdi::DeleteDC(compatible_dc);
            Gdi::ReleaseDC(HWND(0), screen_dc);
            return Err(AppError::ParseError("Failed to create compatible bitmap".to_string()));
        }

        // Select the bitmap into the compatible DC
        let old_bitmap = Gdi::SelectObject(compatible_dc, bitmap);

        // Copy the screen to the bitmap
        if Gdi::BitBlt(
            compatible_dc,
            0, 0,
            screen_width, screen_height,
            screen_dc,
            0, 0,
            Gdi::SRCCOPY
        ).is_err() {
            Gdi::SelectObject(compatible_dc, old_bitmap);
            Gdi::DeleteObject(bitmap);
            Gdi::DeleteDC(compatible_dc);
            Gdi::ReleaseDC(HWND(0), screen_dc);
            return Err(AppError::ParseError("Failed to copy screen to bitmap".to_string()));
        }

        // Get bitmap information
        let mut bitmap_info = Gdi::BITMAPINFO {
            bmiHeader: Gdi::BITMAPINFOHEADER {
                biSize: size_of::<Gdi::BITMAPINFOHEADER>() as u32,
                biWidth: screen_width,
                biHeight: -screen_height, // Negative height for top-down DIB
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0, // BI_RGB = 0
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [Gdi::RGBQUAD::default()],
        };

        // Create a buffer for the bitmap data
        let buffer_size = (screen_width * screen_height * 4) as usize;
        // Use with_capacity to avoid unnecessary initialization
        let mut buffer = Vec::with_capacity(buffer_size);
        buffer.resize(buffer_size, 0);

        // Get the bitmap data
        if Gdi::GetDIBits(
            compatible_dc,
            bitmap,
            0,
            screen_height as u32,
            Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
            &mut bitmap_info,
            Gdi::DIB_RGB_COLORS
        ) == 0 {
            Gdi::SelectObject(compatible_dc, old_bitmap);
            Gdi::DeleteObject(bitmap);
            Gdi::DeleteDC(compatible_dc);
            Gdi::ReleaseDC(HWND(0), screen_dc);
            return Err(AppError::ParseError("Failed to get bitmap data".to_string()));
        }

        // Clean up
        Gdi::SelectObject(compatible_dc, old_bitmap);
        Gdi::DeleteObject(bitmap);
        Gdi::DeleteDC(compatible_dc);
        Gdi::ReleaseDC(HWND(0), screen_dc);

        // Convert BGRA to RGBA
        for i in (0..buffer_size).step_by(4) {
            let b = buffer[i];
            buffer[i] = buffer[i + 2];
            buffer[i + 2] = b;
        }

        // Create an RgbaImage from the buffer
        RgbaImage::from_raw(screen_width as u32, screen_height as u32, buffer)
            .ok_or_else(|| AppError::ParseError("Failed to create image from buffer".to_string()))
    }
}

/// Capture a screenshot of a specific area of the screen
pub fn capture_screen_area(x: i32, y: i32, width: u32, height: u32) -> Result<RgbaImage> {
    let full_screenshot = capture_screen()?;
    let cropped = DynamicImage::ImageRgba8(full_screenshot)
        .crop(x as u32, y as u32, width, height);
    Ok(cropped.to_rgba8())
}
