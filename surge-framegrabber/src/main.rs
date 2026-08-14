use std::{
    io::{Write, stdin, stdout},
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use anyhow::Result;
use cameraunit_asi::{CameraUnit, CameraUnitASI, ROI, open_first_camera};
use risio::prelude::*;
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

const CROPPED_WIDTH: usize = 200;
const THRESH: u16 = 100;
fn main() -> Result<()> {
    let stdin = stdin();
    //setting up stdout and going into raw mode
    let mut stdout = stdout().into_raw_mode().unwrap();
    //printing welcoming message, clearing the screen and going to left top corner with the cursor
    write!(
        stdout,
        r#"{}{}q to exit,"#,
        termion::cursor::Goto(1, 1),
        termion::clear::All
    )
    .unwrap();
    stdout.flush().unwrap();

    let nc = cameraunit_asi::num_cameras();
    if nc < 1 {
        return Ok(());
    }

    let (grabber_tx, grabber_rx) = mpsc::channel::<bool>();
    let (cropper_tx, cropper_rx) = mpsc::channel::<bool>();
    let (cam, _caminfo) = open_first_camera()?;
    let grabber = std::thread::spawn(move || frame_grabber(cam, grabber_rx));
    let cropper = std::thread::spawn(move || cropper(cropper_rx));
    //detecting keydown events
    for c in stdin.keys() {
        if grabber.is_finished() {
            // println!("grabber finished");
            cropper_tx.send(true)?;
        }
        if cropper.is_finished() {
            // println!("cropper finished");
            grabber_tx.send(true)?;
        }
        if grabber.is_finished() && cropper.is_finished() {
            grabber.join().unwrap()?;
            cropper.join().unwrap()?;
            break;
        }
        //clearing the screen and going to top left corner
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, 1),
            termion::clear::All
        )
        .unwrap();

        //i reckon this speaks for itself
        match c.unwrap() {
            Key::Ctrl('c') | Key::Char('q') => {
                cropper_tx.send(true)?;
                grabber_tx.send(true)?;
            }
            _ => (),
        }

        stdout.flush().unwrap();
    }
    Ok(())
}

fn frame_grabber(mut cam: CameraUnitASI, rx: Receiver<bool>) -> Result<()> {
    let mut image: ShmImage<u16> = match ShmImage::create_new("raw", &[540, 960]) {
        Err(_) => ShmImage::open("raw").unwrap(),
        Ok(im) => im,
    };
    cam.set_exposure(Duration::from_millis(1000))?;
    cam.set_roi(&ROI {
        x_min: 0,
        y_min: 0,
        width: 3840,
        height: 2160,
        bin_x: 4,
        bin_y: 4,
    })?;
    // cam.set_gain(1000.0)?;
    loop {
        cam.start_exposure()?;
        loop {
            if let Ok(true) = rx.try_recv() {
                return Ok(());
            }
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
        let pixels: Vec<u16> = red
            .iter()
            .zip(green)
            .zip(blue)
            .map(|((r, g), b)| *r as u16 + *g as u16 + *b as u16)
            .collect();
        unsafe { image.modify(|(i, v)| *v = pixels[i])? };
        unsafe { image.sem_post_all() };
    }
}

fn cropper(rx: Receiver<bool>) -> Result<()> {
    let mut image: ShmImage<u16> = ShmImage::open("raw").unwrap();
    let width = image.metadata().size[1] as usize;
    let height = image.metadata().size[0] as usize;
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
        if let Ok(true) = rx.try_recv() {
            return Ok(());
        }
        unsafe { image.sem_wait(0) };
        let array = unsafe { image.array() };
        for x in 0..width {
            for y in 0..height {
                let val = array[x + width * y].saturating_sub(THRESH) as f32;
                cog_x += val * x as f32;
                cog_y += val * y as f32;
                cog_sum += val;
            }
        }
        cog_x /= cog_sum + 1e-5;
        cog_y /= cog_sum + 1e-5;
        cog_x -= CROPPED_WIDTH as f32 / 2.0;
        cog_y -= CROPPED_WIDTH as f32 / 2.0;
        let x0: usize = 0.max(cog_x as usize).min(width - CROPPED_WIDTH - 1);
        let y0: usize = 0.max(cog_y as usize).min(height - CROPPED_WIDTH - 1);
        for (i, v) in cropped_array.iter_mut().enumerate() {
            let sx: usize = i % CROPPED_WIDTH;
            let sy: usize = i / CROPPED_WIDTH;
            *v = array[x0 + sx + (y0 + sy) * width];
        }
    }
}
