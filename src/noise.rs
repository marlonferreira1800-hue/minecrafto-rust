// Clean-room port of Minecraft Java Edition's ImprovedNoise / Perlin Octave generator
// (net.minecraft.world.level.levelgen.synth.ImprovedNoise)
#[cfg(target_arch = "wasm32")]
use crate::minicore::*;

pub const BIOME_PLAINS: u8 = 0;
pub const BIOME_FOREST: u8 = 1;
pub const BIOME_DESERT: u8 = 2;
pub const BIOME_SNOW_MOUNTAINS: u8 = 3;

#[inline(always)]
pub fn fast_floor(x: f32) -> i32 {
    let xi = x as i32;
    if x < (xi as f32) {
        xi - 1
    } else {
        xi
    }
}

/// Quintic Perlin fade curve: 6t^5 - 15t^4 + 10t^3
#[inline(always)]
pub fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline(always)]
pub fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

/// Deterministic spatial hash replacing the 512-byte permutation table for seeded worlds
#[inline(always)]
pub fn hash_coord(x: i32, y: i32, z: i32, seed: i32) -> i32 {
    let xm = (x & 0x7FFF) - 16384;
    let ym = (y & 0x7FFF) - 16384;
    let zm = (z & 0x7FFF) - 16384;
    let mut h = (seed & 0x7FFFFFFF) ^ (xm * 37471) ^ (ym * 66821) ^ (zm * 98249);
    h = (h ^ (h >> 13)) ^ ((h & 0x7FFF) * 48271);
    h ^ (h >> 11)
}

/// Classic Perlin 3D gradient dot product (matching ImprovedNoise.grad)
#[inline(always)]
pub fn grad3(hash: i32, x: f32, y: f32, z: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    let a = if (h & 1) == 0 { u } else { -u };
    let b = if (h & 2) == 0 { v } else { -v };
    a + b
}

#[inline(always)]
pub fn grad2(hash: i32, x: f32, z: f32) -> f32 {
    let h = hash & 7;
    let u = if h < 4 { x } else { z };
    let v = if h < 4 { z } else { x };
    let a = if (h & 1) == 0 { u } else { -u };
    let b = if (h & 2) == 0 { v } else { -v };
    a + b
}

/// 2D Perlin noise in range [-1.0, 1.0]
pub fn noise2d(x: f32, z: f32, seed: i32) -> f32 {
    let xi = fast_floor(x);
    let zi = fast_floor(z);
    let xf = x - (xi as f32);
    let zf = z - (zi as f32);

    let u = fade(xf);
    let v = fade(zf);

    let h00 = hash_coord(xi, 0, zi, seed);
    let h10 = hash_coord(xi + 1, 0, zi, seed);
    let h01 = hash_coord(xi, 0, zi + 1, seed);
    let h11 = hash_coord(xi + 1, 0, zi + 1, seed);

    let x1 = lerp(u, grad2(h00, xf, zf), grad2(h10, xf - 1.0, zf));
    let x2 = lerp(u, grad2(h01, xf, zf - 1.0), grad2(h11, xf - 1.0, zf - 1.0));

    lerp(v, x1, x2) * 0.70710678
}

/// 3D Perlin noise in range [-1.0, 1.0] for caves & 3D density overhang
pub fn noise3d(x: f32, y: f32, z: f32, seed: i32) -> f32 {
    let xi = fast_floor(x);
    let yi = fast_floor(y);
    let zi = fast_floor(z);

    let xf = x - (xi as f32);
    let yf = y - (yi as f32);
    let zf = z - (zi as f32);

    let u = fade(xf);
    let v = fade(yf);
    let w = fade(zf);

    let h000 = hash_coord(xi, yi, zi, seed);
    let h100 = hash_coord(xi + 1, yi, zi, seed);
    let h010 = hash_coord(xi, yi + 1, zi, seed);
    let h110 = hash_coord(xi + 1, yi + 1, zi, seed);
    let h001 = hash_coord(xi, yi, zi + 1, seed);
    let h101 = hash_coord(xi + 1, yi, zi + 1, seed);
    let h011 = hash_coord(xi, yi + 1, zi + 1, seed);
    let h111 = hash_coord(xi + 1, yi + 1, zi + 1, seed);

    let x00 = lerp(u, grad3(h000, xf, yf, zf), grad3(h100, xf - 1.0, yf, zf));
    let x10 = lerp(
        u,
        grad3(h010, xf, yf - 1.0, zf),
        grad3(h110, xf - 1.0, yf - 1.0, zf),
    );
    let x01 = lerp(
        u,
        grad3(h001, xf, yf, zf - 1.0),
        grad3(h101, xf - 1.0, yf, zf - 1.0),
    );
    let x11 = lerp(
        u,
        grad3(h011, xf, yf - 1.0, zf - 1.0),
        grad3(h111, xf - 1.0, yf - 1.0, zf - 1.0),
    );

    let y0 = lerp(v, x00, x10);
    let y1 = lerp(v, x01, x11);
    lerp(w, y0, y1)
}

/// Multi-octave Fractional Brownian Motion (fBm) matching PerlinNoise.getValue
pub fn fbm2d(x: f32, z: f32, octaves: i32, seed: i32) -> f32 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_val = 0.0;
    let mut i = 0;
    while i < octaves {
        total += noise2d(x * frequency, z * frequency, seed + i * 1013) * amplitude;
        max_val += amplitude;
        amplitude *= 0.5;
        frequency *= 2.02;
        i += 1;
    }
    total / max_val
}

/// Compute Biome at world (wx, wz) using temperature & humidity noise (BiomeSource.java)
pub fn get_biome(wx: i32, wz: i32, seed: i32) -> u8 {
    let fx = (wx as f32) * 0.0085;
    let fz = (wz as f32) * 0.0085;
    let temp = fbm2d(fx, fz, 2, seed + 777);
    let rain = fbm2d(fx + 50.0, fz - 50.0, 2, seed + 999);
    let relief = fbm2d((wx as f32) * 0.015, (wz as f32) * 0.015, 3, seed + 333);

    if relief > 0.26 || temp < -0.22 {
        BIOME_SNOW_MOUNTAINS
    } else if temp > 0.20 && rain < 0.05 {
        BIOME_DESERT
    } else if rain > 0.05 {
        BIOME_FOREST
    } else {
        BIOME_PLAINS
    }
}

/// Compute surface elevation Y at world (wx, wz) in range [12..54] (water level = 22)
pub fn get_surface_height(wx: i32, wz: i32, seed: i32) -> i32 {
    let fx = wx as f32;
    let fz = wz as f32;
    let continental = fbm2d(fx * 0.012, fz * 0.012, 4, seed);
    let detail = fbm2d(fx * 0.045, fz * 0.045, 2, seed + 42);
    let ridge_raw = noise2d(fx * 0.018, fz * 0.018, seed + 333);
    let ridge = if ridge_raw > 0.15 {
        (ridge_raw - 0.15) * 26.0
    } else {
        0.0
    };

    let h = 25.0 + continental * 14.0 + detail * 4.5 + ridge;
    let mut hi = fast_floor(h);
    if hi < 8 {
        hi = 8;
    }
    if hi > 54 {
        hi = 54;
    }
    hi
}
