//! What a sub-level needs from the world. The game implements this over its
//! levels; tests implement it over a map.

use crate::math::BlockPos;

/// A block state as the game numbers them. `0` is air.
pub type BlockState = u32;

pub trait BlockWorld {
    /// The state at `pos`, `0` for air or an unloaded cell.
    fn block(&self, pos: BlockPos) -> BlockState;

    /// Whether the cell stops a moving sub-level. Full cubes only in stage 1;
    /// partial shapes come from the game's collision shapes later.
    fn is_solid(&self, pos: BlockPos) -> bool {
        self.block(pos) != 0
    }

    /// Blocks that hold up whatever touches them and never come loose:
    /// bedrock, the world's floor, unloaded chunks.
    fn is_anchor(&self, pos: BlockPos) -> bool;

    /// Whether this block may leave the grid. The game refuses block entities
    /// and other blocks it cannot carry yet.
    fn can_move(&self, pos: BlockPos) -> bool {
        self.block(pos) != 0 && !self.is_anchor(pos)
    }

    /// Mass of one block in kilograms, for later stages.
    fn mass(&self, _state: BlockState) -> f64 {
        1000.0
    }

    fn set_block(&mut self, pos: BlockPos, state: BlockState);
}
