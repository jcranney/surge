use libcamera::{
    camera::CameraConfigurationStatus,
    camera_manager::CameraManager,
    framebuffer::AsFrameBuffer,
    framebuffer_allocator::{FrameBuffer, FrameBufferAllocator},
    framebuffer_map::MemoryMappedFrameBuffer,
    pixel_format::PixelFormat,
    properties,
    request::ReuseFlag,
    stream::StreamRole,
};
use risio::prelude::*;
use std::time::Duration;

// drm-fourcc does not have MJPEG type yet, construct it from raw fourcc identifier

fn main() {
    let pixel_format_rgb: PixelFormat = PixelFormat::parse("R8").unwrap();
    let mut image: ShmImage<u8> = match ShmImage::create_new("raw_frame", &[360, 640]) {
        Err(_) => ShmImage::open("raw_frame").unwrap(),
        Ok(im) => im,
    };

    // let filename = match std::env::args().nth(1) {
    //     Some(f) => f,
    //     None => {
    //         println!("Error: missing file output parameter");
    //         println!("Usage: ./video_capture </path/to/output.mjpeg>");
    //         exit(1);
    //     }
    // };

    let mgr = CameraManager::new().unwrap();

    let cameras = mgr.cameras();

    let cam = cameras.get(1).expect("No cameras found");

    println!(
        "Using camera: {}",
        *cam.properties().get::<properties::Model>().unwrap()
    );

    let mut cam = cam.acquire().expect("Unable to acquire camera");

    // This will generate default configuration for each specified role
    let mut cfgs = cam.generate_configuration(&[StreamRole::Raw]).unwrap();

    // cfgs.get_mut(0)
    //     .unwrap()
    //     .set_pixel_format(PIXEL_FORMAT_MJPEG);

    cfgs.get_mut(0).unwrap().set_pixel_format(pixel_format_rgb);

    println!("Generated config: {cfgs:#?}");

    match cfgs.validate() {
        CameraConfigurationStatus::Valid => println!("Camera configuration valid!"),
        CameraConfigurationStatus::Adjusted => {
            println!("Camera configuration was adjusted: {cfgs:#?}")
        }
        CameraConfigurationStatus::Invalid => panic!("Error validating camera configuration"),
    }

    cam.configure(&mut cfgs)
        .expect("Unable to configure camera");

    let mut alloc = FrameBufferAllocator::new(&cam);

    // Allocate frame buffers for the stream
    let cfg = cfgs.get(0).unwrap();
    let stream = cfg.stream().unwrap();
    let buffers = alloc.alloc(&stream).unwrap();
    println!("Allocated {} buffers", buffers.len());

    // Convert FrameBuffer to MemoryMappedFrameBuffer, which allows reading &[u8]
    let buffers = buffers
        .into_iter()
        .map(|buf| MemoryMappedFrameBuffer::new(buf).unwrap())
        .collect::<Vec<_>>();

    // Create capture requests and attach buffers
    let reqs = buffers
        .into_iter()
        .enumerate()
        .map(|(i, buf)| {
            let mut req = cam.create_request(Some(i as u64)).unwrap();
            req.add_buffer(&stream, buf).unwrap();
            req
        })
        .collect::<Vec<_>>();

    // Completed capture requests are returned as a callback
    let (tx, rx) = std::sync::mpsc::channel();
    cam.on_request_completed(move |req| {
        tx.send(req).unwrap();
    });

    // TODO: Set `Control::FrameDuration()` here. Blocked on https://github.com/lit-robotics/libcamera-rs/issues/2
    cam.start(None).unwrap();

    // Enqueue all requests to the camera
    for req in reqs {
        println!("Request queued for execution: {req:#?}");
        cam.queue_request(req).map_err(|(_, e)| e).unwrap();
    }

    // Ensure that pixel format was unchanged
    assert_eq!(
        cfgs.get(0).unwrap().get_pixel_format(),
        pixel_format_rgb,
        "R8 is not supported by the camera"
    );

    let array = unsafe { image.array_mut() };
    loop {
        // println!("Waiting for camera request execution");
        // Allow extra time for slower pipelines/first frame startup.
        let mut req = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("Camera request failed");

        // println!("Camera request {req:?} completed!");
        // println!("Metadata: {:#?}", req.metadata());

        // Get framebuffer for our stream
        let framebuffer: &MemoryMappedFrameBuffer<FrameBuffer> = req.buffer(&stream).unwrap();
        // println!("FrameBuffer metadata: {:#?}", framebuffer.metadata());

        let planes = framebuffer.data();
        let frame_data = planes.first().unwrap();
        let bytes_used = framebuffer
            .metadata()
            .unwrap()
            .planes()
            .get(0)
            .unwrap()
            .bytes_used as usize;

        array.copy_from_slice(
            frame_data[..bytes_used]
                .to_vec()
                .into_iter()
                .rev()
                .collect::<Vec<u8>>()
                .as_slice(),
        );
        // println!("Written {} bytes to {}", bytes_used, &filename);

        // Recycle the request back to the camera for execution
        req.reuse(ReuseFlag::REUSE_BUFFERS);
        cam.queue_request(req).map_err(|(_, e)| e).unwrap();
    }

    // Everything is cleaned up automatically by Drop implementations
}
