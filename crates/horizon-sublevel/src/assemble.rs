//! Finding the blocks that lost their support.

use std::collections::{BTreeMap, HashSet, VecDeque};

use crate::math::BlockPos;
use crate::world::{BlockState, BlockWorld};

#[derive(Clone, Copy, Debug)]
pub struct AssembleLimits {
    /// A group bigger than this counts as supported: the search gives up
    /// instead of tearing a mountain out of the ground.
    pub max_blocks: usize,
    /// How far, in blocks, the search may wander from where it started.
    pub max_radius: i32,
}

impl Default for AssembleLimits {
    fn default() -> Self {
        Self {
            max_blocks: 4096,
            max_radius: 48,
        }
    }
}

/// A group of blocks that no longer touches an anchor.
#[derive(Clone, Debug, PartialEq)]
pub struct Unsupported {
    pub blocks: BTreeMap<BlockPos, BlockState>,
}

/// Starting at `start`, collects the face-connected movable blocks. Returns
/// `None` when the group reaches an anchor, a block that cannot move, or a
/// limit: then it stays in the grid.
pub fn find_unsupported(
    world: &impl BlockWorld,
    start: BlockPos,
    limits: AssembleLimits,
) -> Option<Unsupported> {
    if !world.can_move(start) {
        return None;
    }
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    let mut blocks = BTreeMap::new();
    while let Some(pos) = queue.pop_front() {
        blocks.insert(pos, world.block(pos));
        if blocks.len() > limits.max_blocks {
            return None;
        }
        for next in pos.neighbours() {
            if !seen.insert(next) {
                continue;
            }
            if world.is_anchor(next) {
                return None;
            }
            if world.block(next) == 0 {
                continue;
            }
            let far = (next.x - start.x)
                .abs()
                .max((next.y - start.y).abs())
                .max((next.z - start.z).abs());
            if !world.can_move(next) || far > limits.max_radius {
                return None;
            }
            queue.push_back(next);
        }
    }
    Some(Unsupported { blocks })
}

/// After `changed` was broken or replaced, the groups around it that now hang
/// in the air. Each neighbour is searched once; groups are not repeated.
pub fn unsupported_after_change(
    world: &impl BlockWorld,
    changed: BlockPos,
    limits: AssembleLimits,
) -> Vec<Unsupported> {
    let mut found: Vec<Unsupported> = Vec::new();
    let mut starts = vec![changed];
    starts.extend(changed.neighbours());
    for start in starts {
        if found.iter().any(|g| g.blocks.contains_key(&start)) {
            continue;
        }
        if let Some(group) = find_unsupported(world, start, limits) {
            found.push(group);
        }
    }
    found
}
