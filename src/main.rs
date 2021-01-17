#![allow(dead_code)]

mod camera;
mod checked_obj;
mod intersect;
mod light;
mod material;
mod plane;
mod ray;
mod scene;
mod spectrum;
mod sphere;
mod textured_obj;
mod vector;

use checked_obj::CheckedObject;
use material::Material;
use plane::Plane;
use ray::Ray;
use scene::Scene;
use spectrum::Spectrum;
use sphere::Sphere;
use vector::Vector3;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color as SDLColor, PixelFormatEnum};
use sdl2::rect::{Point, Rect};

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::camera::Camera;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;
const FPS: u64 = 60;
const SAMPLES: u32 = 500;
const WORKERS: usize = 8;
const GUI_SAMPLE_STEP: u32 = 50;
const WORKERS_STEP: u32 = 4;

#[inline(always)]
fn random(low: f64, high: f64) -> f64 {
    let mut thread_rng = rand::thread_rng();

    SmallRng::from_rng(&mut thread_rng)
        .unwrap()
        .gen_range(low, high)
}

fn main() -> Result<(), String> {
    let mut drawer = Drawer::new(WIDTH, HEIGHT);

    let headless = std::env::var("RAYTRACER_HEADLESS").is_ok();

    if headless {
        println!("headless mode");

        let samples = match std::env::var("RAYTRACER_SAMPLES") {
            Ok(samples_str) => match samples_str.parse::<u32>() {
                Ok(v) => v,
                Err(e) => return Err(e.to_string()),
            },
            Err(_) => SAMPLES,
        };

        drawer.sample(samples);

        println!("writing into ./rendered.png");

        let path = Path::new("./rendered.png");
        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);

        let mut encoder = png::Encoder::new(writer, WIDTH, HEIGHT);
        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        let mut data = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);

        for c in drawer.pixels().iter().map(|x| x.to_color()) {
            data.push(c.r);
            data.push(c.g);
            data.push(c.b);
            data.push(255);
        }

        writer.write_image_data(data.as_slice()).unwrap();

        return Ok(());
    }

    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    let window = video
        .window("raytracer", WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_target(PixelFormatEnum::RGBA8888, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        drawer.sample(GUI_SAMPLE_STEP);

        canvas
            .with_texture_canvas(&mut texture, |canvas| {
                let mut iter = drawer.pixels().into_iter();
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        let color = iter.next().unwrap().to_color();

                        canvas.set_draw_color(SDLColor::RGB(color.r, color.g, color.b));
                        canvas.draw_point(Point::new(x as _, y as _)).unwrap();
                    }
                }
            })
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(SDLColor::RGB(0, 0, 0));
        canvas.clear();
        canvas.copy_ex(
            &texture,
            None,
            Some(Rect::new(0, 0, WIDTH as _, HEIGHT as _)),
            0.0,
            Some(Point::new(WIDTH as _, HEIGHT as _)),
            false,
            false,
        )?;

        canvas.present();

        std::thread::sleep(std::time::Duration::from_millis(1000 / FPS));
    }

    Ok(())
}

struct Drawer {
    scene: Arc<Scene>,
    canvas: Arc<Mutex<Vec<Spectrum>>>,
    camera: Camera,
    width: u32,
    height: u32,
    samples: u32,
}

impl Drawer {
    fn new(width: u32, height: u32) -> Self {
        let mut scene = Scene::new();

        scene.set_sky_color(Spectrum {
            r: 0.1,
            g: 0.1,
            b: 0.1,
        });

        scene.add_object(Sphere {
            center: Vector3 {
                x: -2.2,
                y: 0.0,
                z: 0.0,
            },
            radius: 1.0,
            material: Material {
                diffuse: Spectrum {
                    r: 0.7,
                    g: 0.3,
                    b: 0.9,
                },
                ..Material::default()
            },
        });

        scene.add_object(Sphere {
            center: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            radius: 1.0,
            material: Material {
                diffuse: Spectrum {
                    r: 0.9,
                    g: 0.7,
                    b: 0.3,
                },
                reflective: 0.8,
                ..Material::default()
            },
        });

        scene.add_object(Sphere {
            center: Vector3 {
                x: 2.2,
                y: 0.0,
                z: 0.0,
            },
            radius: 1.0,
            material: Material {
                diffuse: Spectrum {
                    r: 0.3,
                    g: 0.9,
                    b: 0.7,
                },
                refractive: 0.8,
                refractive_index: 1.5,
                ..Material::default()
            },
        });

        scene.add_object(Sphere {
            center: Vector3 {
                x: 0.0,
                y: 4.0,
                z: 0.0,
            },
            radius: 1.0,
            material: Material {
                diffuse: Spectrum {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                },
                emissive: Spectrum {
                    r: 30.0,
                    g: 20.0,
                    b: 10.0,
                },
                ..Material::default()
            },
        });

        scene.add_object(CheckedObject {
            object: Plane::new(
                Vector3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                },
                Vector3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
                Material {
                    diffuse: Spectrum {
                        r: 0.9,
                        g: 0.9,
                        b: 0.9,
                    },
                    ..Material::default()
                },
            ),
            grid_width: 1.0,
            alt_material: Material {
                diffuse: Spectrum {
                    r: 0.4,
                    g: 0.4,
                    b: 0.4,
                },
                ..Material::default()
            },
        });

        let mut camera = Camera::default();
        camera.look_at(
            Vector3 {
                x: 0.0,
                y: 0.0,
                z: 9.0,
            },
            Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            40.0 * std::f64::consts::PI / 180.0,
            width,
            height,
        );

        Self {
            scene: Arc::new(scene),
            camera,
            width,
            height,
            samples: 0,
            canvas: Arc::new(Mutex::new(vec![
                Spectrum {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0
                };
                (width * height) as usize
            ])),
        }
    }

    fn sample(&mut self, samples: u32) {
        let time = Instant::now();
        let current_height = Arc::new(Mutex::new(0));
        let mut handles = Vec::with_capacity(WORKERS);

        let worker = Worker {
            canvas_width: self.width,
            canvas_height: self.height,
            camera: self.camera.clone(),
            scene: Arc::clone(&self.scene),
            canvas: Arc::clone(&self.canvas),
            current_height: Arc::clone(&current_height),
        };

        for i in 0..WORKERS {
            let worker = worker.clone();
            let handle = std::thread::Builder::new()
                .name(format!("Render worker {}", i + 1))
                .stack_size(64 * 1024 * 1024)
                .spawn(move || worker.run(samples))
                .unwrap();

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = time.elapsed().as_millis();

        self.samples += samples;

        println!("{} sample took {}ms", samples, elapsed);
    }

    fn pixels(&self) -> Vec<Spectrum> {
        self.canvas
            .lock()
            .unwrap()
            .iter()
            .map(|x| x.scale(1.0 / (self.samples as f64).max(1.0)))
            .collect()
    }
}

#[derive(Clone)]
struct Worker {
    canvas_width: u32,
    canvas_height: u32,
    camera: Camera,
    scene: Arc<Scene>,
    canvas: Arc<Mutex<Vec<Spectrum>>>,
    current_height: Arc<Mutex<u32>>,
}

impl Worker {
    fn calc_primary_ray(&self, x: f64, y: f64) -> Ray {
        self.camera
            .ray(x + random(-0.5, 0.5), y + random(-0.5, 0.5))
    }

    fn run(&self, samples: u32) {
        loop {
            let render_range = {
                let mut current_height = self.current_height.lock().unwrap();

                if *current_height > self.canvas_height {
                    break;
                }

                let range =
                    *current_height..(*current_height + WORKERS_STEP).min(self.canvas_height);
                *current_height += WORKERS_STEP;

                range
            };

            let mut results = Vec::with_capacity((self.canvas_height * WORKERS_STEP) as usize);

            for _ in 0..samples {
                for y in render_range.clone() {
                    for x in 0..self.canvas_width {
                        let primary_ray = self.calc_primary_ray(x as _, y as _);

                        let result = self.scene.trace(primary_ray, 0);
                        results.push((((y * self.canvas_height) + x), result));
                    }
                }
            }

            let mut canvas_lock = self.canvas.lock().unwrap();
            for (index, result) in results {
                canvas_lock[index as usize] += result;
            }
        }
    }
}
