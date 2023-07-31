use alloc::vec::Vec;
use lazy_static::lazy_static;

const PSF_MAGIC: u32 = 0x864ab572;
pub const FALLBACK_CHAR: [u8; 16] = [
    0xFF, 0xFF, 0xFF, 0xFF, 
    0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF
];

pub fn unpack_psf(psf: &[u8]) -> Font {
    let magic: u32 = psf.chunks(4).map(
        |chunk| {
            u32::from_le_bytes(chunk.try_into().unwrap())
        }
    ).next().expect("Font file is empty");

    match magic {
        PSF_MAGIC => {
            unpack_psf2(psf)
        },
        _ => panic!("Received a non-PSF2 font")
    }
}

/// Unpacks a PSF 2 font into character bitmaps
/// Before you call this, make sure the magic number matches to avoid later parsing errors
fn unpack_psf2(psf: &[u8]) -> Font {
    // Header Structure: 
    //  magic number: u32,
    //  version: u32
    //  header_size: u32,
    //  flags: u32,
    //  num_glyphs: u32,
    //  bytes_per_glyph: u32,
    //  width: u32,
    //  height: u32,;
    let psf_header: Vec<usize> = psf.chunks(4).map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()) as usize).take(8).collect();

    // from the header, we only care about header size, number of glyphs, bytes per glyph, and width
    let header_size = psf_header[2];
    let num_glyphs = psf_header[4];
    let bytes_per_glyph = psf_header[5];
    let width = psf_header[6];

    // bitmap data starts after `header_size` bytes
    let psf_data: Vec<u8> = psf.iter().skip(header_size).map(|v| *v).collect();
    let glyphs: Vec<Vec<u8>> = psf_data.chunks(bytes_per_glyph).take(num_glyphs).map(|glyph| glyph.to_vec()).collect();

    Font {
        glyphs,
        width: width / 8, // the `width` header field is in bits
    }
}

pub struct Font {
    glyphs: Vec<Vec<u8>>,
    pub width: usize,
}

impl Font {
    pub fn get_char(&self, character: char) -> Option<&[u8]> {
        // if character.is_ascii() {
        //     Some(self.glyphs[character as usize].as_slice())
        // } else {
        //     None
        // }

        None
    }
}