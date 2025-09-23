/// RGBA color stored as `[R, G, B, A]` in **0.0~1.0**
pub type Color = [f32; 4];

pub fn weighted_average(weight: &[f32; 3], value: &[f32; 3]) -> f32 {
    let sum = weight[0] + weight[1] + weight[2];
    if sum == 0.0 {
        return 0f32;
    } else {
        return (weight[0]*value[0] + weight[1]*value[1] + weight[2]*value[2]) / (sum)
    }
}

/// Mix a single RGBA pixel by 3-channel weight and 3 mask pixels.
/// 
/// Alpha channel is **preserved**; only RGB components are modified.
/// For each channel `i in [0, 1, 2]`:
/// 1. Extract channel values from the 3 masks into a temporary vector
/// 2. Compute weighted average against `weight`
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
        pixel[i] = weighted_average(weight, &[mask[0][i], mask[1][i], mask[2][i]]);
    }
}