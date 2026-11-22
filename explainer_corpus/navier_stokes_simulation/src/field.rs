//! Decode validated solver snapshots and rasterize a scalar field.
//! Numerical solvers never run during scene construction.

use std::fs;
use std::io::{self, Cursor, Read};
use std::path::Path;

#[derive(Clone, Copy)]
struct Leaf {
    level: u32,
    i: u32,
    j: u32,
    wet: f64,
    velocity: [f64; 2],
    vorticity: f64,
}

#[derive(Clone, Copy)]
pub struct TunnelGeometry {
    pub length: f64,
    pub height: f64,
    pub cylinder: [f64; 3],
}

pub struct Frame {
    pub time: f64,
    pub wet_leaves: usize,
    pub level_min: u32,
    pub level_max: u32,
    length: f64,
    height: f64,
    cylinder: [f64; 3],
    dx: f64,
    nx: usize,
    ny: usize,
    leaves: Vec<Leaf>,
    owner: Vec<usize>,
    reconstructed: Vec<Option<f64>>,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn uint(reader: &mut impl Read) -> io::Result<u32> {
    let mut bytes = [0; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn real(reader: &mut impl Read) -> io::Result<f64> {
    let mut bytes = [0; 8];
    reader.read_exact(&mut bytes)?;
    let value = f64::from_le_bytes(bytes);
    if !value.is_finite() {
        return Err(invalid("Nonfinite solver field"));
    }
    Ok(value)
}

fn minmod(a: f64, b: f64) -> f64 {
    if a * b <= 0.0 {
        0.0
    } else {
        a.signum() * a.abs().min(b.abs())
    }
}

impl Frame {
    pub fn load(path: &Path) -> io::Result<Self> {
        const HEADER_BYTES: u64 = 80;
        const LEAF_BYTES: u64 = 52;
        const FACE_BYTES: u64 = 32;
        let file_size = fs::metadata(path)?.len();
        if !(HEADER_BYTES..=128 * 1024 * 1024).contains(&file_size) {
            return Err(invalid("Unexpected solver snapshot size"));
        }
        let bytes = fs::read(path)?;
        let mut reader = Cursor::new(bytes);
        let mut magic = [0; 8];
        reader.read_exact(&mut magic)?;
        if &magic != b"NSFLOW01" || uint(&mut reader)? != 0x01020304 {
            return Err(invalid("Unexpected solver snapshot version or endian order"));
        }
        let _step = uint(&mut reader)?;
        let n_cells = uint(&mut reader)? as usize;
        let n_faces = uint(&mut reader)? as usize;
        if n_cells == 0
            || file_size != HEADER_BYTES + n_cells as u64 * LEAF_BYTES + n_faces as u64 * FACE_BYTES
        {
            return Err(invalid("Solver snapshot record counts do not match its bytes"));
        }
        let time = real(&mut reader)?;
        let box_width = real(&mut reader)?;
        let length = real(&mut reader)?;
        let height = real(&mut reader)?;
        let cylinder = [real(&mut reader)?, real(&mut reader)?, real(&mut reader)? / 2.0];
        let tunnel = box_width == 2.56 && length == 2.2 && height == 0.41
            && cylinder == [0.2, 0.2, 0.05];
        let shear = box_width == 1.0 && length == 1.0 && height == 1.0 && cylinder == [0.0; 3];
        if time < 0.0 || !(tunnel || shear) {
            return Err(invalid("Unexpected tunnel geometry or time"));
        }
        let levels = if tunnel { 8..=11 } else { 4..=7 };

        let mut leaves = Vec::with_capacity(n_cells);
        for _ in 0..n_cells {
            let level = uint(&mut reader)?;
            let i = uint(&mut reader)?;
            let j = uint(&mut reader)?;
            let wet = real(&mut reader)?;
            let velocity = [real(&mut reader)?, real(&mut reader)?];
            let _pressure = real(&mut reader)?;
            let vorticity = real(&mut reader)?;
            if !levels.contains(&level) || !(0.0..=1.0).contains(&wet) {
                return Err(invalid("Invalid leaf fraction or level"));
            }
            leaves.push(Leaf { level, i, j, wet, velocity, vorticity });
        }
        for _ in 0..n_faces {
            let axis = uint(&mut reader)?;
            let level = uint(&mut reader)?;
            let i = uint(&mut reader)?;
            let j = uint(&mut reader)?;
            let open = real(&mut reader)?;
            let flux = real(&mut reader)?;
            if axis > 1 || !levels.contains(&level)
                || i > 1 << level || j > 1 << level || !(0.0..=1.0).contains(&open)
                || (open == 0.0 && flux.abs() > 1e-12)
            {
                return Err(invalid("Invalid solver face coordinates or flux"));
            }
        }
        let level_min = leaves.iter().map(|cell| cell.level).min().unwrap();
        let level_max = leaves.iter().map(|cell| cell.level).max().unwrap();
        let dx = box_width / (1 << level_max) as f64;
        let nx = (length / dx).round() as usize;
        let ny = (height / dx).round() as usize;
        let mut owner = vec![usize::MAX; nx * ny];
        for (index, cell) in leaves.iter().enumerate() {
            let span = 1usize << (level_max - cell.level);
            let x0 = cell.i as usize * span;
            let y0 = cell.j as usize * span;
            if x0 + span > nx || y0 + span > ny {
                return Err(invalid("Leaf lies outside the physical channel"));
            }
            for y in y0..y0 + span {
                for occupied in &mut owner[y * nx + x0..y * nx + x0 + span] {
                    if *occupied != usize::MAX {
                        return Err(invalid("Overlapping quadtree leaves"));
                    }
                    *occupied = index;
                }
            }
        }
        if owner.contains(&usize::MAX) {
            return Err(invalid("The saved mesh leaves a hole in the physical channel"));
        }
        let wet_leaves = leaves.iter().filter(|cell| cell.wet > 0.0).count();
        let mut frame = Self {
            time, wet_leaves, level_min, level_max, length, height, cylinder, dx, nx, ny,
            leaves, owner, reconstructed: vec![None; nx * ny],
        };
        frame.reconstruct();
        Ok(frame)
    }

    pub fn is_tunnel(&self) -> bool {
        self.cylinder[2] > 0.0
    }

    pub fn tunnel_geometry(&self) -> Option<TunnelGeometry> {
        self.is_tunnel().then_some(TunnelGeometry {
            length: self.length,
            height: self.height,
            cylinder: self.cylinder,
        })
    }

    /// The velocity stored at the center of the leaf containing this point.
    /// This is a discrete sample, with no interpolation or tracer integration.
    pub fn cell_velocity_at(&self, x: f64, y: f64) -> Option<[f64; 2]> {
        if !(0.0..self.length).contains(&x) || !(0.0..self.height).contains(&y) {
            return None;
        }
        if self.is_tunnel()
            && (x - self.cylinder[0]).hypot(y - self.cylinder[1]) <= self.cylinder[2]
        {
            return None;
        }
        let index = self.index((x / self.dx) as isize, (y / self.dx) as isize);
        let cell = self.leaves[self.owner[index]];
        (cell.wet > 0.0).then_some(cell.velocity)
    }

    pub fn height_over_width(&self) -> f64 {
        self.height / self.length
    }

    fn index(&self, i: isize, j: isize) -> usize {
        j.clamp(0, self.ny as isize - 1) as usize * self.nx
            + i.clamp(0, self.nx as isize - 1) as usize
    }

    fn reconstruct(&mut self) {
        // Limited slopes estimated from the saved neighboring leaf centers.
        // Sampling this raster later blends scalar values before coloring.
        for leaf in &self.leaves {
            if leaf.wet == 0.0 {
                continue;
            }
            let size = 1isize << (self.level_max - leaf.level);
            let (i0, j0) = (leaf.i as isize * size, leaf.j as isize * size);
            let (i1, j1) = (i0 + size, j0 + size);
            let center = |index: isize| index as f64 + size as f64 / 2.0;
            let middle = (i0 + size / 2, j0 + size / 2);
            let slope = |a, b, axis: usize| {
                let a: &Leaf = &self.leaves[self.owner[a]];
                let b: &Leaf = &self.leaves[self.owner[b]];
                if a.wet == 0.0 || b.wet == 0.0 {
                    return 0.0;
                }
                let other_center = |other: &Leaf| {
                    let coordinate = if axis == 0 { other.i } else { other.j };
                    (f64::from(coordinate) + 0.5) * (1 << (self.level_max - other.level)) as f64
                };
                let axis_center = [center(i0), center(j0)][axis];
                let h1 = axis_center - other_center(a);
                let h2 = other_center(b) - axis_center;
                if h1 <= 0.0 || h2 <= 0.0 {
                    0.0
                } else {
                    minmod((leaf.vorticity - a.vorticity) / h1, (b.vorticity - leaf.vorticity) / h2)
                }
            };
            let gx = slope(self.index(i0 - 1, middle.1), self.index(i1, middle.1), 0);
            let gy = slope(self.index(middle.0, j0 - 1), self.index(middle.0, j1), 1);
            for j in j0..j1 {
                for i in i0..i1 {
                    let index = self.index(i, j);
                    self.reconstructed[index] = Some(leaf.vorticity
                        + gx * (i as f64 + 0.5 - center(i0))
                        + gy * (j as f64 + 0.5 - center(j0)));
                }
            }
        }
    }

    fn sample(&self, x: f64, y: f64) -> Option<f64> {
        let gx = x / self.dx - 0.5;
        let gy = y / self.dx - 0.5;
        let (i, j) = (gx.floor() as isize, gy.floor() as isize);
        let (s, t) = (gx - i as f64, gy - j as f64);
        let mut total = 0.0;
        let mut weight = 0.0;
        for (di, wi) in [(0, 1.0 - s), (1, s)] {
            for (dj, wj) in [(0, 1.0 - t), (1, t)] {
                if let Some(value) = self.reconstructed[self.index(i + di, j + dj)] {
                    total += wi * wj * value;
                    weight += wi * wj;
                }
            }
        }
        (weight > 0.0).then_some(total / weight)
    }

    pub fn raster(
        &self,
        width: u32,
        height: u32,
        mesh: bool,
        next: Option<(&Self, f64)>,
    ) -> Vec<u8> {
        let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
        let px = self.length / f64::from(width);
        let py = self.height / f64::from(height);
        let color_limit = if self.is_tunnel() { 50.0 } else { 6.0 };
        for row in 0..height {
            let y = self.height - (f64::from(row) + 0.5) * py;
            for col in 0..width {
                let x = (f64::from(col) + 0.5) * px;
                let radius = (x - self.cylinder[0]).hypot(y - self.cylinder[1]);
                let mut color = if let Some(mut w) = self.sample(x, y) {
                    if let Some((next_frame, fraction)) = next
                        && let Some(next_w) = next_frame.sample(x, y)
                    {
                        w += fraction * (next_w - w);
                    }
                    let end = if w < 0.0 { [68, 180, 253] } else { [255, 122, 71] };
                    blend([18, 29, 47], end, (w.abs() / color_limit).min(1.0).sqrt())
                } else {
                    [18, 29, 47]
                };
                if mesh {
                    let index = self.index((x / self.dx) as isize, (y / self.dx) as isize);
                    let cell = self.leaves[self.owner[index]];
                    let h = self.dx * (1u32 << (self.level_max - cell.level)) as f64;
                    let ex = x / h - f64::from(cell.i);
                    let ey = y / h - f64::from(cell.j);
                    let distance = (ex.min(1.0 - ex) * h / px)
                        .min(ey.min(1.0 - ey) * h / py);
                    color = blend(color, [217, 239, 246], 0.34 * (1.0 - distance).clamp(0.0, 1.0));
                }
                if self.is_tunnel() {
                    let rim = radius - self.cylinder[2];
                    color = blend(color, [165, 191, 209], (1.5 - rim.abs() / px).clamp(0.0, 1.0));
                    if rim < -px {
                        color = [8, 15, 26];
                    }
                }
                pixels.extend([color[0], color[1], color[2], 255]);
            }
        }
        pixels
    }
}

fn blend(a: [u8; 3], b: [u8; 3], fraction: f64) -> [u8; 3] {
    std::array::from_fn(|i| {
        (f64::from(a[i]) * (1.0 - fraction) + f64::from(b[i]) * fraction).round() as u8
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn velocity_samples_use_the_owning_wet_cell_and_respect_the_cylinder() {
        let frame = Frame {
            time: 0.25,
            wet_leaves: 1,
            level_min: 0,
            level_max: 0,
            length: 2.0,
            height: 1.0,
            cylinder: [0.5, 0.5, 0.2],
            dx: 1.0,
            nx: 2,
            ny: 1,
            leaves: vec![
                Leaf { level: 0, i: 0, j: 0, wet: 0.8, velocity: [0.7, -0.1], vorticity: -0.5 },
                Leaf { level: 0, i: 1, j: 0, wet: 0.0, velocity: [1.1, 0.2], vorticity: 0.3 },
            ],
            owner: vec![0, 1],
            reconstructed: vec![None; 2],
        };
        assert_eq!(frame.cell_velocity_at(0.2, 0.25), Some([0.7, -0.1]));
        assert_eq!(frame.cell_velocity_at(0.5, 0.5), None);
        assert_eq!(frame.cell_velocity_at(1.1, 0.5), None);
        assert_eq!(frame.cell_velocity_at(2.0, 0.5), None);
        assert_eq!(frame.cell_velocity_at(f64::NAN, 0.5), None);
    }
}
