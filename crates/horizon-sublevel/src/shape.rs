//! Turning a sub-level's blocks into a few boxes for collision.
//!
//! Testing a thousand cubes against the world every tick is what makes moving
//! structures slow. Merging them first leaves a handful of boxes for a solid
//! build: a 10x10x10 cube becomes one box.

use std::collections::BTreeSet;

use crate::math::{Aabb, BlockPos, Vec3};

/// Greedy merge: grow each box along x, then z, then y, while every cell it
/// would cover is filled and not taken. Every cell ends in exactly one box.
pub fn merge_boxes(cells: &BTreeSet<BlockPos>) -> Vec<Aabb> {
    let mut left = cells.clone();
    let mut boxes = Vec::new();
    while let Some(&start) = left.iter().next() {
        let filled = |p: BlockPos, left: &BTreeSet<BlockPos>| left.contains(&p);
        let mut dx = 1;
        while filled(start.offset(dx, 0, 0), &left) {
            dx += 1;
        }
        let mut dz = 1;
        'z: loop {
            for x in 0..dx {
                if !filled(start.offset(x, 0, dz), &left) {
                    break 'z;
                }
            }
            dz += 1;
        }
        let mut dy = 1;
        'y: loop {
            for x in 0..dx {
                for z in 0..dz {
                    if !filled(start.offset(x, dy, z), &left) {
                        break 'y;
                    }
                }
            }
            dy += 1;
        }
        for x in 0..dx {
            for y in 0..dy {
                for z in 0..dz {
                    left.remove(&start.offset(x, y, z));
                }
            }
        }
        let min = start.corner();
        boxes.push(Aabb::new(
            min,
            min + Vec3::new(dx as f64, dy as f64, dz as f64),
        ));
    }
    boxes
}
