use std::path::Path;

use ruwall::{backend, util};

fn write_test_image(path: &Path) {
    // A deterministic 4-colour block image.
    let mut img = image::RgbImage::new(200, 200);
    let blocks: [(u32, u32, [u8; 3]); 4] = [
        (0, 0, [249, 115, 22]),
        (100, 0, [6, 182, 212]),
        (0, 100, [250, 204, 21]),
        (100, 100, [168, 85, 247]),
    ];
    for (bx, by, rgb) in blocks {
        for y in by..by + 100 {
            for x in bx..bx + 100 {
                img.put_pixel(x, y, image::Rgb(rgb));
            }
        }
    }
    img.save(path).unwrap();
}

#[test]
fn test_backend_get_deterministic() {
    let dir = common::isolated_env();
    let img = dir.join("four_blocks.png");
    write_test_image(&img);

    let dark_a = backend::get(&img, false, 16).unwrap();
    let dark_b = backend::get(&img, false, 16).unwrap();
    assert_eq!(dark_a, dark_b);
    assert_eq!(dark_a.len(), 16);

    for color in &dark_a {
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);
    }
}

#[test]
fn test_backend_get_light() {
    let dir = common::isolated_env();
    let img = dir.join("four_blocks.png");
    write_test_image(&img);

    let light = backend::get(&img, true, 16).unwrap();
    assert_eq!(light.len(), 16);
    let dark = backend::get(&img, false, 16).unwrap();
    assert_eq!(dark.len(), 16);
    assert_ne!(light, dark);
}

#[test]
fn test_backend_get_orders_by_population() {
    let dir = common::isolated_env();
    let img = dir.join("dominant.png");

    // 40% orange, with three smaller accents. pywal's adjust() uses the first
    // quantised colour as the background, so the dominant colour must lead.
    let mut image = image::RgbImage::new(200, 200);
    for y in 0..200 {
        for x in 0..200 {
            let color = if x < 40 {
                [6, 182, 212]
            } else if x < 80 {
                [250, 204, 21]
            } else if x < 120 {
                [168, 85, 247]
            } else {
                [249, 115, 22]
            };
            image.put_pixel(x, y, image::Rgb(color));
        }
    }
    image.save(&img).unwrap();

    let palette = backend::get(&img, false, 16).unwrap();
    assert_eq!(palette.len(), 16);

    // The dominant colour is warm. Before ordering by population the first
    // entry was an arbitrary cyan box from the split order.
    let [r, g, b] = util::hex_to_rgb(&palette[0]);
    assert!(r > g && g > b, "background should be the warm dominant colour: {}", palette[0]);

    let distinct: std::collections::HashSet<&String> = palette.iter().collect();
    assert!(
        distinct.len() >= 4,
        "palette should keep the accent colours, got {:?}",
        palette
    );
}

#[test]
fn test_backend_get_missing_image() {
    let dir = common::isolated_env();
    let missing = dir.join("missing.png");
    assert!(backend::get(&missing, false, 16).is_err());
}

mod common;