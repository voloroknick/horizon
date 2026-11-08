//! Sub-levels and the set the game keeps per level.

use std::collections::{BTreeMap, BTreeSet};

use crate::assemble::{AssembleLimits, unsupported_after_change};
use crate::math::{Aabb, BlockPos, Vec3};
use crate::physics;
use crate::shape::merge_boxes;
use crate::world::{BlockState, BlockWorld};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubLevelId(pub u64);

/// A rigid group of blocks out of the grid.
///
/// Blocks are kept in local coordinates; `position` is where local `(0,0,0)`
/// is in the world. Stage 1 bodies don't rotate, so the pose is a translation.
#[derive(Clone, Debug)]
pub struct SubLevel {
    pub id: SubLevelId,
    pub blocks: BTreeMap<BlockPos, BlockState>,
    pub position: Vec3,
    pub velocity: Vec3,
    /// Collision boxes in local coordinates, rebuilt when blocks change.
    pub boxes: Vec<Aabb>,
    pub mass: f64,
    pub on_ground: bool,
    /// Ticks spent slow and on the ground.
    pub resting_ticks: u32,
}

impl SubLevel {
    fn new(
        id: SubLevelId,
        world_blocks: &BTreeMap<BlockPos, BlockState>,
        world: &impl BlockWorld,
    ) -> Self {
        let origin = *world_blocks.keys().next().expect("a sub-level has blocks");
        let blocks: BTreeMap<_, _> = world_blocks
            .iter()
            .map(|(p, s)| (p.offset(-origin.x, -origin.y, -origin.z), *s))
            .collect();
        let mass = blocks.values().map(|s| world.mass(*s)).sum();
        let mut sub = Self {
            id,
            blocks,
            position: origin.corner(),
            velocity: Vec3::ZERO,
            boxes: Vec::new(),
            mass,
            on_ground: false,
            resting_ticks: 0,
        };
        sub.rebuild_shape();
        sub
    }

    pub fn rebuild_shape(&mut self) {
        let cells: BTreeSet<_> = self.blocks.keys().copied().collect();
        self.boxes = merge_boxes(&cells);
    }

    pub fn world_boxes(&self) -> Vec<Aabb> {
        self.boxes.iter().map(|b| b.moved(self.position)).collect()
    }

    pub fn bounds(&self) -> Option<Aabb> {
        let mut it = self.world_boxes().into_iter();
        let first = it.next()?;
        Some(it.fold(first, |a, b| {
            Aabb::new(
                Vec3::new(
                    a.min.x.min(b.min.x),
                    a.min.y.min(b.min.y),
                    a.min.z.min(b.min.z),
                ),
                Vec3::new(
                    a.max.x.max(b.max.x),
                    a.max.y.max(b.max.y),
                    a.max.z.max(b.max.z),
                ),
            )
        }))
    }

    /// The whole grid position this body would land on.
    pub fn grid_origin(&self) -> BlockPos {
        BlockPos::new(
            self.position.x.round() as i32,
            self.position.y.round() as i32,
            self.position.z.round() as i32,
        )
    }

    pub fn is_asleep(&self) -> bool {
        self.resting_ticks >= SubLevels::SLEEP_AFTER_TICKS
    }
}

/// Blocks a sub-level put back into the grid, and the ones that had no room
/// (the game drops those as items).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Landing {
    pub id: Option<SubLevelId>,
    pub placed: Vec<(BlockPos, BlockState)>,
    pub dropped: Vec<(BlockPos, BlockState)>,
}

/// Every sub-level of one level.
#[derive(Debug, Default)]
pub struct SubLevels {
    pub bodies: BTreeMap<SubLevelId, SubLevel>,
    next_id: u64,
    pub limits: AssembleLimits,
}

impl SubLevels {
    /// One second of rest before a body lands back in the grid.
    pub const SLEEP_AFTER_TICKS: u32 = 20;

    pub fn new() -> Self {
        Self::default()
    }

    /// Call after `changed` was broken, replaced or hit. The groups that lost
    /// their support leave the grid and become sub-levels.
    pub fn wake(&mut self, world: &mut impl BlockWorld, changed: BlockPos) -> Vec<SubLevelId> {
        let groups = unsupported_after_change(world, changed, self.limits);
        let mut ids = Vec::new();
        for group in groups {
            let id = SubLevelId(self.next_id);
            self.next_id += 1;
            let body = SubLevel::new(id, &group.blocks, world);
            for pos in group.blocks.keys() {
                world.set_block(*pos, 0);
            }
            self.bodies.insert(id, body);
            ids.push(id);
        }
        ids
    }

    /// One game tick: moves every awake body and lands the ones that rested
    /// long enough. Returns what landed this tick.
    pub fn tick(&mut self, world: &mut impl BlockWorld) -> Vec<Landing> {
        let mut landed = Vec::new();
        for body in self.bodies.values_mut() {
            let boxes = body.world_boxes();
            let (moved, contacts) = physics::step(world, &boxes, &mut body.velocity);
            body.position += moved;
            body.on_ground = contacts.on_ground;
            if contacts.on_ground && body.velocity.length() < physics::SLEEP_SPEED {
                body.resting_ticks += 1;
            } else {
                body.resting_ticks = 0;
            }
            if body.is_asleep() {
                landed.push(body.id);
            }
        }
        landed
            .into_iter()
            .filter_map(|id| self.land(world, id))
            .collect()
    }

    /// Puts a body's blocks back into the grid at the nearest whole position.
    pub fn land(&mut self, world: &mut impl BlockWorld, id: SubLevelId) -> Option<Landing> {
        let body = self.bodies.remove(&id)?;
        let origin = body.grid_origin();
        let mut landing = Landing {
            id: Some(id),
            ..Landing::default()
        };
        for (local, state) in body.blocks {
            let pos = local.offset(origin.x, origin.y, origin.z);
            if world.block(pos) == 0 {
                world.set_block(pos, state);
                landing.placed.push((pos, state));
            } else {
                landing.dropped.push((pos, state));
            }
        }
        Some(landing)
    }

    /// Pushes a body, as a hit or an explosion would. Wakes it up.
    pub fn push(&mut self, id: SubLevelId, impulse: Vec3) {
        if let Some(body) = self.bodies.get_mut(&id) {
            body.velocity += impulse * (1000.0 / body.mass.max(1.0));
            body.resting_ticks = 0;
        }
    }
}
