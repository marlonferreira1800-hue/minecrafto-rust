// Clean-room implementation of Minecraft Java Edition CraftingManager (Recipes.java)
#[cfg(target_arch = "wasm32")]
use crate::minicore::*;

use crate::blocks::*;

#[inline(always)]
fn grid_at(grid: &[u8; 9], idx: usize) -> u8 {
    unsafe { *(grid as *const [u8; 9] as *const u8).offset(idx as isize) }
}

/// Evaluates a 3x3 crafting grid (row-major 9 slots: g0..g8, 0 = empty).
/// Returns `(output_item_id as i32) | ((output_count as i32) << 8)` or 0 if no recipe matches.
pub fn evaluate_crafting_grid(
    g0: u8,
    g1: u8,
    g2: u8,
    g3: u8,
    g4: u8,
    g5: u8,
    g6: u8,
    g7: u8,
    g8: u8,
) -> i32 {
    let grid = [g0, g1, g2, g3, g4, g5, g6, g7, g8];

    let mut non_zero = 0;
    let mut single_id: u8 = 0;
    let mut i = 0;
    while i < 9 {
        let v = grid_at(&grid, i);
        if v != 0 {
            non_zero += 1;
            single_id = v;
        }
        i += 1;
    }

    if non_zero == 0 {
        return 0;
    }

    // 1. Single Wood Log anywhere -> 4x Oak Planks
    if non_zero == 1 && single_id == WOOD_LOG {
        return (PLANKS as i32) | (4 << 8);
    }

    // 2. Single Ore Smelt shortcut -> Coal, Iron Ingot, Gold Ingot, Diamond
    if non_zero == 1 && single_id == COAL_ORE {
        return (ITEM_COAL as i32) | (2 << 8);
    }
    if non_zero == 1 && single_id == IRON_ORE {
        return (ITEM_IRON_INGOT as i32) | (1 << 8);
    }
    if non_zero == 1 && single_id == GOLD_ORE {
        return (ITEM_GOLD_INGOT as i32) | (1 << 8);
    }
    if non_zero == 1 && single_id == DIAMOND_ORE {
        return (ITEM_DIAMOND as i32) | (1 << 8);
    }

    // 3. Two vertical Planks -> 4x Sticks; or Coal over Stick -> 4x Glowstone (Torch/Light)
    if non_zero == 2 {
        let mut r = 0;
        while r < 2 {
            let mut c = 0;
            while c < 3 {
                let top = grid_at(&grid, r * 3 + c);
                let bot = grid_at(&grid, (r + 1) * 3 + c);
                if top == PLANKS && bot == PLANKS {
                    return (ITEM_STICK as i32) | (4 << 8);
                }
                if (top == ITEM_COAL || top == COAL_ORE) && bot == ITEM_STICK {
                    return (GLOWSTONE as i32) | (4 << 8);
                }
                c += 1;
            }
            r += 1;
        }
    }

    // 4. 2x2 square patterns anywhere in the 3x3 grid
    if non_zero == 4 {
        let mut r = 0;
        while r < 2 {
            let mut c = 0;
            while c < 2 {
                let a = grid_at(&grid, r * 3 + c);
                let b = grid_at(&grid, r * 3 + c + 1);
                let d = grid_at(&grid, (r + 1) * 3 + c);
                let e = grid_at(&grid, (r + 1) * 3 + c + 1);
                if a != 0 && a == b && a == d && a == e {
                    if a == PLANKS {
                        return (CRAFTING_TABLE as i32) | (1 << 8);
                    }
                    if a == SAND {
                        return (GLASS as i32) | (4 << 8);
                    }
                    if a == STONE || a == COBBLESTONE {
                        return (BRICKS as i32) | (4 << 8);
                    }
                }
                // 2x2 Sand + Coal checkerboard -> 2x TNT
                if (a == SAND
                    && e == SAND
                    && (b == ITEM_COAL || b == COAL_ORE)
                    && (d == ITEM_COAL || d == COAL_ORE))
                    || ((a == ITEM_COAL || a == COAL_ORE)
                        && (e == ITEM_COAL || e == COAL_ORE)
                        && b == SAND
                        && d == SAND)
                {
                    return (TNT as i32) | (2 << 8);
                }
                c += 1;
            }
            r += 1;
        }
    }

    // 5. Pickaxe recipes (5 items: 3 across top row + 2 sticks down center column)
    if non_zero == 5 && g0 != 0 && g0 == g1 && g1 == g2 && g4 == ITEM_STICK && g7 == ITEM_STICK {
        if g0 == PLANKS {
            return (ITEM_WOOD_PICKAXE as i32) | (1 << 8);
        }
        if g0 == COBBLESTONE || g0 == STONE {
            return (ITEM_STONE_PICKAXE as i32) | (1 << 8);
        }
        if g0 == ITEM_IRON_INGOT || g0 == IRON_ORE {
            return (ITEM_IRON_PICKAXE as i32) | (1 << 8);
        }
        if g0 == ITEM_DIAMOND || g0 == DIAMOND_ORE {
            return (ITEM_DIAMOND_PICKAXE as i32) | (1 << 8);
        }
    }

    // 6. Furnace: 8x Cobblestone ring with empty center (g4 == 0)
    if non_zero == 8
        && g4 == 0
        && g0 == COBBLESTONE
        && g1 == COBBLESTONE
        && g2 == COBBLESTONE
        && g3 == COBBLESTONE
        && g5 == COBBLESTONE
        && g6 == COBBLESTONE
        && g7 == COBBLESTONE
        && g8 == COBBLESTONE
    {
        return (FURNACE as i32) | (1 << 8);
    }

    // 7. Bookshelf: 6x Planks (top & bottom rows) + 3x Wood/Planks in middle
    if non_zero == 9
        && g0 == PLANKS
        && g1 == PLANKS
        && g2 == PLANKS
        && g6 == PLANKS
        && g7 == PLANKS
        && g8 == PLANKS
    {
        return (BOOKSHELF as i32) | (2 << 8);
    }

    // 8. Full 3x3 TNT recipe (5 Sand + 4 Coal)
    if non_zero == 9
        && g0 == SAND
        && g2 == SAND
        && g4 == SAND
        && g6 == SAND
        && g8 == SAND
        && (g1 == ITEM_COAL || g1 == COAL_ORE)
        && (g3 == ITEM_COAL || g3 == COAL_ORE)
        && (g5 == ITEM_COAL || g5 == COAL_ORE)
        && (g7 == ITEM_COAL || g7 == COAL_ORE)
    {
        return (TNT as i32) | (4 << 8);
    }

    0
}
