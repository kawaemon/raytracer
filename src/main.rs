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
use spectrum::Spectrum;
use sphere::Sphere;
use textured_obj::TexturedObj;
use vector::Vector3;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};

use std::fs::File;
use std::time::Instant;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;
const FPS: u64 = 1;
const WARNING_THRESHOLD_MS: u128 = 100;
const SAMPLES: usize = 5;

fn main() -> Result<(), String> {
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
    let mut drawer = Drawer::new();

    let mut time = Instant::now();
    canvas
        .with_texture_canvas(&mut texture, |canvas| {
            drawer.initialize(canvas).unwrap();
        })
        .map_err(|e| e.to_string())?;
    println!("rendering took {}ms", time.elapsed().as_millis());

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

        time = Instant::now();
        canvas
            .with_texture_canvas(&mut texture, |canvas| {
                drawer.draw(canvas).unwrap();
            })
            .map_err(|e| e.to_string())?;
        let elapsed = time.elapsed().as_millis();
        if WARNING_THRESHOLD_MS <= elapsed {
            println!("drawing tooks longer than usual: {}ms", elapsed);
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
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
}

impl Drawer<'_> {
    fn new() -> Self {
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

    fn initialize(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let mut sum = spectrum::BLACK;
                let primary_ray = self.calc_primary_ray(x as _, y as _);

                for _ in 0..SAMPLES {
                    sum += self.scene.trace(primary_ray.clone(), 0);
                }

                canvas.set_draw_color(sum.scale(1.0 / (SAMPLES as f64)).to_color());
                canvas.draw_point(Point::new(x as _, y as _))?;
            }
            println!("{}/{}", x + 1, HEIGHT);
        }

        Ok(())
    }

    fn draw(&mut self, _canvas: &mut Canvas<Window>) -> Result<(), String> {
        Ok(())
    }
}
