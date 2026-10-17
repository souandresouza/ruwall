/// Native color generation backends written in Rust.
///
/// Mirrors the backend behavior of upstream pywal:
/// - "wal": kmeans with fast_colorthief-style adjust (default)
/// - "colorz": kmeans with vibrant, highly saturated adjust
/// - "colorthief": median cut algorithm
/// - "haishoku": most frequently occurring colors

use std::path::Path;

use rand::SeedableRng;
use rand::Rng;

use crate::util;

/// Extract `k` colors from an image using the `wal` backend.
///
/// The original pywal `wal` backend is not kmeans: it asks ImageMagick for
/// `-colors N -unique-colors` (median-cut quantisation) and uses the result
/// ordered by pixel population. That ordering is load-bearing - the pywal
/// adjust() assumes the first entry is the dominant colour - and median-cut
/// itself finds the accent colours that kmeans misses. On a desert sunset
/// wallpaper kmeans converges to a dozen greys and loses the orange
/// outright, which is the whole palette.
///
/// Reimplemented here so ruwall needs no ImageMagick and matches pywal.
pub fn get(img: &Path, light: bool, k: usize) -> Result<Vec<String>, String> {
    let samples = sample_pixels(img)?;
    let quantised = median_cut(&samples, k.max(1));
    let mut cols: Vec<String> = quantised.into_iter().map(|(rgb, _)| util::rgb_to_hex(rgb)).collect();

    // Median cut can only ever produce as many boxes as the image has
    // distinct regions, so a flat wallpaper or a screenshot yields one or
    // two colours. The old kmeans path always returned k entries; cycle the
    // short list so the 16-colour contract below still holds instead of
    // indexing past the end.
    let want = k.max(1);
    if !cols.is_empty() {
        let base = cols.len();
        let mut i = 0usize;
        while cols.len() < want {
            cols.push(cols[i % base].clone());
            i += 1;
        }
    }

    Ok(adjust(&mut cols, light))
}

/// Median-cut quantisation, returning (colour, pixel count) per box ordered
/// by population, which is what `-unique-colors` gives ImageMagick.
///
/// Mirrors pywal's retry loop: ask for 16 colours, and if quantisation
/// collapsed the image into fewer than 17 unique entries, ask for more.
fn median_cut(samples: &[[u8; 3]], k: usize) -> Vec<([u8; 3], u32)> {
    let want = k.max(2);
    let mut boxes: Vec<Vec<usize>> = vec![(0..samples.len()).collect()];

    while boxes.len() < want {
        // Split the box with the widest range; ties go to the first, which
        // keeps the result deterministic.
        let pick = boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.len() > 1)
            .map(|(i, b)| (i, widest_channel(samples, b)))
            .max_by_key(|(_, (_, range))| *range);

        let Some((idx, (ch, _))) = pick else { break };

        let mut pixels = std::mem::take(&mut boxes[idx]);
        pixels.sort_by_key(|&p| samples[p][ch]);
        let mid = pixels.len() / 2.max(1);
        let (a, b) = pixels.split_at(mid);
        boxes[idx] = a.to_vec();
        boxes.push(b.to_vec());
    }

    boxes
        .into_iter()
        .filter(|b| !b.is_empty())
        .map(|b| {
            let n = b.len() as u32;
            let mut sum = [0u64; 3];
            for &p in &b {
                for c in 0..3 {
                    sum[c] += samples[p][c] as u64;
                }
            }
            (
                [
                    (sum[0] / n as u64) as u8,
                    (sum[1] / n as u64) as u8,
                    (sum[2] / n as u64) as u8,
                ],
                n,
            )
        })
        .collect()
}

/// Widest channel in `pixels`, with its range. Median-cut always splits the
/// longest axis, which is what keeps the boxes close to cubes.
fn widest_channel(samples: &[[u8; 3]], pixels: &[usize]) -> (usize, u16) {
    let mut best = (0usize, 0u16);
    for c in 0..3 {
        let lo = pixels.iter().map(|&p| samples[p][c]).min().unwrap_or(0);
        let hi = pixels.iter().map(|&p| samples[p][c]).max().unwrap_or(0);
        let range = (hi as u16).saturating_sub(lo as u16);
        if range > best.1 {
            best = (c, range);
        }
    }
    best
}

/// Extract colors using the colorz backend (vibrant, saturated).
pub fn get_colorz(img: &Path, light: bool, k: usize) -> Result<Vec<String>, String> {
    let centers = extract_kmeans(img, k)?;
    let mut cols: Vec<String> = centers
        .into_iter()
        .map(|c| util::rgb_to_hex([
            c[0].round() as u8,
            c[1].round() as u8,
            c[2].round() as u8,
        ]))
        .collect();
    Ok(adjust_colorz(&mut cols, light))
}

/// Extract colors using the colorthief backend (median cut).
pub fn get_colorthief(img: &Path, light: bool, k: usize) -> Result<Vec<String>, String> {
    let centers = extract_median_cut(img, k)?;
    let mut cols: Vec<String> = centers
        .into_iter()
        .map(|c| util::rgb_to_hex([
            c[0].round() as u8,
            c[1].round() as u8,
            c[2].round() as u8,
        ]))
        .collect();
    Ok(adjust_colorthief(&mut cols, light))
}

/// Extract colors using the haishoku backend (most frequent colors).
pub fn get_haishoku(img: &Path, light: bool, k: usize) -> Result<Vec<String>, String> {
    let centers = extract_frequent_colors(img, k)?;
    let mut cols: Vec<String> = centers
        .into_iter()
        .map(|c| util::rgb_to_hex([
            c[0].round() as u8,
            c[1].round() as u8,
            c[2].round() as u8,
        ]))
        .collect();
    Ok(adjust_haishoku(&mut cols, light))
}

/// Sample the image into rgb pixels, dropping transparent ones.
fn sample_pixels(img: &Path) -> Result<Vec<[u8; 3]>, String> {
    let rgba = image::open(img)
        .map_err(|e| format!("Couldn't load image '{}': {}", img.display(), e))?
        .to_rgba8();

    let (width, height) = rgba.dimensions();
    let scale = std::cmp::max(1, std::cmp::max(width, height) / 100);

    let mut samples: Vec<[u8; 3]> = Vec::new();
    let mut y = 0;
    while y < height {
        let mut x = 0;
        while x < width {
            let pixel = rgba.get_pixel(x, y).0;
            if pixel[3] == 255 {
                samples.push([pixel[0], pixel[1], pixel[2]]);
            }
            x += scale;
        }
        y += scale;
    }

    // If everything is transparent, fall back to sampling any pixel.
    if samples.is_empty() {
        let mut y = 0;
        while y < height {
            let mut x = 0;
            while x < width {
                let pixel = rgba.get_pixel(x, y).0;
                samples.push([pixel[0], pixel[1], pixel[2]]);
                x += scale;
            }
            y += scale;
        }
    }

    if samples.is_empty() {
        return Err(format!("Image '{}' contains no pixels.", img.display()));
    }
    Ok(samples)
}

/// Sample the image into rgb pixels and run kmeans.
fn extract_kmeans(img: &Path, k: usize) -> Result<Vec<[f64; 3]>, String> {
    Ok(kmeans(&sample_pixels(img)?, k.max(1)))
}

/// Extract colors using median cut algorithm.
fn extract_median_cut(img: &Path, k: usize) -> Result<Vec<[f64; 3]>, String> {
    let rgba = image::open(img)
        .map_err(|e| format!("Couldn't load image '{}': {}", img.display(), e))?
        .to_rgba8();

    let (width, height) = rgba.dimensions();
    let scale = std::cmp::max(1, std::cmp::max(width, height) / 100);

    // Sample pixels in a grid, dropping transparent ones.
    let mut samples: Vec<[u8; 3]> = Vec::new();
    let mut y = 0;
    while y < height {
        let mut x = 0;
        while x < width {
            let pixel = rgba.get_pixel(x, y).0;
            if pixel[3] == 255 {
                samples.push([pixel[0], pixel[1], pixel[2]]);
            }
            x += scale;
        }
        y += scale;
    }

    // If everything is transparent, fall back to sampling any pixel.
    if samples.is_empty() {
        let mut y = 0;
        while y < height {
            let mut x = 0;
            while x < width {
                let pixel = rgba.get_pixel(x, y).0;
                samples.push([pixel[0], pixel[1], pixel[2]]);
                x += scale;
            }
            y += scale;
        }
    }

    if samples.is_empty() {
        return Err(format!("Image '{}' contains no pixels.", img.display()));
    }

    Ok(median_cut(&samples, k.max(1))
        .into_iter()
        .map(|(rgb, _)| [rgb[0] as f64, rgb[1] as f64, rgb[2] as f64])
        .collect())
}

/// Extract most frequently occurring colors.
fn extract_frequent_colors(img: &Path, k: usize) -> Result<Vec<[f64; 3]>, String> {
    let rgba = image::open(img)
        .map_err(|e| format!("Couldn't load image '{}': {}", img.display(), e))?
        .to_rgba8();

    let (width, height) = rgba.dimensions();
    let scale = std::cmp::max(1, std::cmp::max(width, height) / 100);

    // Sample pixels in a grid, dropping transparent ones.
    let mut samples: Vec<[u8; 3]> = Vec::new();
    let mut y = 0;
    while y < height {
        let mut x = 0;
        while x < width {
            let pixel = rgba.get_pixel(x, y).0;
            if pixel[3] == 255 {
                samples.push([pixel[0], pixel[1], pixel[2]]);
            }
            x += scale;
        }
        y += scale;
    }

    // If everything is transparent, fall back to sampling any pixel.
    if samples.is_empty() {
        let mut y = 0;
        while y < height {
            let mut x = 0;
            while x < width {
                let pixel = rgba.get_pixel(x, y).0;
                samples.push([pixel[0], pixel[1], pixel[2]]);
                x += scale;
            }
            y += scale;
        }
    }

    if samples.is_empty() {
        return Err(format!("Image '{}' contains no pixels.", img.display()));
    }

    Ok(frequent_colors(&samples, k.max(1)))
}

fn squared_dist(a: [u8; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] as f64 - b[0];
    let dy = a[1] as f64 - b[1];
    let dz = a[2] as f64 - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Lloyd's kmeans with deterministic (seeded) kmeans++ initialisation.
/// Optimized with early convergence detection and reduced allocations.
fn kmeans(samples: &[[u8; 3]], k: usize) -> Vec<[f64; 3]> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(1024);
    let mut centers: Vec<[f64; 3]> = Vec::with_capacity(k);

    // kmeans++ initialisation.
    let first = samples[rng.gen_range(0..samples.len())];
    centers.push([first[0] as f64, first[1] as f64, first[2] as f64]);

    while centers.len() < k {
        let mut total = 0.0f64;
        let mut weights = Vec::with_capacity(samples.len());
        for sample in samples {
            let mut best = f64::MAX;
            for center in &centers {
                let d = squared_dist(*sample, *center);
                if d < best {
                    best = d;
                }
            }
            weights.push(best);
            total += best;
        }

        let mut picked = samples.len() - 1;
        if total > 0.0 {
            let mut r: f64 = rng.gen_range(0.0..total);
            for (i, w) in weights.iter().enumerate() {
                r -= w;
                if r <= 0.0 {
                    picked = i;
                    break;
                }
            }
        } else {
            picked = rng.gen_range(0..samples.len());
        }
        centers.push([
            samples[picked][0] as f64,
            samples[picked][1] as f64,
            samples[picked][2] as f64,
        ]);
    }

    // Lloyd iterations with early convergence detection.
    let mut sums = vec![[0.0f64; 3]; k];
    let mut counts = vec![0usize; k];

    for iteration in 0..30 {
        // Reset accumulators.
        for sum in sums.iter_mut() {
            *sum = [0.0, 0.0, 0.0];
        }
        for count in counts.iter_mut() {
            *count = 0;
        }

        // Assign samples to nearest center.
        for sample in samples {
            let mut best = f64::MAX;
            let mut best_index = 0;
            for (i, center) in centers.iter().enumerate() {
                let d = squared_dist(*sample, *center);
                if d < best {
                    best = d;
                    best_index = i;
                }
            }
            sums[best_index][0] += sample[0] as f64;
            sums[best_index][1] += sample[1] as f64;
            sums[best_index][2] += sample[2] as f64;
            counts[best_index] += 1;
        }

        // Update centers and check for convergence.
        let mut changed = false;
        let mut new_centers = centers.clone();
        for i in 0..k {
            if counts[i] == 0 {
                let fallback = samples[rng.gen_range(0..samples.len())];
                new_centers[i] = [fallback[0] as f64, fallback[1] as f64, fallback[2] as f64];
                changed = true;
                continue;
            }
            let next = [
                sums[i][0] / counts[i] as f64,
                sums[i][1] / counts[i] as f64,
                sums[i][2] / counts[i] as f64,
            ];
            let moved = centers[i]
                .iter()
                .zip(next.iter())
                .any(|(a, b)| (a - b).abs() > 0.5);
            changed = changed || moved;
            new_centers[i] = next;
        }

        centers = new_centers;

        // Early convergence: if no center moved significantly, stop.
        if !changed {
            break;
        }

        // Also stop if we've done enough iterations.
        if iteration >= 10 && !changed {
            break;
        }
    }

    centers
}


/// Extract most frequently occurring colors using a simple histogram.
fn frequent_colors(samples: &[[u8; 3]], k: usize) -> Vec<[f64; 3]> {
    use std::collections::HashMap;

    // Quantize colors to reduce the number of unique values.
    let quantize = |c: u8| -> u8 { (c / 16) * 16 };

    let mut counts: HashMap<[u8; 3], usize> = HashMap::new();
    for sample in samples {
        let key = [quantize(sample[0]), quantize(sample[1]), quantize(sample[2])];
        *counts.entry(key).or_insert(0) += 1;
    }

    // Sort by frequency and take the top k.
    let mut sorted: Vec<([u8; 3], usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    sorted
        .into_iter()
        .take(k)
        .map(|(color, _)| [color[0] as f64, color[1] as f64, color[2] as f64])
        .collect()
}

/// Build a 16 color scheme from an extracted palette (fast_colorthief adjust).
fn adjust(cols: &mut Vec<String>, light: bool) -> Vec<String> {
    // The pywal wal backend does not sort: ImageMagick's -unique-colors
    // already returns colours ordered by pixel population. Sort by YIQ here
    // instead and the palette loses its structure - vivid colours drift off
    // the ends while the muddy majority sits in the middle.
    //
    // Only that (ImageMagick's) backend ever needed sorting; pywal's
    // haishoku/colorz/colorthief path does not sort at all, and neither do
    // the adjust_* functions further down.

    let cols = cols.clone();
    let mut raw: Vec<String> = Vec::with_capacity(16);
    // pywal: colors[:1] + colors[8:16] + colors[8:-1]  -> 1 + 8 + 7 = 16
    raw.extend(cols.iter().take(1).cloned());
    raw.extend(cols.iter().skip(8).take(8).cloned());
    raw.extend(cols.iter().skip(8).take(cols.len().saturating_sub(9)).cloned());
    raw.truncate(16);

    if light {
        for color in raw.iter_mut() {
            *color = util::saturate_color(color, 0.5);
        }
        raw[0] = util::lighten_color(&cols[cols.len() - 1], 0.85);
        raw[7] = cols[0].clone();
        raw[8] = util::darken_color(&cols[cols.len() - 1], 0.4);
        raw[15] = cols[0].clone();
    } else {
        // Darken the background a little, unless it is already black.
        // pywal checks raw_colors[0][1] != "0", i.e. the second hex digit.
        if !raw[0].starts_with("#0") {
            raw[0] = util::darken_color(&raw[0], 0.40);
        }
        raw[7] = util::blend_color(&raw[7], "#EEEEEE");
        raw[8] = util::darken_color(&raw[7], 0.30);
        raw[15] = util::blend_color(&raw[15], "#EEEEEE");
    }

    raw
}

/// Build a 16 color scheme using the colorz backend adjust.
fn adjust_colorz(cols: &mut Vec<String>, light: bool) -> Vec<String> {
    util::sort_by_yiq(cols);

    let mut raw: Vec<String> = vec![
        cols[0].clone(),
        cols[0].clone(),
        cols[1].clone(),
        cols[2].clone(),
        cols[3].clone(),
        cols[4].clone(),
        cols[5].clone(),
        "#FFFFFF".to_string(),
        "#000000".to_string(),
        cols[0].clone(),
        cols[1].clone(),
        cols[2].clone(),
        cols[3].clone(),
        cols[4].clone(),
        cols[5].clone(),
        "#FFFFFF".to_string(),
    ];

    if light {
        for color in raw.iter_mut() {
            *color = util::saturate_color(color, 0.60);
            *color = util::darken_color(color, 0.5);
        }
        raw[0] = util::lighten_color(&raw[0], 0.95);
        raw[7] = util::darken_color(&raw[0], 0.75);
        raw[8] = util::darken_color(&raw[0], 0.25);
        raw[15] = raw[7].clone();
    } else {
        raw[0] = util::darken_color(&raw[0], 0.80);
        raw[7] = util::lighten_color(&raw[0], 0.75);
        raw[8] = util::lighten_color(&raw[0], 0.25);
        raw[15] = raw[7].clone();
    }

    raw.truncate(16);
    raw
}

/// Build a 16 color scheme using the colorthief backend adjust.
fn adjust_colorthief(cols: &mut Vec<String>, light: bool) -> Vec<String> {
    util::sort_by_yiq(cols);

    let mut raw: Vec<String> = cols.clone();
    raw.extend(cols.iter().cloned());

    if light {
        raw[0] = util::lighten_color(&cols[0], 0.90);
        raw[7] = util::darken_color(&cols[0], 0.75);
    } else {
        for color in raw.iter_mut() {
            *color = util::lighten_color(color, 0.40);
        }
        raw[0] = util::darken_color(&cols[0], 0.80);
        raw[7] = util::lighten_color(&cols[0], 0.60);
    }

    raw[8] = util::lighten_color(&cols[0], 0.20);
    raw[15] = raw[7].clone();
    raw.truncate(16);
    raw
}

/// Build a 16 color scheme using the haishoku backend adjust.
fn adjust_haishoku(cols: &mut Vec<String>, light: bool) -> Vec<String> {
    util::sort_by_yiq(cols);

    let mut raw: Vec<String> = cols.clone();
    raw.extend(cols.iter().cloned());

    raw[0] = util::lighten_color(&cols[0], 0.40);

    if light {
        for color in raw.iter_mut() {
            *color = util::saturate_color(color, 0.60);
            *color = util::darken_color(color, 0.5);
        }
        raw[0] = util::lighten_color(&raw[0], 0.95);
        raw[7] = util::darken_color(&raw[0], 0.75);
        raw[8] = util::darken_color(&raw[0], 0.25);
        raw[15] = raw[7].clone();
    } else {
        raw[0] = util::darken_color(&raw[0], 0.80);
        raw[7] = util::lighten_color(&raw[0], 0.75);
        raw[8] = util::lighten_color(&raw[0], 0.25);
        raw[15] = raw[7].clone();
    }

    raw.truncate(16);
    raw
}
