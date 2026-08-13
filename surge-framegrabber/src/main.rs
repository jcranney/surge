use risio::prelude::*;

fn main() {
    let mut image: ShmImage<u8> = match ShmImage::create_new("raw_frame", &[360, 640]) {
        Err(_) => ShmImage::open("raw_frame").unwrap(),
        Ok(im) => im,
    };
    let array = unsafe { image.array_mut() };
    loop {
        todo!()
    }
}
