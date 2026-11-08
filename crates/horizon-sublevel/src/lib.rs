//! Sub-levels: regions of blocks that leave the world grid, move with physics
//! and land back on it.
//!
//! The world stays an ordinary block grid. When a player or the game disturbs a
//! block, [`SubLevels::wake`] looks for the blocks that lost their support. They
//! are taken out of the grid and become one [`SubLevel`]: a rigid body that
//! carries its blocks in local coordinates. The body falls and collides with the
//! grid. Once it has rested for a while it goes to sleep and its blocks are put
//! back into the grid at the nearest whole position.
//!
//! The crate knows nothing about the game. It talks to the world through the
//! [`BlockWorld`] trait, so the same code runs on the integrated server, a
//! dedicated server and in tests.
//!
//! Stage 1 (this crate today): translation only, gravity, exact collision of the
//! body's boxes against the grid, sleep and landing. Stage 2 replaces
//! [`physics`] with a Rapier pipeline for rotation and body-to-body contact; the
//! rest of the crate does not change.

pub mod assemble;
pub mod math;
pub mod physics;
pub mod shape;
pub mod sublevel;
pub mod world;

pub use assemble::{AssembleLimits, Unsupported, find_unsupported};
pub use math::{Aabb, BlockPos, Vec3};
pub use shape::merge_boxes;
pub use sublevel::{Landing, SubLevel, SubLevelId, SubLevels};
pub use world::{BlockState, BlockWorld};
