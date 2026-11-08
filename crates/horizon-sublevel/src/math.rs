//! The little vector maths sub-levels need, without a dependency.

use std::ops::{Add, AddAssign, Mul, Neg, Sub};

/// A block position in the world grid, or in a sub-level's own grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub const fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }

    /// The six face neighbours, below first: support search walks down first.
    pub fn neighbours(self) -> [BlockPos; 6] {
        [
            self.offset(0, -1, 0),
            self.offset(0, 1, 0),
            self.offset(-1, 0, 0),
            self.offset(1, 0, 0),
            self.offset(0, 0, -1),
            self.offset(0, 0, 1),
        ]
    }

    pub fn corner(self) -> Vec3 {
        Vec3::new(self.x as f64, self.y as f64, self.z as f64)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, o: Vec3) {
        *self = *self + o;
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, k: f64) -> Vec3 {
        Vec3::new(self.x * k, self.y * k, self.z * k)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

/// An axis-aligned box, min inclusive and max exclusive, in blocks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn of_block(pos: BlockPos) -> Self {
        let min = pos.corner();
        Self::new(min, min + Vec3::new(1.0, 1.0, 1.0))
    }

    pub fn moved(self, by: Vec3) -> Self {
        Self::new(self.min + by, self.max + by)
    }

    pub fn volume(self) -> f64 {
        (self.max.x - self.min.x) * (self.max.y - self.min.y) * (self.max.z - self.min.z)
    }

    /// Grows the box towards `by`, the space a move by `by` sweeps.
    pub fn expand_towards(self, by: Vec3) -> Self {
        let mut b = self;
        if by.x < 0.0 {
            b.min.x += by.x
        } else {
            b.max.x += by.x
        }
        if by.y < 0.0 {
            b.min.y += by.y
        } else {
            b.max.y += by.y
        }
        if by.z < 0.0 {
            b.min.z += by.z
        } else {
            b.max.z += by.z
        }
        b
    }

    /// The block cells this box touches.
    pub fn cells(self) -> impl Iterator<Item = BlockPos> {
        const EPS: f64 = 1e-7;
        let (x0, x1) = (
            (self.min.x + EPS).floor() as i32,
            (self.max.x - EPS).floor() as i32,
        );
        let (y0, y1) = (
            (self.min.y + EPS).floor() as i32,
            (self.max.y - EPS).floor() as i32,
        );
        let (z0, z1) = (
            (self.min.z + EPS).floor() as i32,
            (self.max.z - EPS).floor() as i32,
        );
        (x0..=x1).flat_map(move |x| {
            (y0..=y1).flat_map(move |y| (z0..=z1).map(move |z| BlockPos::new(x, y, z)))
        })
    }

    /// Vanilla's `Shapes.collide` for one axis: how far `self` may move by
    /// `delta` along `axis` before it touches `other`.
    pub fn clip(self, other: Aabb, axis: usize, delta: f64) -> f64 {
        let (a_min, a_max, b_min, b_max) = (self.min, self.max, other.min, other.max);
        let get = |v: Vec3, i: usize| [v.x, v.y, v.z][i];
        for i in 0..3 {
            if i != axis && (get(a_max, i) <= get(b_min, i) || get(a_min, i) >= get(b_max, i)) {
                return delta;
            }
        }
        if delta > 0.0 && get(a_max, axis) <= get(b_min, axis) {
            delta.min(get(b_min, axis) - get(a_max, axis))
        } else if delta < 0.0 && get(a_min, axis) >= get(b_max, axis) {
            delta.max(get(b_max, axis) - get(a_min, axis))
        } else {
            delta
        }
    }
}
