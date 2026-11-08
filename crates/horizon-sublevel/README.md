# horizon-sublevel

Sub-levels: regions of blocks that leave the world grid, move with physics and
land back on it. The foundation for falling structures, then Create-style
contraptions and Aeronautics-style ships, written for Horizon rather than
ported from Sable (whose PolyForm Shield licence keeps its code off limits).

## Stages

| Stage | What | Status |
|---|---|---|
| 1 | Unsupported groups leave the grid, fall with vanilla falling-block gravity and drag, collide with the grid (axis-clipped, no tunnelling), sleep and land on whole positions; blocks with no room are reported as drops; greedy box merging for collision | **done, tested** (`cargo test -p horizon-sublevel`) |
| 2 | Game integration: wake on `break_block` / explosions on the server thread, tick after `tick_moving_pistons` in `tick_world`, a Horizon payload channel for spawn / pose / land, client drawing through `moving_block_vertices` (as `moving_piston_geometry` does), save with the chunk | next |
| 3 | Rotation and body-to-body contact: a Rapier (Apache-2.0) pipeline behind `physics`, a pose with a rotation, landing snapped to quarter turns | needs `rapier3d` in the workspace |
| 4 | Players and mobs standing on moving sub-levels (carry the rider by the body's motion before entity collision) | |
| 5 | Sub-levels that cross chunks and dimensions freely, persistent ids; then thrusters and lift for ships | |

## Integration points (stage 2)

- Wake: `Game::break_block_as_player` (`crates/horizon-client/src/game.rs`), after the
  block is removed; explosions in `game/explosion.rs`.
- Tick: `Game::tick_world`, next to `tick_moving_pistons`.
- `BlockWorld` for the game: `self.world.block(x, y, z)`, anchors = bedrock and unloaded chunks,
  `can_move` refuses block entities until they have adapters.
- Network: single player is a client of its own server thread, so every pose goes through a
  payload like `game/native_net.rs` (`horizon:sublevel`): spawn (id, blocks), pose (id, position)
  each tick while awake, land (id).
- Client drawing: `Game::moving_block_vertices(state, seed, at, out)` in
  `game/moving_pistons.rs` draws any block at any position; call it per block next to
  `moving_piston_geometry` in `game.rs`.
- Off by default (a Horizon-only gameplay change), behind a game rule or an extras setting.
