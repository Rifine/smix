use core::f32;
use std::{collections::HashMap, path::PathBuf};

use eframe::egui::{self, Slider};
use image::{imageops, RgbaImage};
use rfd::FileDialog;
use smix::Mask;

#[derive(Clone, PartialEq)]
struct Args {
    pub weight: [f32; 3],
    pub scale: f32,
    pub key: String,
}

impl Args {
    pub fn new(weight: [f32; 3], default_key: String) -> Self {
        Self {
            weight,
            scale: 1.0,
            key: default_key
        }
    }
}

pub struct PreView {
    masks: HashMap<String, Mask>,
    tex: Option<egui::TextureHandle>,
    current: Args,
    last: Args,
}

impl PreView {
    pub fn new(weight: [f32; 3], masks: HashMap<String, Mask>) -> Self {
        let init = Args::new(weight, masks.iter().next().map(|(s, _)| s.clone()).unwrap());
        Self {
            masks,
            tex: None,
            current: init,
            last: Args::new([0.0, 0.0, 0.0], "".into()),
        }
    }

    pub fn preview_256x(&self) -> RgbaImage {
        use imageops::FilterType::Nearest;
        let mask = &self.masks[&self.current.key];
        let img = mask.generate(&self.current.weight);
        let img = image::imageops::resize(img.get_rgba(), 256, 256, Nearest);
        img
    }

    pub fn update_preview(&mut self, ctx: &egui::Context) {
        let preview = self.preview_256x();
        let img = egui::ColorImage::from_rgba_unmultiplied([256, 256], preview.as_raw());
        if let Some(handle) = &mut self.tex {
            handle.set(img, egui::TextureOptions::default());
        } else {
            self.tex = Some(ctx.load_texture("preview", img, Default::default()))
        }

        self.last.clone_from(&self.current);
    }
}

impl eframe::App for PreView {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let changed = self.last != self.current;
        if changed {
            self.update_preview(ctx);
        }

        egui::SidePanel::left("Control")
            .min_width(128.0).resizable(false)
            .show(ctx, |ui| {
                // Masks list
                ui.label("Masks");
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        for key in self.masks.keys() {
                            let selected = *key == self.current.key;
                            if ui.selectable_label(selected, key).clicked() {
                                self.current.key = key.clone();
                            }
                        }
                    }
                );
            }
        );

        egui::SidePanel::right("Args")
            .resizable(false)
            .show(ctx, |ui| {
                // Args setting
                ui.vertical(|ui| {
                    ui.label("Weights:");
                    ui.add(Slider::new(&mut self.current.weight[0], 0.0..=1.0).text("R").step_by(0.01));
                    ui.add(Slider::new(&mut self.current.weight[1], 0.0..=1.0).text("G").step_by(0.01));
                    ui.add(Slider::new(&mut self.current.weight[2], 0.0..=1.0).text("B").step_by(0.01));
                    ui.separator();
                    ui.add(Slider::new(&mut self.current.scale, 0.1..=5.0).text("Scale").step_by(0.1));
                    ui.separator();
                    
                    if ui.button("Save").clicked() {
                        let img = self.masks[&self.current.key].generate(&self.current.weight);
                        let (w, h) = img.dimensions();
                        let nwidth = (w as f32 * self.current.scale) as u32;
                        let nheight = (h as f32 * self.current.scale) as u32;
                        if let Some(path) = FileDialog::new()
                            .add_filter("PNG", &["png"])
                            .set_file_name(img.export_name(&self.current.key, nwidth, nheight))
                            .set_title("Save the preview image")
                            .set_directory(std::env::current_dir().unwrap_or(PathBuf::new()))
                            .save_file()
                        {
                            if let Err(e) = img.save_as(path, nwidth, nheight, imageops::FilterType::Lanczos3) {
                                eprintln!("save failed: {e}");
                            } else {
                                println!("saved image.");
                            }
                        }
                    }
                });
            }
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                if let Some(tex) = &self.tex {
                    let max_size = ui.available_size().min_elem();
                    ui.image((tex.id(), egui::vec2(max_size, max_size)));
                } else {
                    ui.label("Loading...");
                }
            });
        });

        if changed {
            ctx.request_repaint();
        }
    }
}