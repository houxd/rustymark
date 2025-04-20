use std::fs;

use anyhow::Result;
use clap::Parser;
use image::imageops::overlay;
use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::definitions::Image;
use imageproc::drawing::{draw_text_mut, text_size};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use rusttype::{Font, Scale};
use serde::{Deserialize, Serialize};

const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);
fn main() -> Result<()> {
    let mut config: AppConfig = AppConfig::parse();

    if config.text.is_empty() {
        return Err(anyhow::anyhow!("没有提供水印文本，请使用 --text 参数"));
    }

    config.rotate = std::f32::consts::PI / config.rotate;
    let watermark = gen_watermark(&config);

    let covered = cover_image_with_watermark(config.input, watermark);

    covered.save(&config.output).expect("Failed to save image");

    println!("水印已添加，输出文件: {}", &config.output);

    Ok(())
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, clap::Parser)]
#[command(
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct AppConfig {
    #[arg(short, long)]
    text: Vec<String>,

    #[arg(short, long, default_value = "./msyh.ttc")]
    font: String,

    #[arg(short, long, default_value = "./input.png")]
    input: String,

    #[arg(short, long, default_value = "./output.png")]
    output: String,

    #[arg(short, long, default_value_t = -6.0)]
    rotate: f32,

    #[arg(short, long, value_parser = parse_color, default_value = "0,0,0,100")]
    color: [u8; 4],

    #[arg(short, long, default_value_t = 10)]
    margin: u32,

    #[arg(short, long, default_value_t = 0)]
    alpha: u8,
}

fn parse_color(s: &str) -> Result<[u8; 4], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 {
        return Err(format!("颜色格式应为 'R,G,B,A'，但得到 '{}'", s));
    }

    let mut color = [0; 4];
    for (i, part) in parts.iter().enumerate() {
        color[i] = part
            .parse::<u8>()
            .map_err(|_| format!("无法将 '{}' 解析为 0-255 之间的数字", part))?;
    }

    Ok(color)
}

fn cover_image_with_watermark(image_path: String, watermark: RgbaImage) -> RgbaImage {
    let mut image = image::open(image_path).unwrap().to_rgba8();

    let line = image.height() + 120 / watermark.height();
    let column = image.width() + 80 / watermark.width();

    for i in 0..line {
        for j in 0..column {
            overlay(
                &mut image,
                &watermark,
                (i * watermark.width()) as i64 - 60,
                (j * watermark.height()) as i64 - 40,
            );
        }
    }

    image
}

fn gen_watermark(config: &AppConfig) -> RgbaImage {
    let pic = gen_text_pic(config.clone());
    let rotated = rotate_image(pic, config.clone());
    cut_image(rotated, config.clone())
}

fn gen_text_pic(config: AppConfig) -> RgbaImage {
    let width = 1000;
    let height = 600;

    let mut img = ImageBuffer::from_pixel(width, height, TRANSPARENT);

    let inteded_text_height = 24.4;
    let scale = Scale {
        x: inteded_text_height,
        y: inteded_text_height,
    };

    let font = fs::read(config.font).unwrap();
    let font = Font::try_from_vec(font).unwrap();

    let mut longest_text_start_x = 0;
    let mut shortest_text_start_x = 0;
    let mut total_text_height = 0;
    let margin = 10;

    for text in config.text.iter() {
        let (text_width, text_height) = text_size(scale, &font, text);
        let text_start_x = ((width - text_width as u32) / 2) as i32;
        if text_start_x > longest_text_start_x {
            longest_text_start_x = text_width;
        }
        if text_start_x < shortest_text_start_x || shortest_text_start_x == 0 {
            shortest_text_start_x = text_width;
        }

        if text_height > total_text_height {
            total_text_height = text_height;
        }
    }
    let avg_text_width = (longest_text_start_x + shortest_text_start_x) / 2;

    for (index, text) in config.text.iter().enumerate() {
        let (_text_width, text_height) = text_size(scale, &font, text);
        let final_height = get_start_height(
            height,
            config.text.len() as u32,
            index as u32,
            text_height as u32,
            margin,
        );
        // 在图像上绘制文字
        draw_text_mut(
            &mut img,
            Rgba([0, 0, 0, 100]),
            avg_text_width,
            final_height,
            scale,
            &font,
            text,
        );
    }

    img.save("watermark_raw.png").expect("Failed to save image");
    return img;
}

fn rotate_image(img: RgbaImage, config: AppConfig) -> RgbaImage {
    let rotated = rotate_about_center(&img, config.rotate, Interpolation::Bicubic, TRANSPARENT);

    let output_path = "watermark_rotated.png";
    rotated.save(output_path).expect("Failed to save image");
    return rotated;
}

fn cut_image(mut rotated: RgbaImage, config: AppConfig) -> RgbaImage {
    let mut empty_lines = 0;
    let mut empty_columns = 0;
    let mut cutted_height = rotated.height();
    let mut cutted_width = rotated.width();
    let mut top = 0;
    let mut left = 0;

    for y in 0..rotated.height() {
        if is_empty_line(y, &mut rotated, config.alpha) {
            empty_lines += 1;
        } else {
            if empty_lines > config.margin && top == 0 {
                top = empty_lines - config.margin;
            }
            empty_lines = 0;
        }
    }

    if empty_lines > config.margin {
        cutted_height -= empty_lines - config.margin;
    }

    for x in 0..rotated.width() {
        if is_empty_column(x, &mut rotated, config.alpha) {
            empty_columns += 1;
        } else {
            if empty_columns > config.margin && left == 0 {
                left = empty_columns - config.margin;
            }
            empty_columns = 0;
        }
    }
    if empty_columns > 50 {
        cutted_width -= empty_columns - 50;
    }

    let new_width = cutted_width - left;
    let new_height = cutted_height - top;

    let mut cutted = RgbaImage::new(new_width, new_height);
    for x in left..cutted_width {
        for y in top..cutted_height {
            let p = rotated.get_pixel(x, y);
            let d = p.clone();

            *cutted.get_pixel_mut(x - left, y - top) = d;
        }
    }

    let output_path = "watermark_cutted.png";
    cutted.save(output_path).expect("Failed to save image");
    cutted
}

fn get_start_height(height: u32, length: u32, index: u32, text_height: u32, margin: u32) -> i32 {
    let start = (height - ((text_height + margin) * length - margin)) / 2;
    let offset = (text_height + margin) * index;
    return (start + offset) as i32;
}

fn is_empty_line(line: u32, img: &Image<Rgba<u8>>, alpha: u8) -> bool {
    for i in 0..img.width() {
        let p = img.get_pixel(i, line);
        let d = p.clone();
        if d[3] != alpha {
            return false;
        }
    }

    return true;
}

fn is_empty_column(row: u32, img: &mut Image<Rgba<u8>>, alpha: u8) -> bool {
    for i in 0..img.height() {
        let p = img.get_pixel(row, i);
        let d = p.clone();
        if d[3] != alpha {
            return false;
        }
    }

    return true;
}
