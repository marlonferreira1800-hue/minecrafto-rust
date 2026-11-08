// Clean-room implementation of Minecraft Java Edition 3D Voxel Raycaster (Level.clip / HitResult)
#[cfg(target_arch = "wasm32")]
use crate::minicore::*;

use crate::blocks::*;
use crate::noise::fast_floor;
use crate::world::*;

#[inline(always)]
fn abs_f32(v: f32) -> f32 {
    if v < 0.0 { -v } else { v }
}

/// Performs Amanatides & Woo 3D DDA voxel traversal from eye (ox, oy, oz) along (dx, dy, dz).
/// Returns 1 if a targetable block was hit within max_dist (default 5.5 blocks), or 0 if missed.
/// Writes hit details to state_buf as i32s:
/// [0] = hit_x, [1] = hit_y, [2] = hit_z
/// [3] = normal_x, [4] = normal_y, [5] = normal_z (adjacent placement offset!)
/// [6] = hit_block_id
pub unsafe fn raycast_voxel(
    base_voxels: usize,
    state_buf: usize,
    ox: f32,
    oy: f32,
    oz: f32,
    dx: f32,
    dy: f32,
    dz: f32,
    max_dist: f32,
) -> i32 {
    let mut ix = fast_floor(ox);
    let mut iy = fast_floor(oy);
    let mut iz = fast_floor(oz);

    let step_x = if dx > 0.0 { 1 } else if dx < 0.0 { -1 } else { 0 };
    let step_y = if dy > 0.0 { 1 } else if dy < 0.0 { -1 } else { 0 };
    let step_z = if dz > 0.0 { 1 } else if dz < 0.0 { -1 } else { 0 };

    let t_delta_x = if abs_f32(dx) > 0.00001 {
        abs_f32(1.0 / dx)
    } else {
        1000000.0
    };
    let t_delta_y = if abs_f32(dy) > 0.00001 {
        abs_f32(1.0 / dy)
    } else {
        1000000.0
    };
    let t_delta_z = if abs_f32(dz) > 0.00001 {
        abs_f32(1.0 / dz)
    } else {
        1000000.0
    };

    let mut t_max_x = if dx > 0.0 {
        ((ix + 1) as f32 - ox) * t_delta_x
    } else if dx < 0.0 {
        (ox - ix as f32) * t_delta_x
    } else {
        1000000.0
    };

    let mut t_max_y = if dy > 0.0 {
        ((iy + 1) as f32 - oy) * t_delta_y
    } else if dy < 0.0 {
        (oy - iy as f32) * t_delta_y
    } else {
        1000000.0
    };

    let mut t_max_z = if dz > 0.0 {
        ((iz + 1) as f32 - oz) * t_delta_z
    } else if dz < 0.0 {
        (oz - iz as f32) * t_delta_z
    } else {
        1000000.0
    };

    let mut nx = 0;
    let mut ny = 0;
    let mut nz = 0;
    let mut dist = 0.0;
    let mut steps = 0;

    while steps < 48 && dist <= max_dist {
        if in_bounds(ix, iy, iz) {
            let b = get_block_raw(base_voxels, ix, iy, iz);
            if b != AIR && b != WATER && b != LAVA {
                let out = state_buf as *mut i32;
                *out.offset(0) = ix;
                *out.offset(1) = iy;
                *out.offset(2) = iz;
                *out.offset(3) = nx;
                *out.offset(4) = ny;
                *out.offset(5) = nz;
                *out.offset(6) = b as i32;
                return 1;
            }
        }

        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                ix += step_x;
                dist = t_max_x;
                t_max_x += t_delta_x;
                nx = -step_x;
                ny = 0;
                nz = 0;
            } else {
                iz += step_z;
                dist = t_max_z;
                t_max_z += t_delta_z;
                nx = 0;
                ny = 0;
                nz = -step_z;
            }
        } else if t_max_y < t_max_z {
            iy += step_y;
            dist = t_max_y;
            t_max_y += t_delta_y;
            nx = 0;
            ny = -step_y;
            nz = 0;
        } else {
            iz += step_z;
            dist = t_max_z;
            t_max_z += t_delta_z;
            nx = 0;
            ny = 0;
            nz = -step_z;
        }
        steps += 1;
    }

    0
}
