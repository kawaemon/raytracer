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

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;
const FPS: u64 = 1;
const SAMPLES: u32 = 500;
const WORKERS: usize = 8;
const GUI_SAMPLE_STEP: u32 = 1;
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
                        canvas.draw_point(Point::new(x as _, y as _));
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
    eye: Vector3,
    width: u32,
    height: u32,
    samples: u32,
}

impl Drawer {
    fn new(width: u32, height: u32) -> Self {
        let mut scene = Scene::new();

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
                ..Material::default()
            },
        });

        scene.add_object(Plane::new(
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
        ));

        Self {
            scene: Arc::new(scene),
            eye: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 7.0,
            },
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

    fn calc_primary_ray(eye: Vector3, x: f64, y: f64) -> Ray {
        let (width, height) = (WIDTH as f64, HEIGHT as f64);
        let image_plane = height;

        let dx = x + random(0.0, 1.0) - width / 2.0;
        let dy = -(y + random(0.0, 1.0) - height / 2.0);
        let dz = -image_plane;

        Ray::new(
            eye,
            Vector3 {
                x: dx,
                y: dy,
                z: dz,
            }
            .normalize(),
        )
    }

    fn sample(&mut self, samples: u32) {
        let time = Instant::now();
        let current_height = Arc::new(Mutex::new(0));
        let mut handles = Vec::with_capacity(WORKERS);

        for _ in 0..WORKERS {
            let canvas_width = self.width;
            let canvas_height = self.height;
            let eye = self.eye;
            let scene = Arc::clone(&self.scene);
            let canvas = Arc::clone(&self.canvas);
            let current_height = Arc::clone(&current_height);

            handles.push(std::thread::spawn(move || loop {
                let render_range = {
                    let mut current_height = current_height.lock().unwrap();

                    if *current_height > canvas_height {
                        break;
                    }

                    let range =
                        *current_height..(*current_height + WORKERS_STEP).min(canvas_height);
                    *current_height += WORKERS_STEP;

                    range
                };

                let mut results = Vec::with_capacity((canvas_height * WORKERS_STEP) as usize);

                for _ in 0..samples {
                    for y in render_range.clone() {
                        for x in 0..canvas_width {
                            let primary_ray = Self::calc_primary_ray(eye, x as _, y as _);

                            let result = scene.trace(primary_ray, 0);
                            results.push((((y * canvas_height) + x), result));
                        }
                    }
                }

                let mut canvas_lock = canvas.lock().unwrap();
                for (index, result) in results {
                    canvas_lock[index as usize] += result;
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = time.elapsed().as_millis();

        self.samples += samples;

        println!("{} sample took {}ms", samples, elapsed);
    }

    fn pixels<'a>(&'a self) -> Vec<Spectrum> {
        self.canvas
            .lock()
            .unwrap()
            .iter()
            .map(|x| x.scale(1.0 / (self.samples as f64).max(1.0)))
            .collect()
    }
}
