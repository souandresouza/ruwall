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

/// Extract `k` colors from an image using kmeans.
pub fn get(img: &Path, light: bool, k: usize) -> Result<Vec<String>, String> {
    let centers = extract_kmeans(img, k)?;
    let mut cols: Vec<String> = centers
        .into_iter()
        .map(|c| util::rgb_to_hex([
            c[0].round() as u8,
            c[1].round() as u8,
            c[2].round() as u8,
        ]))
        .collect();
    Ok(adjust(&mut cols, light))
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

/// Sample the image into rgb pixels and run kmeans.
fn extract_kmeans(img: &Path, k: usize) -> Result<Vec<[f64; 3]>, String> {
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

    Ok(kmeans(&samples, k.max(1)))
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

    Ok(median_cut(&samples, k.max(1)))
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

/// Median cut algorithm for color extraction.
fn median_cut(samples: &[[u8; 3]], k: usize) -> Vec<[f64; 3]> {
    if samples.is_empty() {
        return Vec::new();
    }

    // Start with all pixels in one bucket.
    let mut buckets: Vec<Vec<[u8; 3]>> = vec![samples.to_vec()];

    // Split buckets until we have k buckets.
    while buckets.len() < k {
        // Find the bucket with the largest range.
        let mut max_range = 0.0f64;
        let mut max_idx = 0;

        for (i, bucket) in buckets.iter().enumerate() {
            if bucket.len() < 2 {
                continue;
            }
            let (min_r, max_r) = bucket.iter().fold((255u8, 0u8), |(min, max), p| {
                (min.min(p[0]), max.max(p[0]))
            });
            let (min_g, max_g) = bucket.iter().fold((255u8, 0u8), |(min, max), p| {
                (min.min(p[1]), max.max(p[1]))
            });
            let (min_b, max_b) = bucket.iter().fold((255u8, 0u8), |(min, max), p| {
                (min.min(p[2]), max.max(p[2]))
            });

            let range = (max_r - min_r) as f64 + (max_g - min_g) as f64 + (max_b - min_b) as f64;
            if range > max_range {
                max_range = range;
                max_idx = i;
            }
        }

        if max_range == 0.0 {
            break;
        }

        // Split the bucket along the channel with the largest range.
        let bucket = &buckets[max_idx];
        let (min_r, max_r) = bucket.iter().fold((255u8, 0u8), |(min, max), p| {
            (min.min(p[0]), max.max(p[0]))
        });
        let (min_g, max_g) = bucket.iter().fold((255u8, 0u8), |(min, max), p| {
            (min.min(p[1]), max.max(p[1]))
        });
        let (min_b, max_b) = bucket.iter().fold((255u8, 0u8), |(min, max), p| {
            (min.min(p[2]), max.max(p[2]))
        });

        let r_range = (max_r - min_r) as f64;
        let g_range = (max_g - min_g) as f64;
        let b_range = (max_b - min_b) as f64;

        let channel = if r_range >= g_range && r_range >= b_range {
            0
        } else if g_range >= b_range {
            1
        } else {
            2
        };

        // Sort by the channel and split in half.
        let mut sorted = bucket.clone();
        sorted.sort_by(|a, b| a[channel].cmp(&b[channel]));

        let mid = sorted.len() / 2;
        let left = sorted[..mid].to_vec();
        let right = sorted[mid..].to_vec();

        buckets.remove(max_idx);
        buckets.push(left);
        buckets.push(right);
    }

    // Calculate the average color of each bucket.
    buckets
        .iter()
        .map(|bucket| {
            if bucket.is_empty() {
                return [0.0, 0.0, 0.0];
            }
            let sum = bucket.iter().fold([0u64; 3], |mut acc, p| {
                acc[0] += p[0] as u64;
                acc[1] += p[1] as u64;
                acc[2] += p[2] as u64;
                acc
            });
            [
                sum[0] as f64 / bucket.len() as f64,
                sum[1] as f64 / bucket.len() as f64,
                sum[2] as f64 / bucket.len() as f64,
            ]
        })
        .collect()
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
