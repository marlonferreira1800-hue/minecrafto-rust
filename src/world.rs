// Clean-room implementation of Minecraft Java Edition World & Chunk Mesher
// (net.minecraft.world.level.Level, LevelChunk, ChunkRenderer)
#[cfg(target_arch = "wasm32")]
use crate::minicore::*;

use crate::blocks::*;
use crate::noise::*;

pub const CHUNK_SIZE: i32 = 16;
pub const CHUNKS_X: i32 = 8;
pub const CHUNKS_Z: i32 = 8;
pub const WORLD_X: i32 = CHUNKS_X * CHUNK_SIZE; // 128
pub const WORLD_Y: i32 = 64;
pub const WORLD_Z: i32 = CHUNKS_Z * CHUNK_SIZE; // 128
pub const WATER_LEVEL: i32 = 22;

// Linear memory layout (for wasm32 and native arena):
// 0x020000 .. 0x120000 (1,048,576 bytes): Voxel Block IDs [WORLD_X * WORLD_Z * WORLD_Y]
// 0x120000 .. 0x220000 (1,048,576 bytes): Voxel Light Levels (high 4 bits = sky, low 4 bits = block light)
// 0x220000 .. 0x520000 (3,145,728 bytes): Chunk Opaque Mesh Float32 Output Buffer
// 0x520000 .. 0x720000 (2,097,152 bytes): Chunk Translucent/Water Mesh Float32 Output Buffer
// 0x720000 .. 0x721000 (4,096 bytes):     Shared State / Raycast / Physics / Crafting I/O Structs
pub const MEM_VOXELS: usize = 0x020000;
pub const MEM_LIGHT: usize = 0x120000;
pub const MEM_MESH_OPAQUE: usize = 0x220000;
pub const MEM_MESH_TRANS: usize = 0x520000;
pub const MEM_STATE: usize = 0x720000;

#[inline(always)]
pub fn in_bounds(x: i32, y: i32, z: i32) -> bool {
    x >= 0 && x < WORLD_X && y >= 0 && y < WORLD_Y && z >= 0 && z < WORLD_Z
}

#[inline(always)]
pub fn voxel_index(x: i32, y: i32, z: i32) -> usize {
    (((x * WORLD_Z + z) * WORLD_Y) + y) as usize
}

#[inline(always)]
pub unsafe fn get_block_raw(base_voxels: usize, x: i32, y: i32, z: i32) -> u8 {
    if !in_bounds(x, y, z) {
        if y < 0 {
            return BEDROCK;
        }
        return AIR;
    }
    let idx = voxel_index(x, y, z);
    let ptr = base_voxels as *const u8;
    *ptr.offset(idx as isize)
}

#[inline(always)]
pub unsafe fn set_block_raw(base_voxels: usize, x: i32, y: i32, z: i32, id: u8) {
    if in_bounds(x, y, z) {
        let idx = voxel_index(x, y, z);
        let ptr = base_voxels as *mut u8;
        *ptr.offset(idx as isize) = id;
    }
}

#[inline(always)]
pub unsafe fn get_light_raw(base_light: usize, x: i32, y: i32, z: i32) -> u8 {
    if !in_bounds(x, y, z) {
        return 0xF0; // Full sky light above/outside world
    }
    let idx = voxel_index(x, y, z);
    let ptr = base_light as *const u8;
    *ptr.offset(idx as isize)
}

#[inline(always)]
pub unsafe fn set_light_raw(base_light: usize, x: i32, y: i32, z: i32, val: u8) {
    if in_bounds(x, y, z) {
        let idx = voxel_index(x, y, z);
        let ptr = base_light as *mut u8;
        *ptr.offset(idx as isize) = val;
    }
}

/// Recomputes sky light and local emitter block light for a region around (cx0..cx1, cz0..cz1)
pub unsafe fn recompute_lighting_region(
    base_voxels: usize,
    base_light: usize,
    x0: i32,
    z0: i32,
    x1: i32,
    z1: i32,
) {
    let min_x = if x0 < 0 { 0 } else { x0 };
    let min_z = if z0 < 0 { 0 } else { z0 };
    let max_x = if x1 > WORLD_X { WORLD_X } else { x1 };
    let max_z = if z1 > WORLD_Z { WORLD_Z } else { z1 };

    // Pass 1: Vertical Sky Light column cast (Minecraft Java Level.updateSkyBrightness)
    let mut x = min_x;
    while x < max_x {
        let mut z = min_z;
        while z < max_z {
            let mut sky: u8 = 15;
            let mut y = WORLD_Y - 1;
            while y >= 0 {
                let b = get_block_raw(base_voxels, x, y, z);
                if b != AIR && b != GLASS {
                    if b == LEAVES || b == WATER {
                        if sky > 2 {
                            sky -= 2;
                        } else {
                            sky = 0;
                        }
                    } else if is_opaque(b) {
                        sky = 0;
                    }
                }
                let emit = light_emission(b);
                set_light_raw(base_light, x, y, z, (sky << 4) | (emit & 0x0F));
                y -= 1;
            }
            z += 1;
        }
        x += 1;
    }

    // Pass 2: Local 3D relaxation for block light & cave sky light bleed (4 iterations)
    let mut iter = 0;
    while iter < 4 {
        let mut rx = min_x;
        while rx < max_x {
            let mut rz = min_z;
            while rz < max_z {
                let mut ry = 1;
                while ry < WORLD_Y - 1 {
                    let b = get_block_raw(base_voxels, rx, ry, rz);
                    if !is_opaque(b) || light_emission(b) > 0 {
                        let cur = get_light_raw(base_light, rx, ry, rz);
                        let mut sky = (cur >> 4) & 0x0F;
                        let mut blk = cur & 0x0F;

                        let n0 = get_light_raw(base_light, rx + 1, ry, rz);
                        let n1 = get_light_raw(base_light, rx - 1, ry, rz);
                        let n2 = get_light_raw(base_light, rx, ry + 1, rz);
                        let n3 = get_light_raw(base_light, rx, ry - 1, rz);
                        let n4 = get_light_raw(base_light, rx, ry, rz + 1);
                        let n5 = get_light_raw(base_light, rx, ry, rz - 1);

                        let ptr_n = [n0, n1, n2, n3, n4, n5];
                        let mut ni: isize = 0;
                        while ni < 6 {
                            let nb = *((&ptr_n as *const [u8; 6] as *const u8).offset(ni));
                            let n_sky = (nb >> 4) & 0x0F;
                            let n_blk = nb & 0x0F;
                            if n_sky > sky + 1 {
                                sky = n_sky - 1;
                            }
                            if n_blk > blk + 1 {
                                blk = n_blk - 1;
                            }
                            ni += 1;
                        }
                        set_light_raw(base_light, rx, ry, rz, (sky << 4) | (blk & 0x0F));
                    }
                    ry += 1;
                }
                rz += 1;
            }
            rx += 1;
        }
        iter += 1;
    }
}

/// Generates the entire 8x8 chunk world (128x64x128) in linear memory using Seed
pub unsafe fn generate_world(base_voxels: usize, base_light: usize, seed: i32) {
    // Pass 1: Terrain heightmap, Caves, Strata, Ores, and Water
    let mut x = 0;
    while x < WORLD_X {
        let mut z = 0;
        while z < WORLD_Z {
            let biome = get_biome(x, z, seed);
            let surf_y = get_surface_height(x, z, seed);

            let mut y = 0;
            while y < WORLD_Y {
                let mut block = AIR;
                if y == 0 {
                    block = BEDROCK;
                } else if y <= surf_y {
                    // Check 3D cave noise below surface
                    let mut is_cave = false;
                    if y > 2 && y < surf_y - 1 {
                        let c1 = noise3d(
                            (x as f32) * 0.065,
                            (y as f32) * 0.085,
                            (z as f32) * 0.065,
                            seed + 555,
                        );
                        let c2 = noise3d(
                            (x as f32) * 0.04,
                            (y as f32) * 0.05,
                            (z as f32) * 0.04,
                            seed + 888,
                        );
                        if c1 > 0.43 && c2 > 0.12 {
                            is_cave = true;
                        }
                    }

                    if is_cave {
                        if y <= 5 {
                            block = LAVA;
                        } else {
                            block = AIR;
                        }
                    } else if y == surf_y {
                        if surf_y <= WATER_LEVEL + 1 && biome != BIOME_SNOW_MOUNTAINS {
                            block = SAND;
                        } else if biome == BIOME_DESERT {
                            block = SAND;
                        } else if biome == BIOME_SNOW_MOUNTAINS && surf_y >= 34 {
                            block = SNOW_GRASS;
                        } else {
                            block = GRASS;
                        }
                    } else if y >= surf_y - 3 {
                        if biome == BIOME_DESERT || surf_y <= WATER_LEVEL + 1 {
                            block = SAND;
                        } else {
                            block = DIRT;
                        }
                    } else {
                        // Stone + Ore generation (WorldGenMinable equivalent)
                        let ore_hash = hash_coord(x, y, z, seed + 1999) & 1023;
                        if y <= 13 && ore_hash < 5 {
                            block = DIAMOND_ORE;
                        } else if y <= 24 && ore_hash >= 5 && ore_hash < 16 {
                            block = GOLD_ORE;
                        } else if y <= 42 && ore_hash >= 16 && ore_hash < 48 {
                            block = IRON_ORE;
                        } else if ore_hash >= 48 && ore_hash < 98 {
                            block = COAL_ORE;
                        } else {
                            block = STONE;
                        }
                    }
                } else if y <= WATER_LEVEL {
                    block = WATER;
                }

                set_block_raw(base_voxels, x, y, z, block);
                y += 1;
            }
            z += 1;
        }
        x += 1;
    }

    // Pass 2: Trees, Cacti, and Glowstone Cave Clusters (Feature placement)
    let mut tx = 4;
    while tx < WORLD_X - 4 {
        let mut tz = 4;
        while tz < WORLD_Z - 4 {
            // Keep a small clear meadow right at initial spawn (64, 64) so player isn't inside a tree
            let dx_spawn = tx - 64;
            let dz_spawn = tz - 64;
            let dist_spawn_sq = dx_spawn * dx_spawn + dz_spawn * dz_spawn;

            if dist_spawn_sq > 16 {
                let h = hash_coord(tx, 0, tz, seed + 4242) & 255;
                let biome = get_biome(tx, tz, seed);
                let sy = get_surface_height(tx, tz, seed);

                if sy > WATER_LEVEL + 1 && sy < WORLD_Y - 9 {
                    let top_id = get_block_raw(base_voxels, tx, sy, tz);
                    if (top_id == GRASS || top_id == SNOW_GRASS)
                        && ((biome == BIOME_FOREST && h < 11)
                            || (biome == BIOME_PLAINS && h < 3)
                            || (biome == BIOME_SNOW_MOUNTAINS && h < 5))
                    {
                        place_oak_tree(base_voxels, tx, sy + 1, tz, seed);
                        tz += 4; // Avoid overlapping trunks
                    } else if top_id == SAND && biome == BIOME_DESERT && h < 5 {
                        // Place 3-block tall Desert Cactus
                        set_block_raw(base_voxels, tx, sy + 1, tz, CACTUS);
                        set_block_raw(base_voxels, tx, sy + 2, tz, CACTUS);
                        set_block_raw(base_voxels, tx, sy + 3, tz, CACTUS);
                        tz += 3;
                    }
                }
            }
            tz += 1;
        }
        tx += 1;
    }

    // Compute initial sky + block lighting across the world
    recompute_lighting_region(base_voxels, base_light, 0, 0, WORLD_X, WORLD_Z);
}

/// Places a classic Minecraft Java Edition Oak Tree (TreeFeature.java)
unsafe fn place_oak_tree(base_voxels: usize, x: i32, y: i32, z: i32, seed: i32) {
    let trunk_h = 4 + (hash_coord(x, y, z, seed + 77) & 1);
    set_block_raw(base_voxels, x, y - 1, z, DIRT);

    // Leaf canopy (2 wide lower layers, 1 wide upper layers)
    let mut ly = y + trunk_h - 2;
    while ly <= y + trunk_h + 1 {
        let dy = ly - (y + trunk_h);
        let radius = if dy < 0 { 2 } else { 1 };
        let mut lx = x - radius;
        while lx <= x + radius {
            let mut lz = z - radius;
            while lz <= z + radius {
                let ax = if lx - x < 0 { x - lx } else { lx - x };
                let az = if lz - z < 0 { z - lz } else { lz - z };
                // Round corners like Java Edition TreeFeature
                if !(ax == radius && az == radius && (dy >= 0 || ((lx + lz) & 1) == 0)) {
                    let existing = get_block_raw(base_voxels, lx, ly, lz);
                    if existing == AIR {
                        set_block_raw(base_voxels, lx, ly, lz, LEAVES);
                    }
                }
                lz += 1;
            }
            lx += 1;
        }
        ly += 1;
    }

    // Vertical Wood Log trunk
    let mut ty = 0;
    while ty < trunk_h {
        set_block_raw(base_voxels, x, y + ty, z, WOOD_LOG);
        ty += 1;
    }
}

/// Finds the highest solid block Y at (wx, wz) for safe player spawning
pub unsafe fn find_spawn_y(base_voxels: usize, wx: i32, wz: i32) -> i32 {
    let mut y = WORLD_Y - 2;
    while y > 1 {
        let b = get_block_raw(base_voxels, wx, y, wz);
        if is_solid(b) && b != LEAVES && b != CACTUS {
            return y + 1;
        }
        y -= 1;
    }
    WATER_LEVEL + 2
}

/// Searches outward from (64, 64) for dry land above water level and writes (spawn_x, spawn_y, spawn_z) to state_buf
pub unsafe fn find_best_spawn(base_voxels: usize, state_buf: usize) -> i32 {
    let mut radius = 0;
    while radius < 36 {
        let mut dx = -radius;
        while dx <= radius {
            let mut dz = -radius;
            while dz <= radius {
                if dx == -radius || dx == radius || dz == -radius || dz == radius {
                    let wx = 64 + dx;
                    let wz = 64 + dz;
                    let sy = find_spawn_y(base_voxels, wx, wz);
                    let head = get_block_raw(base_voxels, wx, sy, wz);
                    if sy > WATER_LEVEL + 1 && head == AIR {
                        let out = state_buf as *mut i32;
                        *out.offset(0) = wx;
                        *out.offset(1) = sy;
                        *out.offset(2) = wz;
                        return sy;
                    }
                }
                dz += 2;
            }
            dx += 2;
        }
        radius += 2;
    }
    let sy = find_spawn_y(base_voxels, 64, 64);
    let out = state_buf as *mut i32;
    *out.offset(0) = 64;
    *out.offset(1) = sy;
    *out.offset(2) = 64;
    sy
}

/// Triggers a TNT spherical crater explosion at (cx, cy, cz) with radius 4
pub unsafe fn explode_at(base_voxels: usize, base_light: usize, cx: i32, cy: i32, cz: i32, radius: i32) {
    let r2 = radius * radius;
    let mut x = cx - radius;
    while x <= cx + radius {
        let mut y = cy - radius;
        while y <= cy + radius {
            let mut z = cz - radius;
            while z <= cz + radius {
                let dx = x - cx;
                let dy = y - cy;
                let dz = z - cz;
                if dx * dx + dy * dy + dz * dz <= r2 && y > 0 {
                    let b = get_block_raw(base_voxels, x, y, z);
                    if b != BEDROCK {
                        set_block_raw(base_voxels, x, y, z, AIR);
                    }
                }
                z += 1;
            }
            y += 1;
        }
        x += 1;
    }
    recompute_lighting_region(
        base_voxels,
        base_light,
        cx - radius - 4,
        cz - radius - 4,
        cx + radius + 5,
        cz + radius + 5,
    );
}

#[inline(always)]
unsafe fn ao_solid(base_voxels: usize, x: i32, y: i32, z: i32) -> i32 {
    let b = get_block_raw(base_voxels, x, y, z);
    if is_opaque(b) { 1 } else { 0 }
}

/// Computes Minecraft Java Edition Smooth Lighting vertex Ambient Occlusion (0..3 -> 0.52..1.0)
#[inline(always)]
fn vertex_ao_factor(s1: i32, s2: i32, corner: i32) -> f32 {
    let occ = if s1 == 1 && s2 == 1 {
        3
    } else {
        s1 + s2 + corner
    };
    1.0 - (occ as f32) * 0.155
}

#[inline(always)]
unsafe fn push_vertex(
    buf_base: usize,
    float_idx: &mut usize,
    vx: f32,
    vy: f32,
    vz: f32,
    u: f32,
    v: f32,
    light: f32,
    sky_factor: f32,
    block_light: f32,
) {
    let ptr = buf_base as *mut f32;
    let i = *float_idx as isize;
    *ptr.offset(i) = vx;
    *ptr.offset(i + 1) = vy;
    *ptr.offset(i + 2) = vz;
    *ptr.offset(i + 3) = u;
    *ptr.offset(i + 4) = v;
    *ptr.offset(i + 5) = light;
    *ptr.offset(i + 6) = sky_factor;
    *ptr.offset(i + 7) = block_light;
    *float_idx += 8;
}

/// Emits 2 triangles (6 vertices, 8 floats per vertex) for a visible block face with Smooth AO
unsafe fn emit_face(
    base_voxels: usize,
    base_light: usize,
    out_buf: usize,
    float_idx: &mut usize,
    x: i32,
    y: i32,
    z: i32,
    block_id: u8,
    face: i32,
) {
    let slot = face_texture_slot(block_id, face);
    let col = slot % 6;
    let row = slot / 6;
    let cell = 1.0 / 6.0;
    let pad = 0.0015;
    let u0 = (col as f32) * cell + pad;
    let v0 = (row as f32) * cell + pad;
    let u1 = ((col + 1) as f32) * cell - pad;
    let v1 = ((row + 1) as f32) * cell - pad;

    let shade = face_shade(face);

    // Neighbor coordinates for sampling light
    let (nx, ny, nz) = if face == FACE_TOP {
        (x, y + 1, z)
    } else if face == FACE_BOTTOM {
        (x, y - 1, z)
    } else if face == FACE_SOUTH {
        (x, y, z + 1)
    } else if face == FACE_NORTH {
        (x, y, z - 1)
    } else if face == FACE_EAST {
        (x + 1, y, z)
    } else {
        (x - 1, y, z)
    };

    let raw_light = get_light_raw(base_light, nx, ny, nz);
    let sky_lvl = ((raw_light >> 4) & 0x0F) as f32 / 15.0;
    let mut blk_lvl = (raw_light & 0x0F) as f32 / 15.0;
    let own_emit = light_emission(block_id);
    if own_emit > 0 {
        blk_lvl = 1.0;
    }

    let fx0 = x as f32;
    let fy0 = y as f32;
    let fz0 = z as f32;
    let fx1 = fx0 + 1.0;
    // Lower top surface of water slightly (0.88 height) just like Minecraft Java LiquidTile!
    let fy1 = if block_id == WATER {
        fy0 + 0.88
    } else {
        fy0 + 1.0
    };
    let fz1 = fz0 + 1.0;

    // Compute per-vertex AO for the 4 corners of the face
    let (ao0, ao1, ao2, ao3) = if face == FACE_TOP {
        let n = ao_solid(base_voxels, x, y + 1, z - 1);
        let s = ao_solid(base_voxels, x, y + 1, z + 1);
        let w = ao_solid(base_voxels, x - 1, y + 1, z);
        let e = ao_solid(base_voxels, x + 1, y + 1, z);
        let nw = ao_solid(base_voxels, x - 1, y + 1, z - 1);
        let ne = ao_solid(base_voxels, x + 1, y + 1, z - 1);
        let sw = ao_solid(base_voxels, x - 1, y + 1, z + 1);
        let se = ao_solid(base_voxels, x + 1, y + 1, z + 1);
        (
            vertex_ao_factor(w, s, sw),
            vertex_ao_factor(e, s, se),
            vertex_ao_factor(e, n, ne),
            vertex_ao_factor(w, n, nw),
        )
    } else {
        let base_ao = if face == FACE_BOTTOM { 0.85 } else { 0.94 };
        (base_ao, base_ao, base_ao, base_ao)
    };

    let l0 = shade * ao0;
    let l1 = shade * ao1;
    let l2 = shade * ao2;
    let l3 = shade * ao3;

    if face == FACE_TOP {
        push_vertex(out_buf, float_idx, fx0, fy1, fz1, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy1, fz1, u1, v1, l1, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy1, fz0, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy1, fz1, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy1, fz0, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy1, fz0, u0, v0, l3, sky_lvl, blk_lvl);
    } else if face == FACE_BOTTOM {
        push_vertex(out_buf, float_idx, fx0, fy0, fz0, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy0, fz0, u1, v1, l1, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy0, fz1, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy0, fz0, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy0, fz1, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy0, fz1, u0, v0, l3, sky_lvl, blk_lvl);
    } else if face == FACE_SOUTH {
        push_vertex(out_buf, float_idx, fx0, fy0, fz1, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy0, fz1, u1, v1, l1, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy1, fz1, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy0, fz1, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy1, fz1, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy1, fz1, u0, v0, l3, sky_lvl, blk_lvl);
    } else if face == FACE_NORTH {
        push_vertex(out_buf, float_idx, fx1, fy0, fz0, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy0, fz0, u1, v1, l1, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy1, fz0, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy0, fz0, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy1, fz0, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy1, fz0, u0, v0, l3, sky_lvl, blk_lvl);
    } else if face == FACE_EAST {
        push_vertex(out_buf, float_idx, fx1, fy0, fz1, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy0, fz0, u1, v1, l1, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy1, fz0, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy0, fz1, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy1, fz0, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx1, fy1, fz1, u0, v0, l3, sky_lvl, blk_lvl);
    } else {
        // FACE_WEST
        push_vertex(out_buf, float_idx, fx0, fy0, fz0, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy0, fz1, u1, v1, l1, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy1, fz1, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy0, fz0, u0, v1, l0, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy1, fz1, u1, v0, l2, sky_lvl, blk_lvl);
        push_vertex(out_buf, float_idx, fx0, fy1, fz0, u0, v0, l3, sky_lvl, blk_lvl);
    }
}

#[inline(always)]
fn should_render_face(block: u8, neighbor: u8) -> bool {
    if neighbor == AIR {
        return true;
    }
    if block == WATER {
        return neighbor != WATER && !is_opaque(neighbor);
    }
    if block == GLASS {
        return neighbor != GLASS && !is_opaque(neighbor);
    }
    if block == LEAVES {
        return !is_opaque(neighbor);
    }
    !is_opaque(neighbor)
}

/// Builds both the Opaque and Translucent (Water) vertex buffers for Chunk (cx, cz).
/// Stores vertex counts in MEM_STATE:
/// - MEM_STATE[0] = opaque vertex count
/// - MEM_STATE[1] = translucent vertex count
pub unsafe fn build_chunk_mesh(
    base_voxels: usize,
    base_light: usize,
    opaque_buf: usize,
    trans_buf: usize,
    state_buf: usize,
    cx: i32,
    cz: i32,
) -> i32 {
    let x0 = cx * CHUNK_SIZE;
    let z0 = cz * CHUNK_SIZE;
    let x1 = x0 + CHUNK_SIZE;
    let z1 = z0 + CHUNK_SIZE;

    let mut opaque_floats: usize = 0;
    let mut trans_floats: usize = 0;

    let mut x = x0;
    while x < x1 {
        let mut z = z0;
        while z < z1 {
            let mut y = 0;
            while y < WORLD_Y {
                let b = get_block_raw(base_voxels, x, y, z);
                if b != AIR {
                    let is_water = b == WATER;
                    let target_buf = if is_water { trans_buf } else { opaque_buf };
                    let target_idx = if is_water {
                        &mut trans_floats
                    } else {
                        &mut opaque_floats
                    };

                    // +Y (Top)
                    if should_render_face(b, get_block_raw(base_voxels, x, y + 1, z)) {
                        emit_face(base_voxels, base_light, target_buf, target_idx, x, y, z, b, FACE_TOP);
                    }
                    // -Y (Bottom)
                    if y > 0 && should_render_face(b, get_block_raw(base_voxels, x, y - 1, z)) {
                        emit_face(base_voxels, base_light, target_buf, target_idx, x, y, z, b, FACE_BOTTOM);
                    }
                    // +Z (South)
                    if should_render_face(b, get_block_raw(base_voxels, x, y, z + 1)) {
                        emit_face(base_voxels, base_light, target_buf, target_idx, x, y, z, b, FACE_SOUTH);
                    }
                    // -Z (North)
                    if should_render_face(b, get_block_raw(base_voxels, x, y, z - 1)) {
                        emit_face(base_voxels, base_light, target_buf, target_idx, x, y, z, b, FACE_NORTH);
                    }
                    // +X (East)
                    if should_render_face(b, get_block_raw(base_voxels, x + 1, y, z)) {
                        emit_face(base_voxels, base_light, target_buf, target_idx, x, y, z, b, FACE_EAST);
                    }
                    // -X (West)
                    if should_render_face(b, get_block_raw(base_voxels, x - 1, y, z)) {
                        emit_face(base_voxels, base_light, target_buf, target_idx, x, y, z, b, FACE_WEST);
                    }
                }
                y += 1;
            }
            z += 1;
        }
        x += 1;
    }

    let opaque_verts = (opaque_floats / 8) as i32;
    let trans_verts = (trans_floats / 8) as i32;
    let state_ptr = state_buf as *mut i32;
    *state_ptr.offset(0) = opaque_verts;
    *state_ptr.offset(1) = trans_verts;
    opaque_verts
}
