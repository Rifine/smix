use std::{collections::HashMap, io::{stdout, Write}, path::{Path, PathBuf}};

use anyhow::{ensure, Ok};
use clap::{Parser, ValueEnum};
use image::{open, Rgba, Rgba32FImage, RgbaImage};
use smix::{mix_pixel};


#[derive(Parser, Debug)]
#[command(author, version, about = "Image mixer (RGB channels only)", long_about = None)]
pub struct Args {
    /// Red channel weight, 0~1 positive float
    r: f32,
    /// Green channel weight, 0~1 positive float
    g: f32,
    /// Blue channel weight, 0~1 positive float
    b: f32,

    /// Output directory (create if missing)
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Directory containing r.png, g.png, b.png
    #[arg(short, long, required = true, value_delimiter = ' ', num_args = 1..)]
    mask_directories: Vec<PathBuf>,

    /// Multiple scale factors; one file per factor (>0)
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    scale: Vec<f32>,

    /// Resize filter used when scaling masks.
    #[arg(short, long, value_enum, default_value_t = Filter::Lanczos3)]
    filter: Filter
}

fn main() -> anyhow::Result<()> {
    let mut tool = Tool::new();

    tool.ensure_args()?;

    tool.load_mask()?;

    tool.generate()?;

    Ok(())
}

pub fn f32img_to_u8img(src: Rgba32FImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);
    for (x, y, p) in src.enumerate_pixels() {
        let [r, g, b, a] = p.0;
        dst.put_pixel(x, y, Rgba([
            (r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (b.clamp(0.0, 1.0) * 255.0).round() as u8,
            (a.clamp(0.0, 1.0) * 255.0).round() as u8
        ]));
    };
    dst
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Filter {
    /// Nearest-neighbor
    Nearest,
    /// Linear interpolation
    Bilinear,
    /// Cubic interpolation
    CatmullRom,
    /// Gaussian blur
    Gaussian,
    /// Lanczos with window 3
    Lanczos3,
}

impl Into<image::imageops::FilterType> for Filter {
    fn into(self) -> image::imageops::FilterType {
        use image::imageops::FilterType::*;
        match self {
            Filter::Nearest => Nearest,
            Filter::Bilinear => Triangle,
            Filter::CatmullRom => CatmullRom,
            Filter::Gaussian => Gaussian,
            Filter::Lanczos3 => Lanczos3
        }
    }
}

pub struct Mask([Rgba32FImage; 3]);

impl Mask {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        Ok(Self([
            open(path.join("r.png"))?.into_rgba32f(),
            open(path.join("g.png"))?.into_rgba32f(),
            open(path.join("b.png"))?.into_rgba32f()
        ]))
    }

    pub fn good(&self) -> bool {
        self.0[0].dimensions() == self.0[1].dimensions() && self.0[1].dimensions() == self.0[2].dimensions()
    }

    pub fn generate(self, weight: &[f32; 3]) -> RgbaImage {
        let (width, height) = self.0[0].dimensions();
        let mut image = Rgba32FImage::new(width, height);
        for (x, y, p) in image.enumerate_pixels_mut() {
            let alpha = self.0[0].get_pixel(x, y).0[3];
            p.0[3] = if alpha == 0.0 { continue } else { alpha };
            let mask = [
                self.0[0].get_pixel(x, y).0,
                self.0[1].get_pixel(x, y).0,
                self.0[2].get_pixel(x, y).0,
            ];
            mix_pixel(&mut p.0, &weight, &mask);
        }
        f32img_to_u8img(image)
    }
}

pub struct Tool {
    args: Args,
    images: HashMap<String, RgbaImage>,
}

impl Tool {
    pub fn new() -> Self {
        Self {
            args: Args::parse(),
            images: HashMap::new(),
        }
    }

    pub fn generate(self) -> anyhow::Result<()> {
        for (i, &s) in self.args.scale.iter().enumerate() {
            for (name, img) in &self.images {
                if s < 0.0 {
                    println!("Scale factor should be positive, but {s} at {i} is negative");
                    continue;
                }
                let (width, height) = img.dimensions();
                let nwidth = (width as f32 * s) as u32;
                let nheight = (height as f32 * s) as u32;

                let output_name = format!("{}_{nwidth}x{nheight}.png", name);
            
                print!("Generating {output_name}...");
                stdout().flush()?;
                if s == 1.0 {
                    img.save(self.args.output.join(output_name))?;
                } else {
                    image::imageops::resize(img, nwidth, nheight, self.args.filter.into()).save(self.args.output.join(output_name))?;
                }
                println!("done");
            }
        }
        Ok(())
    }

    pub fn load_mask(&mut self) -> anyhow::Result<()> {
        let weight = &[self.args.r, self.args.g, self.args.b];
        for path in &self.args.mask_directories {
            let mask = Mask::new(&path)?;
            if mask.good() {
                let name = format!("{}", path.display());
                let name = name.split("/").last().unwrap_or("result");
                self.images.insert(name.into(), mask.generate(weight));
            } else {
                return Err(anyhow::anyhow!("mask: {} has different demensions", path.display()))
            }
        }
        Ok(())
    }

    pub fn ensure_args(&mut self) -> anyhow::Result<()> {
        ensure!(self.args.r >= 0.0 && self.args.r <= 1.0, "Red weight must be in [0, 1]");
        ensure!(self.args.g >= 0.0 && self.args.g <= 1.0, "Green weight must be in [0, 1]");
        ensure!(self.args.b >= 0.0 && self.args.b <= 1.0, "Blue weight must be in [0, 1]");

        println!("RGB weights: ({}, {}, {})", self.args.r, self.args.g, self.args.b);

        self.args.scale.push(1.0);
        self.args.scale.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        self.args.scale.dedup();

        if !self.args.output.exists() {
            println!("Output directory does not exists");
            std::fs::create_dir_all(&self.args.output)?;
            println!("Create directory: {}", self.args.output.display());
        } else  {
            println!("Output directory: {}", self.args.output.display());
        }

        Ok(())
    }
}