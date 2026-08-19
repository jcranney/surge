use anyhow::Result;
use risio::prelude::*;

const CROPPED_WIDTH: usize = 200;
const THRESH: f32 = 200.0;
fn main() -> Result<()> {
    let mut image: ShmImage<u8> = ShmImage::open("raw").unwrap();
    let width = image.metadata().size[1] as usize;
    let height = image.metadata().size[0] as usize;
    let depth = image.metadata().size[2] as usize;
    let mut cropped_image: ShmImage<u16> =
        match ShmImage::create_new("cropped", &[CROPPED_WIDTH, CROPPED_WIDTH]) {
            Err(_) => ShmImage::open("cropped").unwrap(),
            Ok(im) => im,
        };
    let cropped_array = unsafe { cropped_image.array_mut() };
    unsafe { image.sem_flush(0) };
    loop {
        let mut cog_x = 0.0;
        let mut cog_y = 0.0;
        let mut cog_sum = 0.0;
        unsafe { image.sem_wait(0) };
        let array = unsafe { image.array() };
        let mut m: f32 = 0.0;
        for x in 0..width {
            for y in 0..height {
                let mut val: f32 = 0.0;
                for d in 0..depth {
                    val += array[x * depth + depth * width * y + d] as f32;
                }
                m = m.max(val);
                val -= THRESH;
                val = val.max(0.0);
                cog_x += val * x as f32;
                cog_y += val * y as f32;
                cog_sum += val;
            }
        }
        println!("max: {:?}, cog_sum: {:?}", m, cog_sum);
        cog_x /= cog_sum + 1e-5;
        cog_y /= cog_sum + 1e-5;
        cog_x -= CROPPED_WIDTH as f32 / 2.0;
        cog_y -= CROPPED_WIDTH as f32 / 2.0;
        let x0: usize = 0.max(cog_x as usize).min(width - CROPPED_WIDTH - 1);
        let y0: usize = 0.max(cog_y as usize).min(height - CROPPED_WIDTH - 1);
        for (i, v) in cropped_array.iter_mut().enumerate() {
            let sx: usize = i % CROPPED_WIDTH;
            let sy: usize = i / CROPPED_WIDTH;
            *v = 0;
            for d in 0..depth {
                *v = v.saturating_add(array[(x0 + sx) * depth + (y0 + sy) * width * depth + d] as u16);
            }
        }
    }
}
