use std::time::Duration;

use anyhow::Result;
use cameraunit_asi::{CameraUnit, CameraUnitASI, ROI, open_first_camera};
use risio::prelude::*;

fn main() -> Result<()> {
    let nc = cameraunit_asi::num_cameras();
    if nc < 1 {
        return Ok(());
    }
    let mut dropped = 0;
    loop {
        println!("dropped: {dropped}");
        dropped += 1;
        let (cam, _caminfo) = open_first_camera()?;
        match frame_grabber(cam) {
            _ => {
                std::thread::sleep(Duration::from_millis(1000));
                continue;
            }
        };
    }
}

fn frame_grabber(mut cam: CameraUnitASI) -> Result<()> {
    let mut image: ShmImage<u8> = match ShmImage::create_new("raw", &[2160, 3840, 3]) {
        Err(_) => ShmImage::open("raw").unwrap(),
        Ok(im) => im,
    };
    cam.set_exposure(Duration::from_millis(10))?;
    cam.set_roi(&ROI {
        x_min: 0,
        y_min: 0,
        width: 3840,
        height: 2160,
        bin_x: 1,
        bin_y: 1,
    })?;
    cam.set_gain(60.0)?;
    loop {
        cam.start_exposure()?;
        loop {
            match cam.image_ready() {
                Ok(true) => break,
                Ok(false) => continue,
                Err(e) => return Err(e.into()),
            }
        }
        let img = cam.download_image()?;
        let red = img.as_u8().unwrap().get_red().unwrap();
        let green = img.as_u8().unwrap().get_green().unwrap();
        let blue = img.as_u8().unwrap().get_blue().unwrap();
        let mut pixels: Vec<u8> = red
            .to_vec();
        pixels.append(&mut green.to_vec());
        pixels.append(&mut blue.to_vec());
        unsafe { image.modify(|(i, v)| *v = pixels[i])? };
        unsafe { image.sem_post_all() };
    }
}
