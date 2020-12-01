mod vector;
use vector::Vector3;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};

use std::time::Duration;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

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

    canvas
        .with_texture_canvas(&mut texture, |canvas| {
            drawer.initialize(canvas).unwrap();
        })
        .map_err(|e| e.to_string())?;

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

        canvas
            .with_texture_canvas(&mut texture, |canvas| {
                drawer.draw(canvas).unwrap();
            })
            .map_err(|e| e.to_string())?;

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

        std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
    }

    Ok(())
}

struct Drawer {
    // 視点の座標
    eye: Vector3<f64>,
    // 球の中心座標
    sphere_center: Vector3<f64>,
    // 球の半径
    sphere_radius: f64,
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
            y: 0,
        }
    }

    fn initialize(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        Ok(())
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        if self.y >= HEIGHT {
            return Ok(());
        }

        for x in 0..WIDTH {
            canvas.set_draw_color(self.calc_pixel_color(x, self.y));
            canvas.draw_point(Point::new(x as _, self.y as _));
        }

        self.y += 1;

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

        if self.intersect_ray_sphere(
            self.eye,
            primary_ray,
            self.sphere_center,
            self.sphere_radius,
        ) {
            Color::RGB(255, 255, 255)
        } else {
            Color::RGB(0, 0, 0)
        }
    }

    fn intersect_ray_sphere(
        &self,
        ray_origin: Vector3<f64>,
        ray_dir: Vector3<f64>,
        sphere_center: Vector3<f64>,
        r: f64,
    ) -> bool {
        let v = ray_origin - sphere_center;

        let a = ray_dir.dot(&v);
        let b = v.dot(&v) - self.sphere_radius.powi(2);

        a * a - b >= 0.0
    }
}
