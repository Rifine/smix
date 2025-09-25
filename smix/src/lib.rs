use std::path::Path;

use image::{imageops, open, Rgba, Rgba32FImage, RgbaImage};

/// RGBA color stored as `[R, G, B, A]` in **0.0~1.0**
pub type Color = [f32; 4];

pub fn apply_weight(weight: &[f32; 3], value: &[f32; 3]) -> f32 {
    return weight[0]*value[0] + weight[1]*value[1] + weight[2]*value[2]
}

/// Mix a single RGBA pixel by 3-channel weight and 3 mask pixels.
/// 
/// Alpha channel is **preserved**; only RGB components are modified.
/// For each channel `i in [0, 1, 2]`:
/// 1. Extract channel values from the 3 masks into a temporary vector
/// 2. Compute `pixel[i]` against `weight`
/// 3. Store result back into `pixel[i]`
/// 
/// # Arguments
/// * `pixel` - In-out RGBA pixel (alpha untouched)
/// * `weight` - Per-channel weights `[Rw, Gw, Bw]` (sum != 0)
/// * `mask` - Exactly 3 RGBA samples (alpha ignored) corresponding to R, G, B masks
/// 
/// # Exmaples
/// ```
/// use smix::mix_pixel;
/// 
/// let mut px = [0.0, 0.0, 0.0, 1.0];
/// let w = [0.8, 0.15, 0.05];
/// let m = [
///     [1.0, 0.0, 0.0, 1.0], // red
///     [0.0, 1.0, 0.0, 1.0], // red
///     [0.0, 0.0, 1.0, 1.0], // red
/// ];
/// mix_pixel(&mut px, &w, &m);
/// assert_eq!(px, [0.8, 0.15, 0.05, 1.0]);
/// ```
pub fn mix_pixel(pixel: &mut Color, weight: &[f32; 3], mask: &[Color; 3]) {
    for i in 0..3 {
        pixel[i] = apply_weight(weight, &[mask[0][i], mask[1][i], mask[2][i]]);
    }
}

pub fn f32img_to_u8img(src: &Rgba32FImage) -> RgbaImage {
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

pub struct Mask {
    images: [Rgba32FImage; 3],
    width: u32,
    height: u32,
}

impl Mask {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let images = [
            open(path.join("r.png"))?.into_rgba32f(),
            open(path.join("g.png"))?.into_rgba32f(),
            open(path.join("b.png"))?.into_rgba32f()
        ];
        let dimensions = images[0].dimensions();
        if dimensions == images[1].dimensions() && dimensions == images[2].dimensions() {
            let (width, height) = dimensions;
            return Ok(Self {
                images,
                width,
                height
            });
        }
        Err(anyhow::anyhow!("Masks have different demensions!"))
    }

    pub fn generate(&self, weight: &[f32; 3]) -> GeneratedImage {
        let mut image = Rgba32FImage::new(self.width, self.height);
        for (x, y, p) in image.enumerate_pixels_mut() {
            let alpha = self.images[0].get_pixel(x, y).0[3];
            p.0[3] = if alpha == 0.0 { continue } else { alpha };
            let mask = [
                self.images[0].get_pixel(x, y).0,
                self.images[1].get_pixel(x, y).0,
                self.images[2].get_pixel(x, y).0,
            ];
            mix_pixel(&mut p.0, &weight, &mask);
        }
        GeneratedImage::new(image)
    }
}

pub struct GeneratedImage {
    img32f: Rgba32FImage,
    img: RgbaImage,
}

impl GeneratedImage {
    pub fn new(img: Rgba32FImage) -> Self {
        Self {
            img: f32img_to_u8img(&img),
            img32f: img,
        }
    }

    pub fn export_name(&self, basename: &String, width: u32, height: u32) -> String {
        format!("{basename}_{width}x{height}.png")
    }

    pub fn get_rgba(&self) -> &RgbaImage {
        &self.img
    }

    pub fn get_rgba_mut(&mut self) -> &mut RgbaImage {
        &mut self.img
    }

    pub fn get_rgba32f(&self) -> &Rgba32FImage {
        &self.img32f
    }

    pub fn get_rgba32f_mut(&mut self) -> &mut Rgba32FImage {
        &mut self.img32f
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.img.dimensions()
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        self.img.save(path)?;
        Ok(())
    }

    pub fn save_as<P: AsRef<Path>>(&self, path: P, nwidth: u32, nheight: u32, filter: imageops::FilterType) -> anyhow::Result<()> {
        imageops::resize(&self.img, nwidth, nheight, filter).save(path)?;
        Ok(())
    }
}