use image::{GenericImage, GenericImageView, RgbaImage};

pub struct Stitcher {
    pub current_image: RgbaImage,
}

const MAX_HEIGHT: u32 = 20_000;

impl Stitcher {
    pub fn new(first_frame: RgbaImage) -> Self {
        Self {
            current_image: first_frame,
        }
    }

    /// Attempts to stitch a new frame to the current image.
    /// Returns the detected vertical offset.
    pub fn add_frame(&mut self, new_frame: &RgbaImage) -> anyhow::Result<u32> {
        let (width, height) = self.current_image.dimensions();
        let (new_w, new_h) = new_frame.dimensions();

        if height >= MAX_HEIGHT {
            anyhow::bail!("Maximum height reached for scrolling capture");
        }

        if width != new_w {
            anyhow::bail!("Frame width mismatch");
        }

        // 1. Extract template from the bottom of current_image
        // We use a 64-pixel high strip.
        let template_h = 64.min(height);
        let template_y = height - template_h;
        
        // 2. Find template in new_frame (searching from top to bottom)
        let mut best_y = 0;
        let mut min_diff = u64::MAX;

        for candidate_y in 0..(new_h - template_h) {
            let diff = self.calculate_diff(new_frame, candidate_y, template_y, template_h, width);
            if diff < min_diff {
                min_diff = diff;
                best_y = candidate_y;
            }
            
            // Short-circuit threshold (Tighten slightly for confidence)
            if diff < (width as u64 * template_h as u64 * 2) { 
                break;
            }
        }

        // 3. Validate match (Confidence check)
        // Average 80 units per pixel (0-255 scale) - allow more noise for commercial robustness
        let max_tolerable_diff = width as u64 * template_h as u64 * 80; 
        if min_diff > max_tolerable_diff {
            return Ok(0); 
        }

        // 4. Determine scroll amount
        // The scroll amount is how much new content is at the bottom.
        // If the template (bottom of A) matched at candidate_y in B,
        // then the content in B below candidate_y + template_h is new.
        let new_content_h = new_h - (best_y + template_h);
        if new_content_h == 0 {
            return Ok(0);
        }

        // 5. Extend current_image and append new content
        let mut final_image = RgbaImage::new(width, height + new_content_h);
        final_image.copy_from(&self.current_image, 0, 0)?;
        
        let new_part = new_frame.view(0, best_y + template_h, width, new_content_h);
        final_image.copy_from(&*new_part, 0, height)?;

        self.current_image = final_image;

        Ok(new_content_h)
    }

    /// Calculates the sum of absolute differences between two horizontal strips.
    /// This is where SIMD auto-vectorization happens.
    fn calculate_diff(
        &self, 
        new_frame: &RgbaImage, 
        new_y: u32, 
        old_y: u32, 
        h: u32, 
        w: u32
    ) -> u64 {
        let mut total_diff: u64 = 0;
        
        for dy in 0..h {
            let row_old = &self.current_image.as_raw()[((old_y + dy) * w * 4) as usize .. ((old_y + dy + 1) * w * 4) as usize];
            let row_new = &new_frame.as_raw()[((new_y + dy) * w * 4) as usize .. ((new_y + dy + 1) * w * 4) as usize];
            
            // Manual loop for better vectorization hints
            for i in 0..row_old.len() {
                total_diff += (row_old[i] as i64 - row_new[i] as i64).abs() as u64;
            }
        }
        
        total_diff
    }
}
