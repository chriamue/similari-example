use image::{DynamicImage, ImageBuffer, Rgb};
use nokhwa::ThreadedCamera;
use rocket::fs::FileServer;
use rocket::http::{ContentType, Status};
use rocket::serde::json::Json;
use rocket::State;
use rocket::{get, options, post, routes};
use serde::{Deserialize, Serialize};
use similari_example::Frame;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn callback(_image: ImageBuffer<Rgb<u8>, Vec<u8>>) {}

#[get("/frame")]
fn frame(frame: &'_ State<Arc<Mutex<Frame>>>) -> (Status, (ContentType, Vec<u8>)) {
    let frame = {
        let img = frame.lock().unwrap();
        let base_img: DynamicImage = DynamicImage::ImageRgb8(img.raw.clone());
        let mut buf = vec![];
        base_img
            .write_to(&mut buf, image::ImageOutputFormat::Jpeg(70))
            .unwrap();
        buf
    };
    (Status::Ok, (ContentType::JPEG, frame))
}

#[get("/tracked")]
fn tracked(frame: &'_ State<Arc<Mutex<Frame>>>) -> (Status, (ContentType, Vec<u8>)) {
    let frame = {
        let img = frame.lock().unwrap();
        let base_img: DynamicImage = DynamicImage::ImageRgb8(img.tracked.clone());
        let mut buf = vec![];
        base_img
            .write_to(&mut buf, image::ImageOutputFormat::Jpeg(70))
            .unwrap();
        buf
    };
    (Status::Ok, (ContentType::JPEG, frame))
}

#[derive(Serialize, Deserialize)]
struct ObjectCoords {
    id: u32,
    world_x: f32,
    world_y: f32,
    model_x: f32,
    model_y: f32,
}

#[derive(Deserialize)]
struct UpdatePositionRequest {
    id: u32,
    x: u32,
    y: u32,
}

#[post("/update-position", format = "application/json", data = "<request>")]
fn post_update_position(request: Json<UpdatePositionRequest>) {
    let data = request.into_inner();
    println!("{} {} {}", data.id, data.x, data.y);
}

#[options("/update-position")]
fn options_update_position() {}

async fn fetch_frame(frame: Arc<Mutex<Frame>>, webcam: Arc<Mutex<ThreadedCamera>>) {
    loop {
        {
            let image = webcam.lock().unwrap().last_frame();
            let mut frame = frame.lock().unwrap();
            let mut f = Frame::new(image);
            f.tracked = frame.tracked.clone();
            *frame = f;
        };
        thread::sleep(Duration::from_millis(40));
    }
}

#[tokio::main]
async fn main() {
    let frame = Arc::new(Mutex::new(Frame::default()));

    let mut webcam = ThreadedCamera::new(0, None).unwrap();
    webcam.open_stream(callback).unwrap();

    {
        let image = webcam.poll_frame().unwrap();
        let mut frame = frame.lock().unwrap();
        *frame = Frame::new(image);
    };

    let webcam = Arc::new(Mutex::new(webcam));

    let fetch_frame_thread = tokio::spawn(fetch_frame(frame.clone(), webcam.clone()));

    let launcher = rocket::build()
        .mount("/", FileServer::from("static"))
        .mount(
            "/",
            routes![
                frame,
                tracked,
                post_update_position,
                options_update_position
            ],
        )
        .manage(webcam)
        .manage(frame);
    let _server = launcher.launch().await.unwrap();
    fetch_frame_thread.abort();
}
