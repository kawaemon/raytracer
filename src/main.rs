mod spectrum;
mod vector;
use spectrum::Spectrum;
use vector::Vector3;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};

use std::time::Instant;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;
const FPS: u64 = 30;
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

struct Drawer {
    eye: Vector3<f64>,
    sphere_center: Vector3<f64>,
    sphere_radius: f64,
    light_pos: Vector3<f64>,
    light_power: Spectrum,
    light2_pos: Vector3<f64>,
    light2_power: Spectrum,
    diffuse_color: Spectrum,
    y: u32,
}

impl Drawer {
    fn new() -> Self {
        Self {
            eye: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 5.0,
            },
            sphere_center: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            sphere_radius: 1.0,
            light_pos: Vector3 {
                x: 10.0,
                y: 10.0,
                z: 10.0,
            },
            light_power: Spectrum {
                r: 4000.0,
                g: 4000.0,
                b: 4000.0,
            },
            light2_pos: Vector3 {
                x: -10.0,
                y: -10.0,
                z: -10.0,
            },
            light2_power: Spectrum {
                r: 4000.0,
                g: 4000.0,
                b: 4000.0,
            },
            diffuse_color: Spectrum {
                r: 1.0,
                g: 0.5,
                b: 0.25,
            },
            y: 0,
        }
    }

    fn initialize(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                canvas.set_draw_color(self.calc_pixel_color(x, y));
                canvas.draw_point(Point::new(x as _, y as _));
            }
        }

        Ok(())
    }

    fn draw(&mut self, _canvas: &mut Canvas<Window>) -> Result<(), String> {
        Ok(())
    }

    fn calc_primary_ray(&self, x: u32, y: u32) -> Vector3<f64> {
        let distance_to_image_plane: i64 = HEIGHT as _;

        let (x, y) = (x as f64, y as f64);
        let (width, height) = (WIDTH as f64, HEIGHT as f64);

        let dx: f64 = x + 0.5 - width / 2.0;
        let dy: f64 = -(y + 0.5 - height / 2.0);
        let dz: f64 = (-distance_to_image_plane) as f64;

        Vector3 {
            x: dx,
            y: dy,
            z: dz,
        }
        .normalize()
    }

    fn calc_pixel_color(&self, x: u32, y: u32) -> Color {
        let primary_ray = self.calc_primary_ray(x, y);
        let t = self.intersect_ray_sphere(
            self.eye,
            primary_ray,
            self.sphere_center,
            self.sphere_radius,
        );

        if let Some(t) = t {
            let p = self.eye + primary_ray.scale(t);
            let n = (p - self.sphere_center).normalize();

            (self.diffuse_lighting(p, n, self.diffuse_color, self.light_pos, self.light_power)
                + self.diffuse_lighting(
                    p,
                    n,
                    self.diffuse_color,
                    self.light2_pos,
                    self.light2_power,
                ))
            .to_color()
        } else {
            Color::RGB(0, 0, 0)
        }
    }

    fn intersect_ray_sphere(
        &self,
        ray_origin: Vector3<f64>,
        ray_dir: Vector3<f64>,
        sphere_center: Vector3<f64>,
        _r: f64,
    ) -> Option<f64> {
        let v = ray_origin - sphere_center;

        let a = ray_dir.dot(&v);
        let b = v.dot(&v) - self.sphere_radius.powi(2);

        let d = a * a - b;

        if d >= 0.0 {
            let s = d.sqrt();
            let mut t = -a - s;

            if t <= 0.0 {
                t = -a + s;
            }

            if 0.0 < t {
                return Some(t);
            }
        }

        None
    }

    fn diffuse_lighting(
        &self,
        p: Vector3<f64>,
        n: Vector3<f64>,
        diffuse_color: Spectrum,
        light_pos: Vector3<f64>,
        light_power: Spectrum,
    ) -> Spectrum {
        let v = light_pos - p;
        let l = v.normalize();

        let dot = n.dot(&l);

        if dot > 0.0 {
            let r = v.len();
            let factor = dot / (4.0 * std::f64::consts::PI * r * r);

            light_power.scale(factor) * diffuse_color
        } else {
            spectrum::BLACK
        }
    }
}
