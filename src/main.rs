use std::time::Instant;

use glam::Vec3;
use pixels::{Error, PixelsBuilder, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod camera;
mod model;

use camera::*;
use model::*;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const RAYS: usize = 2;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 * 3.0);
        WindowBuilder::new()
            .with_title("Ray tracing")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
            .enable_vsync(true)
            .build()?
    };

    let mut camera = Camera::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));
    let mut models = Vec::new();

    models.push(Sphere::new(
        Vec3::new(0.0, 5.0, 20.0),
        5.0,
        Vec3::new(1.0, 0.25, 0.25),
        Vec3::splat(1.0),
        0.2,
    ));
    models.push(Sphere::new(
        Vec3::new(10.0, 5.0, 20.0),
        5.0,
        Vec3::new(0.25, 1.0, 0.25),
        Vec3::splat(1.0),
        0.2,
    ));
    models.push(Sphere::new(
        Vec3::new(0.0, 12.0, 15.0),
        2.0,
        Vec3::splat(0.0),
        Vec3::splat(1.0),
        1.0,
    ));

    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Event::MainEventsCleared = event {
            println!("fps: {:.1}", 1.0 / last_frame.elapsed().as_secs_f32());
            last_frame = Instant::now();
            draw(pixels.frame_mut(), &camera, &models);
            if let Err(err) = pixels.render() {
                println!("{err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    println!("{err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            if input.key_held(VirtualKeyCode::W) {
                camera.position -= camera.direction.normalize();
                camera.reset = true;
            }
            if input.key_held(VirtualKeyCode::A) {
                camera.position += camera.right.normalize();
                camera.reset = true;
            }
            if input.key_held(VirtualKeyCode::S) {
                camera.position += camera.direction.normalize();
                camera.reset = true;
            }
            if input.key_held(VirtualKeyCode::D) {
                camera.position -= camera.right.normalize();
                camera.reset = true;
            }
            if input.key_held(VirtualKeyCode::Space) {
                camera.position.y += 1.0;
                camera.reset = true;
            }
            if input.key_held(VirtualKeyCode::LShift) {
                camera.position.y -= 1.0;
                camera.reset = true;
            }

            let mut x_offset = None;
            let mut y_offset = None;
            if input.key_held(VirtualKeyCode::Up) {
                y_offset = Some(-1.0);
            }
            if input.key_held(VirtualKeyCode::Left) {
                x_offset = Some(-1.0);
            }
            if input.key_held(VirtualKeyCode::Down) {
                y_offset = Some(1.0);
            }
            if input.key_held(VirtualKeyCode::Right) {
                x_offset = Some(1.0);
            }

            if x_offset.is_some() || y_offset.is_some() {
                if let Some(x_offset) = x_offset {
                    camera.yaw += x_offset;
                }
                if let Some(y_offset) = y_offset {
                    camera.pitch += y_offset;
                }

                camera.direction.x =
                    camera.yaw.to_radians().cos() * camera.pitch.to_radians().cos();
                camera.direction.y = camera.pitch.to_radians().sin();
                camera.direction.z =
                    camera.yaw.to_radians().sin() * camera.pitch.to_radians().cos();
                camera.right = Vec3::new(0.0, 1.0, 0.0).cross(camera.direction).normalize();
                camera.up = camera.direction.cross(camera.right);
                camera.reset = true;
            }
        }
    });
}

fn get_ray(x: usize, y: usize, near_clip: f32, fov: f32) -> Vec3 {
    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let plane_height = near_clip * (fov * 0.5).to_radians().tan() * 2.0;
    let plane_width = plane_height * aspect_ratio;

    let tx = x as f32 / (WIDTH as f32 - 1.0);
    let ty = y as f32 / (HEIGHT as f32 - 1.0);

    Vec3::new(
        -plane_width / 2.0 + plane_width * tx,
        -plane_height / 2.0 + plane_height * ty,
        near_clip,
    )
}

fn draw(frame: &mut [u8], camera: &Camera, models: &[Sphere]) {
    for x in 0..WIDTH as usize {
        for y in 0..HEIGHT as usize {
            // TODO Rays could be cached
            let mut origin = Vec3::splat(0.0);
            let mut ray = get_ray(x, y, 2.0, 85.0);
            let mut color = Vec3::splat(1.0);
            let mut light = Vec3::splat(0.2);

            for _ in 0..RAYS {
                let mut hit_model = None;
                let mut hit_distance = None;
                for model in models.iter() {
                    if let Some(distance) = model.intersection(origin, ray, camera) {
                        if hit_distance.is_none() || distance < hit_distance.unwrap() {
                            hit_model = Some(model);
                            hit_distance = Some(distance);
                        }
                    }
                }
                if hit_distance.is_none() {
                    break;
                }
                let hit_model = hit_model.unwrap();
                let hit_distance = hit_distance.unwrap();
                let hit_point = origin + hit_distance * ray.normalize();

                light += hit_model.emission_color * hit_model.emission * color;
                color *= hit_model.color;

                if let Some(r) = hit_model.reflection(ray, hit_point, camera) {
                    ray = r;
                    origin = hit_point;
                } else {
                    break;
                }
            }

            draw_pixel(frame, x, y, &light);
        }
    }
}

fn draw_pixel(frame: &mut [u8], x: usize, y: usize, color: &Vec3) {
    let offset = y * 4 * WIDTH as usize + 4 * x;
    frame[offset] = (color.x * 255.0) as u8;
    frame[offset + 1] = (color.y * 255.0) as u8;
    frame[offset + 2] = (color.z * 255.0) as u8;
    frame[offset + 3] = 0xff;
}
