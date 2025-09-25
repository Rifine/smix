use std::{collections::HashMap, io::{stdout, Write}, path::{PathBuf}};

use anyhow::{ensure};
use clap::{Parser, ValueEnum};
use eframe::egui;
use smix::{Mask};

use crate::gui::PreView;

pub mod gui;

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
    filter: Filter,

    /// Setup a preview gui
    #[arg(short, long, default_value = "true")]
    preview: bool
}

fn main() -> anyhow::Result<()> {
    let mut env = Env::new();

    env.ensure_args()?;

    env.load_mask()?;

    if env.args.preview {
        return env.preview();
    } else {
        env.generate()?;
    }

    Ok(())
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

pub struct Env {
    args: Args,
    masks: HashMap<String, Mask>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            args: Args::parse(),
            masks: HashMap::new(),
        }
    }

    pub fn preview(self) -> anyhow::Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_min_inner_size([768.0, 512.0]).into(),
            ..Default::default()
        };
        let _ = eframe::run_native(
            "smix preview",
            options,
            Box::new(|_cc| Ok(
                Box::new(
                    PreView::new([self.args.r, self.args.g, self.args.b], self.masks)
                )
            )
        ));
        Ok(())
    }

    pub fn generate(self) -> anyhow::Result<()> {
        let weight = &[self.args.r, self.args.g, self.args.b];
        for (i, &s) in self.args.scale.iter().enumerate() {
            for (name, mask) in &self.masks {
                let img = mask.generate(weight);
                if s < 0.0 {
                    println!("Scale factor should be positive, but {s} at {i} is negative");
                    continue;
                }
                let (width, height) = img.dimensions();
                let nwidth = (width as f32 * s) as u32;
                let nheight = (height as f32 * s) as u32;
                let output_name = img.export_name(&name, nwidth, nheight);

                print!("Generating {output_name}...");
                stdout().flush()?;
                if s == 1.0 {
                    img.save(self.args.output.join(output_name))?;
                } else {
                    img.save_as(self.args.output.join(output_name), nwidth, nheight, self.args.filter.into())?;
                }
                println!("done");
            }
        }
        Ok(())
    }

    pub fn load_mask(&mut self) -> anyhow::Result<()> {
        for path in &self.args.mask_directories {
            let mask = Mask::new(&path)?;
            let name = format!("{}", path.display());
            let name = name.split("/").last().unwrap_or("result");
            self.masks.insert(name.into(), mask);
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