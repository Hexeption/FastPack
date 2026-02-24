use image::GenericImageView;

use crate::{
    compressor::{CompressInput, CompressOutput, Compressor},
    error::CompressError,
};

// DDS header field constants.
const DDSD_CAPS: u32 = 0x1;
const DDSD_HEIGHT: u32 = 0x2;
const DDSD_WIDTH: u32 = 0x4;
const DDSD_PIXELFORMAT: u32 = 0x1000;
const DDSD_LINEARSIZE: u32 = 0x80000;
const DDPF_FOURCC: u32 = 0x4;
const DDSCAPS_TEXTURE: u32 = 0x1000;

/// DXT1 / BC1 compressor. No per-pixel alpha. Output is a DDS file.
///
/// Uses a fast min/max bounding-box endpoint selector. Quality is suitable
/// for game atlases; it is not the highest-quality iterative cluster-fit
/// approach, but produces valid BC1 blocks with no external dependencies.
pub struct Dxt1Compressor;

impl Compressor for Dxt1Compressor {
    fn compress(&self, input: &CompressInput<'_>) -> Result<CompressOutput, CompressError> {
        let (w, h) = input.image.dimensions();
        let rgb = input.image.to_rgb8();
        let blocks = encode_bc1(rgb.as_raw(), w, h);
        Ok(CompressOutput {
            data: build_dds(w, h, b"DXT1", &blocks),
        })
    }

    fn format_id(&self) -> &'static str {
        "dxt1"
    }

    fn file_extension(&self) -> &'static str {
        "dds"
    }
}

/// DXT5 / BC3 compressor. Full alpha channel. Output is a DDS file.
///
/// Alpha is encoded with the BC3 6-interpolation alpha block; colour is
/// encoded with BC1 in 4-colour mode.
pub struct Dxt5Compressor;

impl Compressor for Dxt5Compressor {
    fn compress(&self, input: &CompressInput<'_>) -> Result<CompressOutput, CompressError> {
        let (w, h) = input.image.dimensions();
        let rgba = input.image.to_rgba8();
        let blocks = encode_bc3(rgba.as_raw(), w, h);
        Ok(CompressOutput {
            data: build_dds(w, h, b"DXT5", &blocks),
        })
    }

    fn format_id(&self) -> &'static str {
        "dxt5"
    }

    fn file_extension(&self) -> &'static str {
        "dds"
    }
}

fn write_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn build_dds(w: u32, h: u32, fourcc: &[u8; 4], blocks: &[u8]) -> Vec<u8> {
    let mut dds = Vec::with_capacity(4 + 124 + blocks.len());
    dds.extend_from_slice(b"DDS ");
    write_u32(&mut dds, 124); // dwSize
    write_u32(
        &mut dds,
        DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT | DDSD_LINEARSIZE,
    ); // dwFlags
    write_u32(&mut dds, h); // dwHeight
    write_u32(&mut dds, w); // dwWidth
    write_u32(&mut dds, blocks.len() as u32); // dwPitchOrLinearSize
    write_u32(&mut dds, 0); // dwDepth
    write_u32(&mut dds, 1); // dwMipMapCount
    for _ in 0..11 {
        write_u32(&mut dds, 0); // dwReserved1[11]
    }
    // DDS_PIXELFORMAT (32 bytes)
    write_u32(&mut dds, 32); // dwSize
    write_u32(&mut dds, DDPF_FOURCC); // dwFlags
    dds.extend_from_slice(fourcc); // dwFourCC
    for _ in 0..5 {
        write_u32(&mut dds, 0); // dwRGBBitCount, dwRBitMask, dwGBitMask, dwBBitMask, dwABitMask
    }
    write_u32(&mut dds, DDSCAPS_TEXTURE); // dwCaps
    for _ in 0..4 {
        write_u32(&mut dds, 0); // dwCaps2, dwCaps3, dwCaps4, dwReserved2
    }
    dds.extend_from_slice(blocks);
    dds
}

fn encode_bc1(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let bw = w.div_ceil(4);
    let bh = h.div_ceil(4);
    let mut out = Vec::with_capacity((bw * bh * 8) as usize);
    for by in 0..bh {
        for bx in 0..bw {
            let block = extract_block_rgb(rgb, w, h, bx * 4, by * 4);
            out.extend_from_slice(&encode_bc1_block(&block));
        }
    }
    out
}

fn encode_bc3(rgba: &[u8], w: u32, h: u32) -> Vec<u8> {
    let bw = w.div_ceil(4);
    let bh = h.div_ceil(4);
    let mut out = Vec::with_capacity((bw * bh * 16) as usize);
    for by in 0..bh {
        for bx in 0..bw {
            let block = extract_block_rgba(rgba, w, h, bx * 4, by * 4);
            out.extend_from_slice(&encode_bc3_alpha_block(&block));
            let rgb_block: [[u8; 3]; 16] =
                std::array::from_fn(|i| [block[i][0], block[i][1], block[i][2]]);
            out.extend_from_slice(&encode_bc1_block(&rgb_block));
        }
    }
    out
}

fn extract_block_rgb(rgb: &[u8], img_w: u32, img_h: u32, bx: u32, by: u32) -> [[u8; 3]; 16] {
    let mut block = [[0u8; 3]; 16];
    for y in 0..4u32 {
        for x in 0..4u32 {
            let px = (bx + x).min(img_w - 1) as usize;
            let py = (by + y).min(img_h - 1) as usize;
            let i = py * img_w as usize + px;
            block[(y * 4 + x) as usize] = [rgb[i * 3], rgb[i * 3 + 1], rgb[i * 3 + 2]];
        }
    }
    block
}

fn extract_block_rgba(rgba: &[u8], img_w: u32, img_h: u32, bx: u32, by: u32) -> [[u8; 4]; 16] {
    let mut block = [[0u8; 4]; 16];
    for y in 0..4u32 {
        for x in 0..4u32 {
            let px = (bx + x).min(img_w - 1) as usize;
            let py = (by + y).min(img_h - 1) as usize;
            let i = py * img_w as usize + px;
            block[(y * 4 + x) as usize] = [
                rgba[i * 4],
                rgba[i * 4 + 1],
                rgba[i * 4 + 2],
                rgba[i * 4 + 3],
            ];
        }
    }
    block
}

fn rgb_to_565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3)
}

fn rgb_from_565(v: u16) -> [u8; 3] {
    let r5 = (v >> 11) as u8;
    let g6 = ((v >> 5) & 0x3f) as u8;
    let b5 = (v & 0x1f) as u8;
    [
        (r5 << 3) | (r5 >> 2),
        (g6 << 2) | (g6 >> 4),
        (b5 << 3) | (b5 >> 2),
    ]
}

fn color_dist_sq(a: [u8; 3], b: [u8; 3]) -> u32 {
    let dr = a[0] as i32 - b[0] as i32;
    let dg = a[1] as i32 - b[1] as i32;
    let db = a[2] as i32 - b[2] as i32;
    (dr * dr + dg * dg + db * db) as u32
}

fn encode_bc1_block(pixels: &[[u8; 3]; 16]) -> [u8; 8] {
    let (rmin, rmax, gmin, gmax, bmin, bmax) = pixels.iter().fold(
        (255u8, 0u8, 255u8, 0u8, 255u8, 0u8),
        |(rn, rx, gn, gx, bn, bx), p| {
            (
                rn.min(p[0]),
                rx.max(p[0]),
                gn.min(p[1]),
                gx.max(p[1]),
                bn.min(p[2]),
                bx.max(p[2]),
            )
        },
    );

    let mut c0 = rgb_to_565(rmax, gmax, bmax);
    let mut c1 = rgb_to_565(rmin, gmin, bmin);

    // 4-colour mode requires c0 > c1 as raw u16 values.
    if c0 < c1 {
        std::mem::swap(&mut c0, &mut c1);
    } else if c0 == c1 {
        // Degenerate: all pixels the same colour. Adjust to force 4-colour mode
        // so index 3 never resolves to transparent.
        if c0 > 0 {
            c1 -= 1;
        } else {
            c0 += 1;
        }
    }

    let col0 = rgb_from_565(c0);
    let col1 = rgb_from_565(c1);
    let col2 = [
        ((2 * col0[0] as u32 + col1[0] as u32 + 1) / 3) as u8,
        ((2 * col0[1] as u32 + col1[1] as u32 + 1) / 3) as u8,
        ((2 * col0[2] as u32 + col1[2] as u32 + 1) / 3) as u8,
    ];
    let col3 = [
        ((col0[0] as u32 + 2 * col1[0] as u32 + 1) / 3) as u8,
        ((col0[1] as u32 + 2 * col1[1] as u32 + 1) / 3) as u8,
        ((col0[2] as u32 + 2 * col1[2] as u32 + 1) / 3) as u8,
    ];

    let mut indices = 0u32;
    for (i, p) in pixels.iter().enumerate() {
        let d0 = color_dist_sq(*p, col0);
        let d1 = color_dist_sq(*p, col1);
        let d2 = color_dist_sq(*p, col2);
        let d3 = color_dist_sq(*p, col3);
        let idx = if d0 <= d1 && d0 <= d2 && d0 <= d3 {
            0u32
        } else if d1 <= d2 && d1 <= d3 {
            1
        } else if d2 <= d3 {
            2
        } else {
            3
        };
        indices |= idx << (i * 2);
    }

    let mut out = [0u8; 8];
    out[0..2].copy_from_slice(&c0.to_le_bytes());
    out[2..4].copy_from_slice(&c1.to_le_bytes());
    out[4..8].copy_from_slice(&indices.to_le_bytes());
    out
}

fn encode_bc3_alpha_block(pixels: &[[u8; 4]; 16]) -> [u8; 8] {
    let a0 = pixels.iter().map(|p| p[3]).max().unwrap_or(255);
    let a1 = pixels.iter().map(|p| p[3]).min().unwrap_or(0);

    // 6-interpolation mode (a0 > a1): 6 evenly-spaced alphas between a0 and a1.
    let refs: [u8; 8] = if a0 > a1 {
        [
            a0,
            a1,
            ((6 * a0 as u16 + a1 as u16 + 3) / 7) as u8,
            ((5 * a0 as u16 + 2 * a1 as u16 + 3) / 7) as u8,
            ((4 * a0 as u16 + 3 * a1 as u16 + 3) / 7) as u8,
            ((3 * a0 as u16 + 4 * a1 as u16 + 3) / 7) as u8,
            ((2 * a0 as u16 + 5 * a1 as u16 + 3) / 7) as u8,
            ((a0 as u16 + 6 * a1 as u16 + 3) / 7) as u8,
        ]
    } else {
        // All pixels share the same alpha; index 0 maps to a0 for all.
        [a0; 8]
    };

    let mut bits: u64 = 0;
    for (i, p) in pixels.iter().enumerate() {
        let a = p[3];
        let idx = refs
            .iter()
            .enumerate()
            .min_by_key(|&(_, r)| (*r as i16 - a as i16).unsigned_abs())
            .map(|(idx, _)| idx)
            .unwrap_or(0) as u64;
        bits |= idx << (i * 3);
    }

    let mut out = [0u8; 8];
    out[0] = a0;
    out[1] = a1;
    out[2..8].copy_from_slice(&bits.to_le_bytes()[0..6]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bc1_block_solid_colour() {
        let pixels: [[u8; 3]; 16] = [[255, 0, 0]; 16];
        let block = encode_bc1_block(&pixels);
        // c0 must be > c1 (4-colour mode).
        let c0 = u16::from_le_bytes([block[0], block[1]]);
        let c1 = u16::from_le_bytes([block[2], block[3]]);
        assert!(c0 > c1, "4-colour mode: c0({c0}) must be > c1({c1})");
    }

    #[test]
    fn bc3_alpha_block_full_range() {
        let mut pixels = [[0u8; 4]; 16];
        for (i, p) in pixels.iter_mut().enumerate() {
            p[3] = (i * 17) as u8;
        }
        let block = encode_bc3_alpha_block(&pixels);
        assert_eq!(block[0], 255); // a0 = max
        assert_eq!(block[1], 0); // a1 = min
    }

    #[test]
    fn dds_header_magic_and_size() {
        let blocks = vec![0u8; 8];
        let dds = build_dds(4, 4, b"DXT1", &blocks);
        assert_eq!(&dds[0..4], b"DDS ");
        assert_eq!(u32::from_le_bytes(dds[4..8].try_into().unwrap()), 124);
    }

    #[test]
    fn dxt1_output_byte_count() {
        let compressor = Dxt1Compressor;
        let img = image::DynamicImage::new_rgb8(8, 8);
        let out = compressor
            .compress(&CompressInput {
                image: &img,
                pack_mode: fastpack_core::types::config::PackMode::Fast,
                quality: 0,
            })
            .unwrap();
        // 8×8 → 2×2 blocks × 8 bytes = 32 bytes data + 128 bytes DDS header.
        assert_eq!(out.data.len(), 4 + 124 + 4 * 8);
    }

    #[test]
    fn dxt5_output_byte_count() {
        let compressor = Dxt5Compressor;
        let img = image::DynamicImage::new_rgba8(4, 4);
        let out = compressor
            .compress(&CompressInput {
                image: &img,
                pack_mode: fastpack_core::types::config::PackMode::Fast,
                quality: 0,
            })
            .unwrap();
        // 4×4 → 1 block × 16 bytes + 128 bytes DDS header.
        assert_eq!(out.data.len(), 4 + 124 + 16);
    }
}
