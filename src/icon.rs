use std::path::Path;

use image::{ImageBuffer, Pixel, Rgba};

const XMAX: u32 = 128;
const YMAX: u32 = 128;
const EMPTY_PIXELS: u32 = 8;

use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

pub fn draw_and_save_icon(seed: &str, path: &Path) {
    let mut rng: Pcg64 = Seeder::from(seed).make_rng();

    let pixel_format_example = Rgba::<u8>([0, 0, 0, 0]);
    let mut imgbuf = ImageBuffer::from_pixel(XMAX, YMAX, pixel_format_example);

    let radius: f32 = (XMAX / 2 - EMPTY_PIXELS) as f32;

    let hue1: f32 = rng.gen::<f32>() * 360.;
    let mut hue2: f32 = rng.gen::<f32>() * 360.;

    const MIN_HUE_DIFF: f32 = 75.;
    const HUE_CORR: f32 = 125.;
    if (hue1 - hue2).abs() < MIN_HUE_DIFF {
        hue2 += HUE_CORR;
    }
    if (hue2 - hue1).abs() < MIN_HUE_DIFF {
        hue2 -= HUE_CORR;
    }
    if hue2 >= 360. {
        hue2 -= 360.;
    }
    let color1 = hsl_to_rgb(hue1, 95., 55.0);
    let color2 = hsl_to_rgb(hue2, 95., 55.0);

    draw_planet(&mut imgbuf, color1, color2, radius);

    imgbuf.save(path).unwrap();
}

pub fn draw_planet(
    imgbuf: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    color1: Rgba<u8>,
    color2: Rgba<u8>,
    radius: f32,
) {
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let fx = x as f32;
        let fy = y as f32;
        let fxmax = XMAX as f32;
        let fymax = YMAX as f32;

        let dist = ((fx - fxmax / 2.).powf(2.) + (fy - fymax / 2.).powf(2.)).sqrt();

        let cool_diagonal_max = cool_diagonal(fxmax, fymax);
        let t = cool_diagonal(fx, fy) / cool_diagonal_max;

        let mut color = mix(color1, color2, t);

        if dist < radius {
            pixel.blend(&color);
        } else if dist < (radius + 1.0) {
            // Antialiasing artigianale
            let blend = (radius + 1.0) - dist;
            color[3] = (blend * 255.) as u8;
            pixel.blend(&color);
        }
    }
}

pub fn mix(color1: Rgba<u8>, color2: Rgba<u8>, t: f32) -> Rgba<u8> {
    let mut result = Rgba::<u8>([0, 0, 0, 255]);
    for ch in 0..3 {
        result[ch] = u8ch(f32ch(color1[ch]) * (1.0 - t) + f32ch(color2[ch]) * t);
    }
    return result;
}

pub fn f32ch(value: u8) -> f32 {
    return (value as f32) / 255.;
}

pub fn u8ch(value: f32) -> u8 {
    return (value * 255.) as u8;
}

pub fn cool_diagonal(x: f32, y: f32) -> f32 {
    return x + y / 2.5;
}

pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Rgba<u8> {
    let h = h / 360.0;
    let s = s / 100.0;
    let l = l / 100.0;

    if s == 0.0 {
        let u = (255. * l) as u8;
        return Rgba::<u8>([u, u, u, 255]);
    }

    let temp1 = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };

    let temp2 = 2.0 * l - temp1;
    let hue = h;

    let one_third = 1.0 / 3.0;
    let temp_r = bound_ratio(hue + one_third);
    let temp_g = bound_ratio(hue);
    let temp_b = bound_ratio(hue - one_third);

    let r = calc_rgb_unit(temp_r, temp1, temp2) as u8;
    let g = calc_rgb_unit(temp_g, temp1, temp2) as u8;
    let b = calc_rgb_unit(temp_b, temp1, temp2) as u8;
    return Rgba::<u8>([r, g, b, 255]);
}

pub fn bound_ratio(r: f32) -> f32 {
    let mut n = r;
    loop {
        let less = n < 0.0;
        let bigger = n > 1.0;
        if !less && !bigger {
            break n;
        }
        if less {
            n += 1.0;
        } else {
            n -= 1.0;
        }
    }
}

fn calc_rgb_unit(unit: f32, temp1: f32, temp2: f32) -> f32 {
    let mut result = temp2;
    if 6.0 * unit < 1.0 {
        result = temp2 + (temp1 - temp2) * 6.0 * unit
    } else if 2.0 * unit < 1.0 {
        result = temp1
    } else if 3.0 * unit < 2.0 {
        result = temp2 + (temp1 - temp2) * (2.0 / 3.0 - unit) * 6.0
    }
    return result * 255.;
}
