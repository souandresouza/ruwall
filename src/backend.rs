/// Native color generation backend (kmeans) written in Rust.
///
/// Mirrors the "fast_colorthief" backend behaviour of upstream pywal:
/// a palette is extracted from the image, sorted by YIQ and adjusted into
/// a 16 color scheme for dark/light themes.

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

fn squared_dist(a: [u8; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] as f64 - b[0];
    let dy = a[1] as f64 - b[1];
    let dz = a[2] as f64 - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Lloyd's kmeans with deterministic (seeded) kmeans++ initialisation.
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

    // Lloyd iterations.
    for _ in 0..30 {
        let mut sums = vec![[0.0f64; 3]; k];
        let mut counts = vec![0usize; k];

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
        if !changed {
            break;
        }
    }

    centers
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