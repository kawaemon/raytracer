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
use light::Light;
use material::Material;
use plane::Plane;
use ray::Ray;
use scene::Scene;
use spectrum::Color;
use spectrum::Spectrum;
use sphere::Sphere;
use textured_obj::TexturedObj;
use vector::Vector3;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color as SDLColor, PixelFormatEnum},
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;
const FPS: u64 = 1;
const SAMPLES: u32 = 100;

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

        for count in 0..samples {
            println!("sampling {}/{}", count, samples);
            drawer.sample();
        }

        println!("writing into ./rendered.png");

        let path = Path::new("./rendered.png");
        let file = File::create(path).unwrap();
        let mut writer = BufWriter::new(file);

        let mut encoder = png::Encoder::new(writer, WIDTH, HEIGHT);
        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        let mut data = vec![];

        for c in drawer
            .pixels()
            .map(|x| x.to_color())
        {
            data.push(c.r);
            data.push(c.g);
            data.push(c.b);
            data.push(255);
        }

        writer.write_image_data(data.as_slice()).unwrap();

        return Ok(())
    }

    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    let window = video
        .window("raytracing", WIDTH, HEIGHT)
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

        let time = Instant::now();
        drawer.sample();
        let elapsed = time.elapsed().as_millis();

        println!("sampling took {}ms", elapsed);

        canvas
            .with_texture_canvas(&mut texture, |canvas| {
                let mut iter = drawer.pixels();
                for x in 0..WIDTH {
                    for y in 0..HEIGHT {
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

struct Drawer<'obj> {
    scene: Scene<'obj>,
    eye: Vector3,
    width: u32,
    height: u32,
    samples: u32,
    canvas: Vec<Spectrum>,
}

impl Drawer<'_> {
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
            scene,
            eye: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 7.0,
            },
            width,
            height,
            samples: 0,
            canvas: vec![
                Spectrum {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0
                };
                (width * height) as usize
            ],
        }
    }

    fn calc_primary_ray(&self, x: f64, y: f64) -> Ray {
        let (width, height) = (WIDTH as f64, HEIGHT as f64);
        let image_plane = height;

        let dx = x + 0.5 - width / 2.0;
        let dy = -(y + 0.5 - height / 2.0);
        let dz = -image_plane;

        Ray::new(
            self.eye,
            Vector3 {
                x: dx,
                y: dy,
                z: dz,
            }
            .normalize(),
        )
    }

    fn sample(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                let index = (x * self.width) + y;
                let primary_ray = self.calc_primary_ray(x as _, y as _);

                self.canvas[index as usize] += self.scene.trace(primary_ray, 0);
            }
        }

        self.samples += 1;
    }

    fn pixels<'a>(&'a self) -> impl Iterator<Item = Spectrum> + 'a {
        let samples = self.samples;
        self.canvas
            .iter()
            .map(move |x| x.scale(1.0 / (samples as f64).max(1.0)))
    }
}
