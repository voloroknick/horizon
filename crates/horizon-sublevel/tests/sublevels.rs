use std::collections::{BTreeSet, HashMap};

use horizon_sublevel::{Aabb, BlockPos, BlockState, BlockWorld, SubLevels, Vec3, merge_boxes};

const STONE: BlockState = 1;
const PLANKS: BlockState = 2;

/// A world with an unbreakable floor below y = 0.
#[derive(Default)]
struct MapWorld {
    blocks: HashMap<BlockPos, BlockState>,
}

impl MapWorld {
    fn fill(&mut self, from: BlockPos, to: BlockPos, state: BlockState) {
        for x in from.x..=to.x {
            for y in from.y..=to.y {
                for z in from.z..=to.z {
                    self.blocks.insert(BlockPos::new(x, y, z), state);
                }
            }
        }
    }
}

impl BlockWorld for MapWorld {
    fn block(&self, pos: BlockPos) -> BlockState {
        if pos.y < 0 {
            STONE
        } else {
            self.blocks.get(&pos).copied().unwrap_or(0)
        }
    }
    fn is_anchor(&self, pos: BlockPos) -> bool {
        pos.y < 0
    }
    fn set_block(&mut self, pos: BlockPos, state: BlockState) {
        if state == 0 {
            self.blocks.remove(&pos);
        } else {
            self.blocks.insert(pos, state);
        }
    }
}

fn run_until_landed(world: &mut MapWorld, subs: &mut SubLevels, max_ticks: u32) -> u32 {
    for t in 1..=max_ticks {
        subs.tick(world);
        if subs.bodies.is_empty() {
            return t;
        }
    }
    panic!("bodies still moving after {max_ticks} ticks");
}

#[test]
fn a_platform_falls_when_its_pillar_is_broken_and_lands_on_the_floor() {
    let mut world = MapWorld::default();
    // A pillar 0..=9 holding a 3x1x3 platform at y = 10.
    world.fill(BlockPos::new(0, 0, 0), BlockPos::new(0, 9, 0), STONE);
    world.fill(BlockPos::new(-1, 10, -1), BlockPos::new(1, 10, 1), PLANKS);
    let mut subs = SubLevels::new();

    // Break the pillar's top block: the platform hangs in the air.
    world.set_block(BlockPos::new(0, 9, 0), 0);
    let woke = subs.wake(&mut world, BlockPos::new(0, 9, 0));
    assert_eq!(
        woke.len(),
        1,
        "only the platform comes loose; the rest of the pillar stands"
    );
    assert_eq!(
        world.block(BlockPos::new(0, 10, 0)),
        0,
        "the platform left the grid"
    );

    run_until_landed(&mut world, &mut subs, 400);
    // It lands on the pillar's new top (y = 8), so the platform is at y = 9.
    for x in -1..=1 {
        for z in -1..=1 {
            assert_eq!(
                world.block(BlockPos::new(x, 9, z)),
                PLANKS,
                "plank at {x},9,{z}"
            );
            assert_eq!(world.block(BlockPos::new(x, 10, z)), 0);
        }
    }
}

#[test]
fn breaking_a_block_of_a_standing_tower_wakes_nothing() {
    let mut world = MapWorld::default();
    world.fill(BlockPos::new(0, 0, 0), BlockPos::new(2, 5, 2), STONE);
    let mut subs = SubLevels::new();
    world.set_block(BlockPos::new(1, 5, 1), 0);
    assert!(subs.wake(&mut world, BlockPos::new(1, 5, 1)).is_empty());
    assert_eq!(world.blocks.len(), 3 * 6 * 3 - 1);
}

#[test]
fn a_fast_body_does_not_pass_through_a_one_block_floor() {
    let mut world = MapWorld::default();
    world.fill(BlockPos::new(-5, 0, -5), BlockPos::new(5, 0, 5), STONE);
    // A block high above, held by a single pillar block we then remove.
    world.set_block(BlockPos::new(0, 200, 0), PLANKS);
    world.set_block(BlockPos::new(0, 199, 0), STONE);
    let mut subs = SubLevels::new();
    world.set_block(BlockPos::new(0, 199, 0), 0);
    let ids = subs.wake(&mut world, BlockPos::new(0, 199, 0));
    assert_eq!(ids.len(), 1);
    // Give it a huge downward speed: many blocks per tick.
    subs.push(ids[0], Vec3::new(0.0, -50.0, 0.0));
    run_until_landed(&mut world, &mut subs, 600);
    assert_eq!(world.block(BlockPos::new(0, 1, 0)), PLANKS);
}

#[test]
fn a_body_landing_on_a_taken_cell_drops_that_block_instead_of_replacing_it() {
    let mut world = MapWorld::default();
    world.set_block(BlockPos::new(0, 5, 0), PLANKS);
    let mut subs = SubLevels::new();
    let ids = subs.wake(&mut world, BlockPos::new(0, 5, 0));
    assert_eq!(ids.len(), 1);
    // Something is built where it will snap to while it is in the air.
    subs.tick(&mut world);
    let body = &subs.bodies[&ids[0]];
    let target = body.grid_origin();
    world.set_block(target, STONE);
    let landing = subs.land(&mut world, ids[0]).unwrap();
    assert_eq!(landing.dropped, vec![(target, PLANKS)]);
    assert_eq!(world.block(target), STONE);
}

#[test]
fn merged_boxes_cover_every_cell_exactly_once() {
    let mut cells = BTreeSet::new();
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                cells.insert(BlockPos::new(x, y, z));
            }
        }
    }
    assert_eq!(merge_boxes(&cells).len(), 1, "a solid cube is one box");

    // An L-shaped wall with a hole: boxes must cover exactly these cells.
    let mut cells = BTreeSet::new();
    for x in 0..6 {
        for y in 0..4 {
            cells.insert(BlockPos::new(x, y, 0));
        }
    }
    for z in 1..5 {
        for y in 0..4 {
            cells.insert(BlockPos::new(0, y, z));
        }
    }
    cells.remove(&BlockPos::new(3, 1, 0));
    let boxes = merge_boxes(&cells);
    let volume: f64 = boxes.iter().map(|b: &Aabb| b.volume()).sum();
    assert_eq!(volume as usize, cells.len());
    let mut covered = BTreeSet::new();
    for b in &boxes {
        for c in b.cells() {
            assert!(cells.contains(&c), "box covers empty cell {c:?}");
            assert!(covered.insert(c), "cell {c:?} in two boxes");
        }
    }
    assert!(
        boxes.len() < cells.len() / 4,
        "{} boxes for {} cells",
        boxes.len(),
        cells.len()
    );
}
