extern crate bdf;

use std::{env, fs::File, io::Write};

use bdf::{Bitmap, BoundingBox, Glyph};

fn main() {
    let input_file = env::args().nth(1).expect("missing font file");
    let output_file = env::args().nth(2).expect("missing output path");

    let range_start: u32 =
        u32::from_str_radix(&env::args().nth(3).expect("missing range_start"), 16)
            .expect("unkown range_start");
    let range_end: u32 = u32::from_str_radix(&env::args().nth(4).expect("missing range_end"), 16)
        .expect("unkown range_end");

    let scale: u32 = env::args()
        .nth(5)
        .unwrap_or("1".into())
        .parse()
        .expect("unkown scale");

    let font = bdf::open(input_file);
    let font = font.unwrap();

    let x_size = if font.bounds().width * scale % 8 == 0 {
        font.bounds().width * scale
    } else {
        (font.bounds().width * scale / 8 + 1) * 8
    };

    let mut file = File::create(output_file).unwrap();
    for i in range_start..range_end {
        let codepoint = char::from_u32(i);

        let glyph = if codepoint.is_none() {
            None
        } else {
            font.glyphs().get(&codepoint.unwrap())
        };

        let mut data = Vec::new();
        if glyph.is_none() {
            let offset = x_size / 8 * font.bounds().height * scale;
            for _ in 0..offset {
                data.push(0x00);
            }
            file.write_all(&data).unwrap();
            file.flush().unwrap();
            continue;
        }
        let glyph = glyph.unwrap();
        let mut bitmap = render_glyph(&glyph, font.bounds());
        bitmap = rotate_bitmap(&bitmap);

        if scale != 1 {
            bitmap = scale_bitmap(&bitmap, scale);
        }

        for y in 0..bitmap.height() {
            let mut line = Vec::new();
            let mut v = 0u8;

            for x in 0..x_size {
                let bit = if x >= bitmap.width() || y >= bitmap.height() {
                    0u8
                } else {
                    bitmap.get(x, y) as u8
                };
                v = v | (bit << (7 - (x % 8)));
                if (x + 1) % 8 == 0 {
                    line.push(v);
                    v = 0u8;
                }
            }
            data.append(&mut line);
        }
        file.write_all(&data).unwrap();
        file.flush().unwrap();
    }
}

fn rotate_bitmap(bitmap: &Bitmap) -> Bitmap {
    let mut rttd = Bitmap::new(bitmap.height(), bitmap.width());

    for y in 0..bitmap.height() {
        for x in 0..bitmap.width() {
            rttd.set(y, x, bitmap.get(x, y));
        }
    }

    rttd
}

fn scale_bitmap(bitmap: &Bitmap, times: u32) -> Bitmap {
    let mut rttd = Bitmap::new(bitmap.height() * times, bitmap.width() * times);

    for y in 0..bitmap.height() {
        for x in 0..bitmap.width() {
            for i in 0..times {
                for j in 0..times {
                    rttd.set(x * times + j, y * times + i, bitmap.get(x, y));
                }
            }
        }
    }

    rttd
}

fn render_glyph(glyph: &Glyph, fb: &BoundingBox) -> Bitmap {
    let mut bitmap = Bitmap::new(fb.width, fb.height);

    let gb = glyph.bounds();

    for y in 0..glyph.height() {
        for x in 0..glyph.width() {
            let yy: u32 = fb.height - y - 1;
            bitmap.set(
                (x as i32 + (gb.x - fb.x)) as u32,
                (yy as i32 - (gb.y - fb.y)) as u32,
                glyph.get(x, glyph.height() - y - 1),
            );
        }
    }

    bitmap
}
