use anyhow::Result;
use fitrs::{Fits, Hdu};
use risio::prelude::*;

fn main() -> Result<()> {
    let mut image: ShmImage<u8> = ShmImage::open("raw")?;
    unsafe { image.sem_flush(1) };
    let mut data: Vec<f32> = vec![];
    let nframes = 50;
    for i in 0..nframes {
        unsafe { image.sem_wait(1) };
        println!("frame {i}");
        let array = unsafe { image.array() };
        data.append(&mut array.iter().map(|x| *x as f32).collect());
    }
    let shape = [image.metadata().size[1] as usize, image.metadata().size[0] as usize, 3, nframes];
    let mut hdu = Hdu::new(&shape, data);
    hdu.insert("test", "result of test");
    Fits::create("framedump.fits", hdu)?;
    Ok(())
}
