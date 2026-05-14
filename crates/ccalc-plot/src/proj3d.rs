//! Orthographic 3D projection for `plot3` / `scatter3`.
//!
//! Projects `(x, y, z)` triples onto the 2D plane using MATLAB-compatible
//! default view angles: azimuth = −37.5°, elevation = 30°.
//!
//! The projected `(x', y')` pairs can be passed directly to the ASCII
//! or file backends as if they were 2D data.

/// Default azimuth angle in degrees (MATLAB convention).
pub const DEFAULT_AZ: f64 = -37.5;
/// Default elevation angle in degrees (MATLAB convention).
pub const DEFAULT_EL: f64 = 30.0;

/// Projects 3D points onto a 2D plane using orthographic projection.
///
/// Uses the MATLAB default view (azimuth = −37.5°, elevation = 30°).
/// All three slices must have the same length.
///
/// # Examples
///
/// ```
/// use ccalc_plot::proj3d::project_ortho;
/// let x = [1.0, 0.0, -1.0];
/// let y = [0.0, 1.0,  0.0];
/// let z = [0.0, 0.0,  1.0];
/// let (px, py) = project_ortho(&x, &y, &z);
/// assert_eq!(px.len(), 3);
/// assert_eq!(py.len(), 3);
/// ```
pub fn project_ortho(x: &[f64], y: &[f64], z: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let az = DEFAULT_AZ.to_radians();
    let el = DEFAULT_EL.to_radians();
    let (sin_az, cos_az) = (az.sin(), az.cos());
    let (sin_el, cos_el) = (el.sin(), el.cos());

    let px: Vec<f64> = x
        .iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| -xi * sin_az + yi * cos_az)
        .collect();

    let py: Vec<f64> = x
        .iter()
        .zip(y.iter())
        .zip(z.iter())
        .map(|((&xi, &yi), &zi)| xi * cos_az * sin_el + yi * sin_az * sin_el + zi * cos_el)
        .collect();

    (px, py)
}
