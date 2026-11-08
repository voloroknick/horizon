//! Stage 1 physics: gravity, drag and collision against the grid, translation
//! only. Collision clips the move one axis at a time against every solid cell
//! the move sweeps, as vanilla does for entities, so a body never ends inside
//! a block however fast it falls.

use crate::math::{Aabb, Vec3};
use crate::world::BlockWorld;

/// Game ticks per second.
pub const TICKS_PER_SECOND: f64 = 20.0;
/// Blocks per tick², vanilla's falling block gravity.
pub const GRAVITY: f64 = 0.04;
/// Vanilla's falling block drag per tick.
pub const DRAG: f64 = 0.98;
/// A body faster than this, in blocks per tick, is never asleep.
pub const SLEEP_SPEED: f64 = 0.005;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Contacts {
    pub on_ground: bool,
    pub hit_x: bool,
    pub hit_y: bool,
    pub hit_z: bool,
}

/// Moves `boxes` (in world space) by `wanted`, stopping at solid cells.
/// Returns the move made and what was hit.
pub fn collide(world: &impl BlockWorld, boxes: &[Aabb], wanted: Vec3) -> (Vec3, Contacts) {
    let mut moved = Vec3::ZERO;
    let mut contacts = Contacts::default();
    // Vertical first, as vanilla does when the vertical move is the largest.
    for axis in [1, 0, 2] {
        let mut d = [wanted.x, wanted.y, wanted.z][axis];
        if d == 0.0 {
            continue;
        }
        for b in boxes {
            let here = b.moved(moved);
            let mut step = Vec3::ZERO;
            match axis {
                0 => step.x = d,
                1 => step.y = d,
                _ => step.z = d,
            }
            for cell in here.expand_towards(step).cells() {
                if world.is_solid(cell) {
                    d = here.clip(Aabb::of_block(cell), axis, d);
                }
            }
        }
        let original = [wanted.x, wanted.y, wanted.z][axis];
        let blocked = (d - original).abs() > 1e-9;
        match axis {
            0 => {
                moved.x = d;
                contacts.hit_x = blocked;
            }
            1 => {
                moved.y = d;
                contacts.hit_y = blocked;
                contacts.on_ground = blocked && original < 0.0;
            }
            _ => {
                moved.z = d;
                contacts.hit_z = blocked;
            }
        }
    }
    (moved, contacts)
}

/// One tick of a body: gravity, the move, drag, and stopping on contact.
pub fn step(
    world: &impl BlockWorld,
    boxes_world: &[Aabb],
    velocity: &mut Vec3,
) -> (Vec3, Contacts) {
    velocity.y -= GRAVITY;
    let (moved, contacts) = collide(world, boxes_world, *velocity);
    if contacts.hit_x {
        velocity.x = 0.0;
    }
    if contacts.hit_y {
        velocity.y = 0.0;
    }
    if contacts.hit_z {
        velocity.z = 0.0;
    }
    *velocity = *velocity * DRAG;
    if contacts.on_ground {
        // Ground friction, as a falling block's 0.7 on landing.
        velocity.x *= 0.7;
        velocity.z *= 0.7;
    }
    (moved, contacts)
}
