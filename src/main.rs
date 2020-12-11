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
    println!("initialize took {}ms", time.elapsed().as_millis());

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
    eye: Vector3<f64>,
}

impl Drawer<'_> {
    fn new() -> Self {
        let mut scene = Scene::new();

        scene.add_object(Sphere {
            center: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            radius: 1.0,
            material: Material {
                diffuse: Spectrum {
                    r: 0.1,
                    g: 0.5,
                    b: 0.9,
                },
                refractive: 0.9,
                refractive_index: 1.5,
                ..Material::default()
            },
        });

        let floor1_material = Material {
            diffuse: Spectrum {
                r: 0.5,
                g: 0.5,
                b: 0.5,
            },
            ..Material::default()
        };

        let floor2_material = Material {
            diffuse: Spectrum {
                r: 0.2,
                g: 0.2,
                b: 0.2,
            },
            ..Material::default()
        };

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
                floor1_material.clone(),
            ),
            grid_width: 1.0,
            alt_material: floor2_material.clone()
        });

        let decoder = png::Decoder::new(File::open("./wall.png").unwrap());
        let (info, mut reader) = decoder.read_info().unwrap();
        let mut buffer = vec![0; info.buffer_size()];
        reader.next_frame(&mut buffer).unwrap();
        let (width, height) = (info.width, info.height);

        scene.add_object(TexturedObj {
            object: Plane::new(
                Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: -5.0,
                },
                Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
                floor1_material,
            ),
            image: move |x, y| {
                let base_pos = ((y * width + x) * 4) as usize;

                let r = buffer[base_pos];
                let g = buffer[base_pos + 1];
                let b = buffer[base_pos + 2];

                Spectrum {
                    r: r as f64 / std::u8::MAX as f64,
                    g: g as f64 / std::u8::MAX as f64,
                    b: b as f64 / std::u8::MAX as f64,
                }
            },
            texture_size: 10.0,
            image_width: width,
            image_height: height,
            origin: Vector3 {
                x: -5.0,
                y: -5.0,
                z: -5.0,
            },
            u_direction: Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            v_direction: Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        });

        scene.add_light(Light {
            pos: Vector3 {
                x: 100.0,
                y: 100.0,
                z: 100.0,
            },
            power: Spectrum {
                r: 800_000.0,
                g: 800_000.0,
                b: 800_000.0,
            },
        });

        Self {
            scene,
            eye: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 4.0,
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
            },
        )
    }

    fn initialize(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                canvas.set_draw_color(
                    self.scene
                        .trace(self.calc_primary_ray(x as _, y as _), 0)
                        .to_color(),
                );
                canvas.draw_point(Point::new(x as _, y as _))?;
            }
        }

        Ok(())
    }

    fn draw(&mut self, _canvas: &mut Canvas<Window>) -> Result<(), String> {
        Ok(())
    }
}
