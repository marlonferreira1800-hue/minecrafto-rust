#![cfg_attr(
    target_arch = "wasm32",
    feature(no_core, lang_items, auto_traits, intrinsics),
    no_core,
    allow(internal_features, unused_imports)
)]

#[cfg(target_arch = "wasm32")]
pub mod minicore;
#[cfg(target_arch = "wasm32")]
pub use minicore::*;

pub mod blocks;
pub mod crafting;
pub mod noise;
pub mod physics;
pub mod raycast;
pub mod world;

use blocks::*;
use crafting::*;
use noise::*;
use physics::*;
use raycast::*;
use world::*;

#[no_mangle]
pub extern "C" fn wasm_get_voxels_ptr() -> usize {
    MEM_VOXELS
}

#[no_mangle]
pub extern "C" fn wasm_get_light_ptr() -> usize {
    MEM_LIGHT
}

#[no_mangle]
pub extern "C" fn wasm_get_opaque_mesh_ptr() -> usize {
    MEM_MESH_OPAQUE
}

#[no_mangle]
pub extern "C" fn wasm_get_trans_mesh_ptr() -> usize {
    MEM_MESH_TRANS
}

#[no_mangle]
pub extern "C" fn wasm_get_state_ptr() -> usize {
    MEM_STATE
}

#[no_mangle]
pub unsafe extern "C" fn wasm_init_world(seed: i32) -> i32 {
    generate_world(MEM_VOXELS, MEM_LIGHT, seed);
    find_best_spawn(MEM_VOXELS, MEM_STATE)
}

#[no_mangle]
pub unsafe extern "C" fn wasm_relight_all() {
    recompute_lighting_region(MEM_VOXELS, MEM_LIGHT, 0, 0, WORLD_X, WORLD_Z);
}

#[no_mangle]
pub unsafe extern "C" fn wasm_get_block(x: i32, y: i32, z: i32) -> i32 {
    get_block_raw(MEM_VOXELS, x, y, z) as i32
}

#[no_mangle]
pub unsafe extern "C" fn wasm_set_block(x: i32, y: i32, z: i32, block_id: i32) -> i32 {
    if !in_bounds(x, y, z) {
        return 0;
    }
    set_block_raw(MEM_VOXELS, x, y, z, block_id as u8);
    recompute_lighting_region(MEM_VOXELS, MEM_LIGHT, x - 6, z - 6, x + 7, z + 7);
    1
}

#[no_mangle]
pub unsafe extern "C" fn wasm_explode(x: i32, y: i32, z: i32, radius: i32) {
    explode_at(MEM_VOXELS, MEM_LIGHT, x, y, z, radius);
}

#[no_mangle]
pub unsafe extern "C" fn wasm_build_chunk_mesh(cx: i32, cz: i32) -> i32 {
    build_chunk_mesh(
        MEM_VOXELS,
        MEM_LIGHT,
        MEM_MESH_OPAQUE,
        MEM_MESH_TRANS,
        MEM_STATE,
        cx,
        cz,
    )
}

#[no_mangle]
pub unsafe extern "C" fn wasm_raycast(
    ox: f32,
    oy: f32,
    oz: f32,
    dx: f32,
    dy: f32,
    dz: f32,
    max_dist: f32,
) -> i32 {
    raycast_voxel(MEM_VOXELS, MEM_STATE, ox, oy, oz, dx, dy, dz, max_dist)
}

#[no_mangle]
pub unsafe extern "C" fn wasm_step_physics(
    px: f32,
    py: f32,
    pz: f32,
    vx: f32,
    vy: f32,
    vz: f32,
    dt: f32,
    flying: i32,
) {
    step_player_physics(MEM_VOXELS, MEM_STATE, px, py, pz, vx, vy, vz, dt, flying);
}

#[no_mangle]
pub extern "C" fn wasm_can_place_block(
    px: f32,
    py: f32,
    pz: f32,
    bx: i32,
    by: i32,
    bz: i32,
) -> i32 {
    if block_intersects_player(px, py, pz, bx, by, bz) {
        0
    } else {
        1
    }
}

#[no_mangle]
pub extern "C" fn wasm_get_biome(x: i32, z: i32, seed: i32) -> i32 {
    get_biome(x, z, seed) as i32
}

#[no_mangle]
pub extern "C" fn wasm_block_hardness(block_id: i32) -> f32 {
    block_hardness(block_id as u8)
}

#[no_mangle]
pub extern "C" fn wasm_evaluate_craft(
    g0: i32,
    g1: i32,
    g2: i32,
    g3: i32,
    g4: i32,
    g5: i32,
    g6: i32,
    g7: i32,
    g8: i32,
) -> i32 {
    evaluate_crafting_grid(
        g0 as u8, g1 as u8, g2 as u8, g3 as u8, g4 as u8, g5 as u8, g6 as u8, g7 as u8, g8 as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perlin_noise_and_biomes() {
        let h1 = get_surface_height(64, 64, 1337);
        let h2 = get_surface_height(64, 64, 1337);
        assert_eq!(h1, h2, "Noise must be deterministic for a given seed");
        assert!(h1 >= 8 && h1 <= 54, "Surface height within valid bounds");

        let b = get_biome(32, 96, 1337);
        assert!(b <= BIOME_SNOW_MOUNTAINS);
    }

    #[test]
    fn test_crafting_recipes() {
        // 1 Wood Log -> 4 Planks
        let planks = evaluate_crafting_grid(WOOD_LOG, 0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(planks & 0xFF, PLANKS as i32);
        assert_eq!(planks >> 8, 4);

        // 2 Planks vertical -> 4 Sticks
        let sticks = evaluate_crafting_grid(PLANKS, 0, 0, PLANKS, 0, 0, 0, 0, 0);
        assert_eq!(sticks & 0xFF, ITEM_STICK as i32);
        assert_eq!(sticks >> 8, 4);

        // 2x2 Planks -> 1 Crafting Table
        let table = evaluate_crafting_grid(PLANKS, PLANKS, 0, PLANKS, PLANKS, 0, 0, 0, 0);
        assert_eq!(table & 0xFF, CRAFTING_TABLE as i32);
        assert_eq!(table >> 8, 1);

        // Wooden Pickaxe (3 Planks + 2 Sticks)
        let pick = evaluate_crafting_grid(
            PLANKS, PLANKS, PLANKS, 0, ITEM_STICK, 0, 0, ITEM_STICK, 0,
        );
        assert_eq!(pick & 0xFF, ITEM_WOOD_PICKAXE as i32);
        assert_eq!(pick >> 8, 1);

        // Furnace (8 Cobblestone ring)
        let furnace = evaluate_crafting_grid(
            COBBLESTONE,
            COBBLESTONE,
            COBBLESTONE,
            COBBLESTONE,
            0,
            COBBLESTONE,
            COBBLESTONE,
            COBBLESTONE,
            COBBLESTONE,
        );
        assert_eq!(furnace & 0xFF, FURNACE as i32);
    }

    #[test]
    fn test_world_generation_and_raycast() {
        let mut voxels = vec![0u8; (WORLD_X * WORLD_Y * WORLD_Z) as usize];
        let mut light = vec![0u8; (WORLD_X * WORLD_Y * WORLD_Z) as usize];
        let mut state = [0i32; 16];

        unsafe {
            let v_ptr = voxels.as_mut_ptr() as usize;
            let l_ptr = light.as_mut_ptr() as usize;
            let s_ptr = state.as_mut_ptr() as usize;

            generate_world(v_ptr, l_ptr, 2026);
            let spawn_y = find_spawn_y(v_ptr, 64, 64);
            assert!(spawn_y > 8 && spawn_y < 60);

            // Raycast straight down from above spawn_y should hit ground block at spawn_y - 1
            let hit = raycast_voxel(
                v_ptr,
                s_ptr,
                64.5,
                (spawn_y as f32) + 1.5,
                64.5,
                0.0,
                -1.0,
                0.0,
                5.0,
            );
            assert_eq!(hit, 1);
            assert_eq!(state[0], 64);
            assert_eq!(state[1], spawn_y - 1);
            assert_eq!(state[2], 64);
            assert_eq!(state[4], 1); // Normal +Y
        }
    }
}
