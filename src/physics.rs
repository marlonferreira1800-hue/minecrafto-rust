// Clean-room implementation of Minecraft Java Edition Entity AABB Physics
// (net.minecraft.world.phys.AABB & net.minecraft.world.entity.Entity.move)
#[cfg(target_arch = "wasm32")]
use crate::minicore::*;

use crate::blocks::*;
use crate::noise::fast_floor;
use crate::world::*;

pub const PLAYER_HALF_W: f32 = 0.30;
pub const PLAYER_HEIGHT: f32 = 1.80;
pub const PLAYER_EYE_H: f32 = 1.62;

/// Checks if any solid voxel overlaps the AABB [min_x..max_x, min_y..max_y, min_z..max_z]
pub unsafe fn aabb_collides_world(
    base_voxels: usize,
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
) -> bool {
    let x0 = fast_floor(min_x);
    let y0 = fast_floor(min_y);
    let z0 = fast_floor(min_z);
    let x1 = fast_floor(max_x - 0.0001);
    let y1 = fast_floor(max_y - 0.0001);
    let z1 = fast_floor(max_z - 0.0001);

    let mut x = x0;
    while x <= x1 {
        let mut y = y0;
        while y <= y1 {
            let mut z = z0;
            while z <= z1 {
                let b = get_block_raw(base_voxels, x, y, z);
                if is_solid(b) {
                    return true;
                }
                z += 1;
            }
            y += 1;
        }
        x += 1;
    }
    false
}

/// Checks if placing a block at (bx, by, bz) would overlap the player's current AABB
pub fn block_intersects_player(px: f32, py: f32, pz: f32, bx: i32, by: i32, bz: i32) -> bool {
    let p_min_x = px - PLAYER_HALF_W + 0.02;
    let p_max_x = px + PLAYER_HALF_W - 0.02;
    let p_min_y = py + 0.02;
    let p_max_y = py + PLAYER_HEIGHT - 0.02;
    let p_min_z = pz - PLAYER_HALF_W + 0.02;
    let p_max_z = pz + PLAYER_HALF_W - 0.02;

    let b_min_x = bx as f32;
    let b_max_x = b_min_x + 1.0;
    let b_min_y = by as f32;
    let b_max_y = b_min_y + 1.0;
    let b_min_z = bz as f32;
    let b_max_z = b_min_z + 1.0;

    p_max_x > b_min_x
        && p_min_x < b_max_x
        && p_max_y > b_min_y
        && p_min_y < b_max_y
        && p_max_z > b_min_z
        && p_min_z < b_max_z
}

/// Executes one physics step of Minecraft Java Edition Entity.moveAndCollide.
/// Writes results to state_buf as f32s:
/// [0] = new_px, [1] = new_py, [2] = new_pz,
/// [3] = new_vx, [4] = new_vy, [5] = new_vz,
/// [6] = on_ground (1.0 or 0.0), [7] = in_water (1.0 or 0.0), [8] = hazard_damage (1.0 if lava/cactus)
pub unsafe fn step_player_physics(
    base_voxels: usize,
    state_buf: usize,
    mut px: f32,
    mut py: f32,
    mut pz: f32,
    mut vx: f32,
    mut vy: f32,
    mut vz: f32,
    dt: f32,
    flying: i32,
) {
    let clamped_dt = if dt > 0.05 { 0.05 } else { dt };

    // Check water / lava immersion at player waist/feet
    let feet_b = get_block_raw(
        base_voxels,
        fast_floor(px),
        fast_floor(py + 0.4),
        fast_floor(pz),
    );
    let in_water = feet_b == WATER;
    let in_lava = feet_b == LAVA;

    if flying == 0 {
        if in_water || in_lava {
            vy -= 9.5 * clamped_dt; // Reduced fluid gravity
            if vy < -4.5 {
                vy = -4.5;
            }
        } else {
            // Minecraft Java Edition gravity (~28 m/s^2 continuous match for 0.08/tick @ 20Hz)
            vy -= 28.0 * clamped_dt;
            if vy < -55.0 {
                vy = -55.0;
            }
        }
    }

    let dx = vx * clamped_dt;
    let dy = vy * clamped_dt;
    let dz = vz * clamped_dt;

    let mut on_ground = false;

    // 1. Resolve Y axis first (exact Minecraft Java Edition Entity.move order: Y -> X -> Z)
    let target_y = py + dy;
    if !aabb_collides_world(
        base_voxels,
        px - PLAYER_HALF_W,
        target_y,
        pz - PLAYER_HALF_W,
        px + PLAYER_HALF_W,
        target_y + PLAYER_HEIGHT,
        pz + PLAYER_HALF_W,
    ) {
        py = target_y;
    } else {
        if dy < 0.0 {
            on_ground = true;
            py = (fast_floor(target_y) + 1) as f32;
        }
        vy = 0.0;
    }

    // 2. Resolve X axis
    let target_x = px + dx;
    if !aabb_collides_world(
        base_voxels,
        target_x - PLAYER_HALF_W,
        py,
        pz - PLAYER_HALF_W,
        target_x + PLAYER_HALF_W,
        py + PLAYER_HEIGHT,
        pz + PLAYER_HALF_W,
    ) {
        px = target_x;
    } else {
        vx = 0.0;
    }

    // 3. Resolve Z axis
    let target_z = pz + dz;
    if !aabb_collides_world(
        base_voxels,
        px - PLAYER_HALF_W,
        py,
        target_z - PLAYER_HALF_W,
        px + PLAYER_HALF_W,
        py + PLAYER_HEIGHT,
        target_z + PLAYER_HALF_W,
    ) {
        pz = target_z;
    } else {
        vz = 0.0;
    }

    // Keep player inside world X/Z boundaries
    let min_bound = 1.0;
    let max_bound_x = (WORLD_X as f32) - 1.0;
    let max_bound_z = (WORLD_Z as f32) - 1.0;
    if px < min_bound {
        px = min_bound;
    }
    if px > max_bound_x {
        px = max_bound_x;
    }
    if pz < min_bound {
        pz = min_bound;
    }
    if pz > max_bound_z {
        pz = max_bound_z;
    }

    // Check cactus or lava contact damage
    let mut hazard = if in_lava { 1.0 } else { 0.0 };
    if hazard == 0.0 {
        let b_n = get_block_raw(base_voxels, fast_floor(px), fast_floor(py + 0.5), fast_floor(pz - 0.35));
        let b_s = get_block_raw(base_voxels, fast_floor(px), fast_floor(py + 0.5), fast_floor(pz + 0.35));
        let b_e = get_block_raw(base_voxels, fast_floor(px + 0.35), fast_floor(py + 0.5), fast_floor(pz));
        let b_w = get_block_raw(base_voxels, fast_floor(px - 0.35), fast_floor(py + 0.5), fast_floor(pz));
        if b_n == CACTUS || b_s == CACTUS || b_e == CACTUS || b_w == CACTUS {
            hazard = 1.0;
        }
    }

    let out = state_buf as *mut f32;
    *out.offset(0) = px;
    *out.offset(1) = py;
    *out.offset(2) = pz;
    *out.offset(3) = vx;
    *out.offset(4) = vy;
    *out.offset(5) = vz;
    *out.offset(6) = if on_ground { 1.0 } else { 0.0 };
    *out.offset(7) = if in_water { 1.0 } else { 0.0 };
    *out.offset(8) = hazard;
}
