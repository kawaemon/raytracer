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
    ox: f64,
    oy: f64,
    oz: f64,
    // 球の中心座標
    cx: f64,
    cy: f64,
    cz: f64,
    // 球の半径
    r: f64,
    y: u32,
}

impl Drawer {
    fn new() -> Self {
        Self {
            ox: 0.0,
            oy: 0.0,
            oz: 5.0,
            cx: 0.0,
            cy: 0.0,
            cz: 0.0,
            r: 1.0,
            y: 0,
        }
    }

    fn initialize(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
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

    fn calc_pixel_color(&self, x: u32, y: u32) -> Color {
        // 投影面までの距離
        let distance_to_image_plane: i64 = HEIGHT as _;

        // ピクセルに対する一次レイの方向
        let (x, y) = (x as f64, y as f64);
        let (width, height) = (WIDTH as f64, HEIGHT as f64);

        let dx: f64 = x + 0.5 - width / 2.0;
        let dy: f64 = -(y + 0.5 - height / 2.0);
        let dz: f64 = (-distance_to_image_plane) as f64;

        if Self::intersectRaySphere(
            self.ox, self.oy, self.oz, dx, dy, dz, self.cx, self.cy, self.cz, self.r
        ) {
            Color::RGB(255, 255, 255)
        } else {
            Color::RGB(0, 0, 0)
        }
    }

    fn intersectRaySphere(
        ox: f64,
        oy: f64,
        oz: f64,
        dx: f64,
        dy: f64,
        dz: f64,
        cx: f64,
        cy: f64,
        cz: f64,
        r: f64
    ) -> bool {
        let a = sq(dx) + sq(dy) + sq(dz);
        let b = 2.0 * (dx * (ox - cx) + dy * (oy - cy) + dz * (oz - cz));
        let c = sq(ox - cx) + sq(oy - cy) + sq(oz - cz) - sq(r);

        let d = b * b - 4.0 * a * c;
        return 0.0 <= d;
    }
}

fn sq(f: f64) -> f64 {
    f * f
}
