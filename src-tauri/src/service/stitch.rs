use image::RgbaImage;
use serde::Serialize;
use std::time::Instant;

pub struct Stitcher {
    pub current_image: RgbaImage,
    last_added_px: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StitchFrameReport {
    pub added_px: u32,
    pub match_y: Option<u32>,
    pub seeded_match_y: Option<u32>,
    pub template_height: u32,
    pub candidate_count: u32,
    pub match_duration_ms: u128,
    pub best_diff_per_pixel: Option<u32>,
    pub second_best_diff_per_pixel: Option<u32>,
    pub rejected_reason: Option<&'static str>,
}

const MAX_HEIGHT: u32 = 20_000;
const MAX_APPEND_PER_FRAME_PERCENT: u32 = 85;
const MIN_APPEND_PER_FRAME_PX: u32 = 8;
const TEMPLATE_HEIGHT: u32 = 64;
const AMBIGUOUS_MATCH_PERCENT: u64 = 8;

impl Stitcher {
    pub fn new(first_frame: RgbaImage) -> Self {
        Self {
            current_image: first_frame,
            last_added_px: None,
        }
    }

    /// Attempts to stitch a new frame to the current image.
    /// Returns the detected vertical offset.
    pub fn add_frame(&mut self, new_frame: &RgbaImage) -> anyhow::Result<u32> {
        Ok(self.add_frame_with_report(new_frame)?.added_px)
    }

    pub fn add_frame_with_report(
        &mut self,
        new_frame: &RgbaImage,
    ) -> anyhow::Result<StitchFrameReport> {
        let match_started_at = Instant::now();
        let (width, height) = self.current_image.dimensions();
        let (new_w, new_h) = new_frame.dimensions();

        if height >= MAX_HEIGHT {
            anyhow::bail!("Maximum height reached for scrolling capture");
        }

        if width != new_w {
            anyhow::bail!("Frame width mismatch");
        }

        let template_h = TEMPLATE_HEIGHT.min(height).min(new_h);
        if template_h == 0 || new_h <= template_h {
            return Ok(stitch_report(
                0,
                None,
                None,
                template_h,
                0,
                match_started_at.elapsed().as_millis(),
                None,
                None,
                Some("frame_too_small"),
            ));
        }
        let template_y = height - template_h;

        let max_append_h =
            (new_h.saturating_mul(MAX_APPEND_PER_FRAME_PERCENT) / 100).max(MIN_APPEND_PER_FRAME_PX);
        if max_append_h <= MIN_APPEND_PER_FRAME_PX {
            return Ok(stitch_report(
                0,
                None,
                None,
                template_h,
                0,
                match_started_at.elapsed().as_millis(),
                None,
                None,
                Some("frame_too_small"),
            ));
        }
        let min_candidate_y = new_h
            .saturating_sub(template_h)
            .saturating_sub(max_append_h);
        let max_candidate_y = new_h.saturating_sub(template_h);
        if min_candidate_y > max_candidate_y {
            return Ok(stitch_report(
                0,
                None,
                None,
                template_h,
                0,
                match_started_at.elapsed().as_millis(),
                None,
                None,
                Some("frame_too_small"),
            ));
        }

        let mut best_y = min_candidate_y;
        let mut min_diff = u64::MAX;
        let mut second_best: Option<(u32, u64)> = None;
        let candidate_count = max_candidate_y - min_candidate_y + 1;
        let seeded_match_y = self.last_added_px.and_then(|last_added_px| {
            let seed = new_h
                .saturating_sub(template_h)
                .saturating_sub(last_added_px);
            (min_candidate_y..=max_candidate_y)
                .contains(&seed)
                .then_some(seed)
        });

        if let Some(seed_y) = seeded_match_y {
            min_diff =
                self.calculate_diff(new_frame, seed_y, template_y, template_h, width, u64::MAX);
            best_y = seed_y;
        }

        for candidate_y in min_candidate_y..=max_candidate_y {
            if Some(candidate_y) == seeded_match_y {
                continue;
            }
            let diff = self.calculate_diff(
                new_frame,
                candidate_y,
                template_y,
                template_h,
                width,
                min_diff,
            );
            if diff < min_diff {
                second_best = Some((best_y, min_diff));
                min_diff = diff;
                best_y = candidate_y;
            } else if second_best.map_or(true, |(_, second_diff)| diff < second_diff) {
                second_best = Some((candidate_y, diff));
            }
        }

        let best_diff_per_pixel = diff_per_pixel(min_diff, width, template_h);
        let second_best_diff_per_pixel =
            second_best.map(|(_, second_diff)| diff_per_pixel(second_diff, width, template_h));
        let match_duration_ms = match_started_at.elapsed().as_millis();
        let max_tolerable_diff = width as u64 * template_h as u64 * 80;
        if min_diff > max_tolerable_diff {
            self.last_added_px = None;
            return Ok(stitch_report(
                0,
                Some(best_y),
                seeded_match_y,
                template_h,
                candidate_count,
                match_duration_ms,
                Some(best_diff_per_pixel),
                second_best_diff_per_pixel,
                Some("low_confidence_match"),
            ));
        }
        if let Some((second_y, second_diff)) = second_best {
            let nearby_duplicate = best_y.abs_diff(second_y) <= MIN_APPEND_PER_FRAME_PX;
            let ambiguity_margin = min_diff
                .saturating_mul(100 + AMBIGUOUS_MATCH_PERCENT)
                .saturating_add(width as u64 * template_h as u64);
            if !nearby_duplicate && second_diff <= ambiguity_margin {
                self.last_added_px = None;
                return Ok(stitch_report(
                    0,
                    Some(best_y),
                    seeded_match_y,
                    template_h,
                    candidate_count,
                    match_duration_ms,
                    Some(best_diff_per_pixel),
                    second_best_diff_per_pixel,
                    Some("ambiguous_repeating_content"),
                ));
            }
        }

        let new_content_h = new_h - (best_y + template_h);
        if new_content_h < MIN_APPEND_PER_FRAME_PX {
            self.last_added_px = None;
            return Ok(stitch_report(
                0,
                Some(best_y),
                seeded_match_y,
                template_h,
                candidate_count,
                match_duration_ms,
                Some(best_diff_per_pixel),
                second_best_diff_per_pixel,
                Some("tiny_or_no_growth"),
            ));
        }
        if new_content_h.saturating_mul(100) > new_h.saturating_mul(MAX_APPEND_PER_FRAME_PERCENT) {
            self.last_added_px = None;
            return Ok(stitch_report(
                0,
                Some(best_y),
                seeded_match_y,
                template_h,
                candidate_count,
                match_duration_ms,
                Some(best_diff_per_pixel),
                second_best_diff_per_pixel,
                Some("large_jump"),
            ));
        }
        if height + new_content_h > MAX_HEIGHT {
            anyhow::bail!("Maximum height reached for scrolling capture");
        }

        // 5. Extend current_image and append new content without recopying the existing tall image.
        let bytes_per_row = (width * 4) as usize;
        let append_start = ((best_y + template_h) * width * 4) as usize;
        let append_len = new_content_h as usize * bytes_per_row;
        let append_end = append_start + append_len;

        let mut raw = std::mem::take(&mut self.current_image).into_raw();
        raw.reserve(append_len);
        raw.extend_from_slice(&new_frame.as_raw()[append_start..append_end]);

        self.current_image = RgbaImage::from_raw(width, height + new_content_h, raw)
            .ok_or_else(|| anyhow::anyhow!("Failed to extend scrolling capture"))?;
        self.last_added_px = Some(new_content_h);

        Ok(stitch_report(
            new_content_h,
            Some(best_y),
            seeded_match_y,
            template_h,
            candidate_count,
            match_duration_ms,
            Some(best_diff_per_pixel),
            second_best_diff_per_pixel,
            None,
        ))
    }

    /// Calculates the sum of absolute differences between two horizontal strips.
    /// This is where SIMD auto-vectorization happens.
    fn calculate_diff(
        &self,
        new_frame: &RgbaImage,
        new_y: u32,
        old_y: u32,
        h: u32,
        w: u32,
        stop_after: u64,
    ) -> u64 {
        let mut total_diff: u64 = 0;

        for dy in 0..h {
            let row_old = &self.current_image.as_raw()
                [((old_y + dy) * w * 4) as usize..((old_y + dy + 1) * w * 4) as usize];
            let row_new = &new_frame.as_raw()
                [((new_y + dy) * w * 4) as usize..((new_y + dy + 1) * w * 4) as usize];

            // Manual loop for better vectorization hints
            for i in 0..row_old.len() {
                total_diff += (row_old[i] as i64 - row_new[i] as i64).abs() as u64;
                if total_diff > stop_after {
                    return total_diff;
                }
            }
        }

        total_diff
    }
}

fn diff_per_pixel(diff: u64, width: u32, template_height: u32) -> u32 {
    let denominator = u64::from(width).saturating_mul(u64::from(template_height));
    if denominator == 0 {
        return 0;
    }
    (diff / denominator).min(u64::from(u32::MAX)) as u32
}

fn stitch_report(
    added_px: u32,
    match_y: Option<u32>,
    seeded_match_y: Option<u32>,
    template_height: u32,
    candidate_count: u32,
    match_duration_ms: u128,
    best_diff_per_pixel: Option<u32>,
    second_best_diff_per_pixel: Option<u32>,
    rejected_reason: Option<&'static str>,
) -> StitchFrameReport {
    StitchFrameReport {
        added_px,
        match_y,
        seeded_match_y,
        template_height,
        candidate_count,
        match_duration_ms,
        best_diff_per_pixel,
        second_best_diff_per_pixel,
        rejected_reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn row_pattern_image(width: u32, start_row: u32, height: u32) -> RgbaImage {
        RgbaImage::from_fn(width, height, |x, y| {
            let v = ((start_row + y) % 251) as u8;
            Rgba([v, x as u8, 255u8.wrapping_sub(v), 255])
        })
    }

    fn repeating_row_pattern_image(
        width: u32,
        start_row: u32,
        height: u32,
        period: u32,
    ) -> RgbaImage {
        RgbaImage::from_fn(width, height, |x, y| {
            let v = ((start_row + y) % period) as u8;
            Rgba([v, x as u8, 255u8.wrapping_sub(v), 255])
        })
    }

    #[test]
    fn add_frame_appends_only_new_rows_after_scroll() {
        let first = row_pattern_image(4, 0, 100);
        let next = row_pattern_image(4, 20, 100);
        let mut stitcher = Stitcher::new(first);

        let report = stitcher.add_frame_with_report(&next).unwrap();

        assert_eq!(report.added_px, 20);
        assert_eq!(report.match_y, Some(16));
        assert_eq!(report.template_height, 64);
        assert_eq!(report.seeded_match_y, None);
        assert_eq!(report.candidate_count, 37);
        assert_eq!(report.rejected_reason, None);
        assert!(report.best_diff_per_pixel.is_some());
        assert_eq!(stitcher.current_image.dimensions(), (4, 120));
    }

    #[test]
    fn add_frame_uses_previous_scroll_delta_as_seed() {
        let first = row_pattern_image(4, 0, 100);
        let second = row_pattern_image(4, 20, 100);
        let third = row_pattern_image(4, 40, 100);
        let mut stitcher = Stitcher::new(first);

        let first_report = stitcher.add_frame_with_report(&second).unwrap();
        let second_report = stitcher.add_frame_with_report(&third).unwrap();

        assert_eq!(first_report.added_px, 20);
        assert_eq!(second_report.added_px, 20);
        assert_eq!(second_report.match_y, Some(16));
        assert_eq!(second_report.seeded_match_y, Some(16));
        assert_eq!(stitcher.current_image.dimensions(), (4, 140));
    }

    #[test]
    fn add_frame_ignores_identical_frame() {
        let first = row_pattern_image(4, 0, 100);
        let next = row_pattern_image(4, 0, 100);
        let mut stitcher = Stitcher::new(first);

        let added = stitcher.add_frame(&next).unwrap();

        assert_eq!(added, 0);
        assert_eq!(stitcher.current_image.dimensions(), (4, 100));
    }

    #[test]
    fn add_frame_rejects_width_mismatch() {
        let first = row_pattern_image(4, 0, 100);
        let next = row_pattern_image(5, 20, 100);
        let mut stitcher = Stitcher::new(first);

        assert!(stitcher.add_frame(&next).is_err());
    }

    #[test]
    fn add_frame_ignores_tiny_jitter_scroll() {
        let first = row_pattern_image(4, 0, 100);
        let next = row_pattern_image(4, 3, 100);
        let mut stitcher = Stitcher::new(first);

        let added = stitcher.add_frame(&next).unwrap();

        assert_eq!(added, 0);
        assert_eq!(stitcher.current_image.dimensions(), (4, 100));
    }

    #[test]
    fn add_frame_rejects_ambiguous_repeating_content() {
        let first = repeating_row_pattern_image(8, 0, 140, 20);
        let next = repeating_row_pattern_image(8, 20, 140, 20);
        let mut stitcher = Stitcher::new(first);

        let report = stitcher.add_frame_with_report(&next).unwrap();

        assert_eq!(report.added_px, 0);
        assert_eq!(report.rejected_reason, Some("ambiguous_repeating_content"));
        assert!(report.second_best_diff_per_pixel.is_some());
        assert_eq!(stitcher.current_image.dimensions(), (8, 140));
    }

    #[test]
    fn add_frame_rejects_implausibly_large_single_frame_jump() {
        let width = 8;
        let height = 800;
        let first = row_pattern_image(width, 0, height);
        let mut next = row_pattern_image(width, 900, height);

        for y in 0..64 {
            for x in 0..width {
                let pixel = *first.get_pixel(x, height - 64 + y);
                next.put_pixel(x, y, pixel);
            }
        }

        let mut stitcher = Stitcher::new(first);
        let added = stitcher.add_frame(&next).unwrap();

        assert_eq!(added, 0);
        assert_eq!(stitcher.current_image.dimensions(), (width, height));
    }
}
