use std::time::Duration;

pub use rusttype::Font;

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

pub use sdl2::event::Event;
pub use sdl2::rect::Rect;

use crate::Point;

pub struct SimpleImage {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl SimpleImage {
    pub fn new(width: u32, height: u32) -> Self {
        let len = 4*width as usize*height as usize;
        SimpleImage {
            data: (0..len).map(|_| 0).collect(),
            width,
            height,
        }
    }

    pub fn create_text_image(
        font: &Font<'static>,
        text: &str,
        size: f32,
        color: [u8; 3],
    ) -> Result<SimpleImage, Box<dyn std::error::Error>> {
        let scale = rusttype::Scale::uniform(size);
        let point = rusttype::point(0.0, 0.0);
        let glyphs: Vec<_> = font.layout(text, scale, point).collect();
        let (y_min, y_max, width) = glyphs.iter()
            .filter_map(|glyph| glyph.pixel_bounding_box())
            .fold((0i32, 0i32, 0i32), |(y_min, y_max, width), bbox| {
                (
                    y_min.min(bbox.min.y),
                    y_max.max(bbox.max.y),
                    width.max(bbox.max.x)
                )
            });
        let height = y_max - y_min;
        let mut buffer = SimpleImage::new(width as u32, height as u32);

        for glyph in &glyphs {
            let bbox = match glyph.pixel_bounding_box() {
                Some(x) => x,
                None => continue,
            };

            glyph.draw(|x, y, w| {
                let x = x as i32 + bbox.min.x;
                let y = y as i32 + bbox.min.y - y_min;
                buffer[(x as u32, y as u32)] = [
                    color[2],
                    color[1],
                    color[0],
                    (255.0*w) as u8,
                ];
            });
        }

        Ok(buffer)
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }

    pub fn draw_polygon(&mut self, polygon: &[Point], value: [u8; 4])
    {
        let lines: Vec<(Point, Point)> = polygon.iter()
            .copied()
            .zip(polygon.iter().copied().cycle().skip(1))
            .collect();

        let mut top = f64::MAX;
        let mut bottom = 0.0f64;
        for (p0, p1) in lines.iter() {
            top = top.min(p0.y).min(p1.y);
            bottom = bottom.max(p0.y).max(p1.y);
        }

        let mut y = top;
        while y < bottom {
            let mut intersections = vec![];
            for (mut a, mut b) in lines.iter().copied() {
                if a.y == b.y {
                    continue;
                }

                if a.y > b.y {
                    (a, b) = (b, a);
                }
                if y < a.y || y > b.y {
                    continue;
                }

                if a.x == b.x {
                    intersections.push(a.x as i32);
                    continue;
                }

                let k = (b.y - a.y)/(b.x - a.x);
                let m = a.y - k*a.x;
                let x = (y as f64 - m)/k;
                intersections.push(x as i32);
            }

            intersections.sort();

            for (x0, x1) in intersections.iter().copied()
                .zip(intersections.iter().copied().skip(1))
                .enumerate()
                .filter_map(|(i, l)| if i % 2 == 0 { Some(l) } else { None }) {

                for x in x0..=x1 {
                    self[(x as u32, y as u32)] = value;
                }
            }

            y += 0.5;
        }

        for (p0, p1) in lines.iter() {
            let start = (p0.x as i32, p0.y as i32);
            let stop = (p1.x as i32, p1.y as i32);
            for (x, y) in line_drawing::Bresenham::new(start, stop) {
                self[(x as u32, y as u32)] = [0, 0, 0, 0xFF].into();
            }
        }
    }
}

impl std::ops::Deref for SimpleImage {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data.as_slice()
    }
}

impl std::ops::Index<(u32, u32)> for SimpleImage {
    type Output = [u8; 4];

    fn index(&self, (x, y): (u32, u32)) -> &Self::Output {
        let idx = 4*y*self.width + 4*x;
        self.data[idx as usize..idx as usize + 4].try_into().unwrap()
    }
}

impl std::ops::IndexMut<(u32, u32)> for SimpleImage {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Self::Output {
        let idx = 4*y*self.width + 4*x;
        (&mut self.data[idx as usize..idx as usize + 4]).try_into().unwrap()
    }
}

pub struct SimpleWindow<S> {
    context: sdl2::Sdl,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    width: u32,
    height: u32,
    state: S,
}

impl<S> SimpleWindow<S> {
    pub fn new(width: u32, height: u32, state: S) -> Result<SimpleWindow<S>, Box<dyn std::error::Error>> {
        let context = sdl2::init()?;
        let video_subsystem = context.video()?;

        let window = video_subsystem.window("rust-sdl2 demo", width, height)
            .position_centered()
            .build()?;

        let canvas = window.into_canvas().build()?;

        Ok(SimpleWindow {
            context,
            canvas,
            width,
            height,
            state,
        })
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn state(&self) -> &S { &self.state }
    pub fn state_mut(&mut self) -> &mut S { &mut self.state }

    pub fn draw_image(
        &mut self,
        image: &SimpleImage,
        target: Option<sdl2::rect::Rect>,
        blend: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let texture_creator = self.canvas.texture_creator();

        let mut texture = texture_creator.create_texture_static(
            sdl2::pixels::PixelFormatEnum::ARGB8888,
            image.width(),
            image.height(),
        )?;
        if blend {
            texture.set_blend_mode(sdl2::render::BlendMode::Blend);
        }
        texture.update(None, &*image, 4*image.width() as usize)?;

        self.canvas.copy(
            &texture,
            None,
            target,
        )?;

        Ok(())
    }

    pub fn stroke_circle(
        &mut self,
        cx: f64,
        cy: f64,
        radius: f64,
        thickness: f64,
        color: [u8; 4],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let x0 = (cx - radius).max(0.0);
        let y0 = (cy - radius).max(0.0);
        let x1 = (cx + radius).min(self.width as f64);
        let y1 = (cy + radius).min(self.height as f64);

        let r0 = radius - thickness;
        let r1 = radius;

        self.canvas.set_draw_color((color[0], color[1], color[2], color[3]));

        let mut x = x0;
        while x < x1 {
            let mut y = y0;
            while y < y1 {
                let r = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                if r < r0 || r > r1 {
                    y += 1.0;
                    continue;
                }

                self.canvas.draw_point((x as i32, y as i32))?;

                y += 1.0;
            }
            x += 1.0;
        }

        Ok(())
    }

    pub fn run<F>(mut self, callback: F) -> Result<(), Box<dyn std::error::Error>>
        where F: Fn(&mut SimpleWindow<S>, Vec<sdl2::event::Event>) -> Result<(), Box<dyn std::error::Error>>,
    {
        self.canvas.set_draw_color(Color::RGBA(0, 0, 0, 0xFF));
        self.canvas.clear();
        self.canvas.present();
        let mut event_pump = self.context.event_pump()?;
        'running: loop {
            self.canvas.clear();

            let mut events = vec![];
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => events.push(event),
                }
            }

            callback(&mut self, events)?;

            self.canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(())
    }
}
