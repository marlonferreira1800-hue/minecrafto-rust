// Clean-room implementation of Minecraft Java Edition Block Registry (net.minecraft.world.level.tile.Tile)
#[cfg(target_arch = "wasm32")]
use crate::minicore::*;

pub const AIR: u8 = 0;
pub const STONE: u8 = 1;
pub const GRASS: u8 = 2;
pub const DIRT: u8 = 3;
pub const COBBLESTONE: u8 = 4;
pub const PLANKS: u8 = 5;
pub const BEDROCK: u8 = 6;
pub const SAND: u8 = 7;
pub const WATER: u8 = 8;
pub const WOOD_LOG: u8 = 9;
pub const LEAVES: u8 = 10;
pub const GLASS: u8 = 11;
pub const COAL_ORE: u8 = 12;
pub const IRON_ORE: u8 = 13;
pub const GOLD_ORE: u8 = 14;
pub const DIAMOND_ORE: u8 = 15;
pub const CRAFTING_TABLE: u8 = 16;
pub const FURNACE: u8 = 17;
pub const GLOWSTONE: u8 = 18;
pub const BRICKS: u8 = 19;
pub const SNOW_GRASS: u8 = 20;
pub const CACTUS: u8 = 21;
pub const TNT: u8 = 22;
pub const BOOKSHELF: u8 = 23;
pub const LAVA: u8 = 24;

// Items & Tools for Crafting / Survival
pub const ITEM_STICK: u8 = 50;
pub const ITEM_COAL: u8 = 51;
pub const ITEM_IRON_INGOT: u8 = 52;
pub const ITEM_GOLD_INGOT: u8 = 53;
pub const ITEM_DIAMOND: u8 = 54;
pub const ITEM_WOOD_PICKAXE: u8 = 60;
pub const ITEM_STONE_PICKAXE: u8 = 61;
pub const ITEM_IRON_PICKAXE: u8 = 62;
pub const ITEM_DIAMOND_PICKAXE: u8 = 63;
pub const ITEM_APPLE: u8 = 70;

// Face indices matching Minecraft Java Edition Facing enum:
// 0 = +Y (Top / UP)
// 1 = -Y (Bottom / DOWN)
// 2 = +Z (South)
// 3 = -Z (North)
// 4 = +X (East)
// 5 = -X (West)
pub const FACE_TOP: i32 = 0;
pub const FACE_BOTTOM: i32 = 1;
pub const FACE_SOUTH: i32 = 2;
pub const FACE_NORTH: i32 = 3;
pub const FACE_EAST: i32 = 4;
pub const FACE_WEST: i32 = 5;

#[inline(always)]
pub fn is_solid(block_id: u8) -> bool {
    block_id != AIR && block_id != WATER && block_id != LAVA
}

#[inline(always)]
pub fn is_fluid(block_id: u8) -> bool {
    block_id == WATER || block_id == LAVA
}

#[inline(always)]
pub fn is_opaque(block_id: u8) -> bool {
    block_id != AIR && block_id != WATER && block_id != LEAVES && block_id != GLASS && block_id != CACTUS
}

#[inline(always)]
pub fn is_translucent(block_id: u8) -> bool {
    block_id == WATER || block_id == GLASS || block_id == LEAVES
}

#[inline(always)]
pub fn light_emission(block_id: u8) -> u8 {
    if block_id == GLOWSTONE || block_id == LAVA {
        15
    } else if block_id == FURNACE {
        12
    } else {
        0
    }
}

/// Returns Minecraft Java Edition block hardness (seconds base to mine by hand, -1.0 = unbreakable)
#[inline(always)]
pub fn block_hardness(block_id: u8) -> f32 {
    if block_id == BEDROCK {
        -1.0
    } else if block_id == STONE || block_id == FURNACE {
        1.5
    } else if block_id == COBBLESTONE || block_id == PLANKS || block_id == WOOD_LOG || block_id == BRICKS {
        2.0
    } else if block_id == COAL_ORE || block_id == IRON_ORE || block_id == GOLD_ORE || block_id == DIAMOND_ORE {
        3.0
    } else if block_id == GRASS || block_id == SNOW_GRASS {
        0.6
    } else if block_id == DIRT || block_id == SAND {
        0.5
    } else if block_id == LEAVES || block_id == TNT {
        0.2
    } else if block_id == GLASS || block_id == GLOWSTONE {
        0.3
    } else if block_id == CACTUS {
        0.4
    } else {
        1.0
    }
}

/// Maps (block_id, face_dir) to a 0..35 tile index in the 6x6 16x16 pixel-art texture atlas.
/// Equivalent to Tile.getTexture(int face) in Minecraft Java Edition.
#[inline(always)]
pub fn face_texture_slot(block_id: u8, face: i32) -> i32 {
    if block_id == STONE {
        1
    } else if block_id == GRASS {
        if face == FACE_TOP {
            0 // grass_top
        } else if face == FACE_BOTTOM {
            2 // dirt
        } else {
            3 // grass_side
        }
    } else if block_id == DIRT {
        2
    } else if block_id == COBBLESTONE {
        4
    } else if block_id == PLANKS {
        5
    } else if block_id == BEDROCK {
        6
    } else if block_id == SAND {
        7
    } else if block_id == WATER {
        8
    } else if block_id == WOOD_LOG {
        if face == FACE_TOP || face == FACE_BOTTOM {
            10 // log_top
        } else {
            9 // log_side
        }
    } else if block_id == LEAVES {
        11
    } else if block_id == GLASS {
        12
    } else if block_id == COAL_ORE {
        13
    } else if block_id == IRON_ORE {
        14
    } else if block_id == GOLD_ORE {
        15
    } else if block_id == DIAMOND_ORE {
        16
    } else if block_id == CRAFTING_TABLE {
        if face == FACE_TOP {
            17 // crafting_top
        } else if face == FACE_BOTTOM {
            5 // planks
        } else {
            18 // crafting_side
        }
    } else if block_id == FURNACE {
        if face == FACE_TOP || face == FACE_BOTTOM {
            1 // stone
        } else if face == FACE_SOUTH {
            19 // furnace_front
        } else {
            20 // furnace_side
        }
    } else if block_id == GLOWSTONE {
        21
    } else if block_id == BRICKS {
        22
    } else if block_id == SNOW_GRASS {
        if face == FACE_TOP {
            23 // snow_top
        } else if face == FACE_BOTTOM {
            2 // dirt
        } else {
            24 // snow_side
        }
    } else if block_id == CACTUS {
        if face == FACE_TOP || face == FACE_BOTTOM {
            25
        } else {
            26
        }
    } else if block_id == TNT {
        if face == FACE_TOP {
            27
        } else if face == FACE_BOTTOM {
            28
        } else {
            29
        }
    } else if block_id == BOOKSHELF {
        if face == FACE_TOP || face == FACE_BOTTOM {
            5 // planks
        } else {
            30 // bookshelf
        }
    } else if block_id == LAVA {
        31
    } else {
        1
    }
}

/// Directional lighting multiplier matching Minecraft Java Edition's classic vertex shading:
/// Top (+Y) = 1.0, North/South (+/-Z) = 0.8, East/West (+/-X) = 0.6, Bottom (-Y) = 0.5
#[inline(always)]
pub fn face_shade(face: i32) -> f32 {
    if face == FACE_TOP {
        1.0
    } else if face == FACE_SOUTH || face == FACE_NORTH {
        0.82
    } else if face == FACE_EAST || face == FACE_WEST {
        0.65
    } else {
        0.50
    }
}
