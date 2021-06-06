extern crate image;
extern crate rusttype;
extern crate termcolor;

use std::error::Error;
use std::ops::{Index, Mul};
use std::convert::TryInto;
use image::{RgbaImage, Rgba, Primitive, GenericImageView, Pixel, Luma};
use rusttype::{Font, Scale};
use termcolor::{Color, ColorChoice, ColorSpec, WriteColor};

pub mod char_maps {
    pub const DEFAULT: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
    pub const SOLID: [char; 1] = ['█'];
}

#[derive(Copy, Clone, Debug)]
pub struct Cell {
    pub character: char,
    pub colour: Rgba<u8>,
}

pub struct AsciiArt {
    pub cells: Vec<Cell>,
    pub dimensions: (usize, usize)
}

impl AsciiArt {
    pub fn print(&self) -> Result<(), Box<dyn Error>> {
        let mut stdout = termcolor::StandardStream::stdout(ColorChoice::Always);

        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                let cell = self[(x, y)];
                let mut stdout = termcolor::StandardStream::stdout(ColorChoice::Always);

                let colour = Some(Color::Rgb(cell.colour[0] as u8, cell.colour[1] as u8, cell.colour[2] as u8));

                stdout.set_color(&ColorSpec::new().set_fg(colour))?;

                print!("{}", cell.character);

                if x == self.dimensions.0 - 1 {
                    println!();
                }
            }
        }

        stdout.reset()?;

        Ok(())
    }

    pub fn to_image(&self, font: Font, font_size: usize, background_colour: Option<Rgba<u8>>)
        -> Result<RgbaImage, Box<dyn Error>> {
        let font_scale = Scale::uniform(font_size as f32);
        let image_dimensions = (
            self.dimensions.0 * font_size,
            self.dimensions.1 * font_size
        );

        let mut image = if let Some(background_colour) = background_colour {
            RgbaImage::from_pixel(image_dimensions.0 as u32, image_dimensions.1 as u32, background_colour)
        } else {
            RgbaImage::new(image_dimensions.0 as u32, image_dimensions.1 as u32)
        };

        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                let mut cell = self[(x, y)];

                font.glyph(cell.character)
                    .scaled(font_scale)
                    .positioned(rusttype::point((x * font_size) as f32, (y * font_size) as f32))
                    .draw(|glyph_x, glyph_y, coverage| {
                        let pixel_position = (
                            glyph_x as usize + (x * font_size),
                            glyph_y as usize + (y * font_size),
                        );

                        cell.colour[3] = (u8::MAX as f32 * coverage) as u8;

                        if pixel_position.0 < image_dimensions.0 && pixel_position.1 < image_dimensions.1 {
                            image.put_pixel(pixel_position.0 as u32, pixel_position.1 as u32, cell.colour);
                        }
                    });
            }
        }

        Ok(image)
    }
}

impl Index<(usize, usize)> for AsciiArt {
    type Output = Cell;
    
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.cells[index.0 + self.dimensions.0 * index.1]
    }
}

impl ToString for AsciiArt {
    fn to_string(&self) -> String {
        let mut string = String::with_capacity(self.dimensions.0 * self.dimensions.1);

        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                let cell = self[(x, y)];

                string.push(cell.character);
            }

            string.push_str("\n");
        }

        string
    }
}

pub struct AsciiArtBuilder<'a, S, P, I>
    where S: Primitive,
          P: Pixel<Subpixel = S>,
          I: GenericImageView<Pixel = P>, {
    image: &'a I,
    sample_dimensions: (usize, usize),
    char_map: Vec<char>,
}

impl<'a, S, P, I> AsciiArtBuilder<'a, S, P, I>
    where S: 'static + Primitive,
          P: Pixel<Subpixel = S>,
          I: GenericImageView<Pixel = P> {
    const DEFAULT_SAMPLE_DIMENSIONS: (usize, usize) = (10, 10);
    
    pub fn sample_dimensions(mut self, sample_dimensions: (usize, usize)) -> Self {
        self.sample_dimensions = sample_dimensions;
        self
    }
    
    pub fn char_map(mut self, char_map: Vec<char>) -> Self {
        self.char_map = char_map;
        self
    }

    pub fn build(self) -> AsciiArt {
        let image_dimensions = self.image.dimensions();

        let ascii_dimensions = (
            image_dimensions.0 as usize / self.sample_dimensions.0,
            image_dimensions.1 as usize / self.sample_dimensions.1,
        );

        let mut cells = Vec::new();

        for y in 0..ascii_dimensions.1 {
            for x in 0..ascii_dimensions.0 {
                let image_sample = self.image.view(
                    (x * self.sample_dimensions.0) as u32,
                    (y * self.sample_dimensions.1) as u32,
                    self.sample_dimensions.0 as u32,
                    self.sample_dimensions.1 as u32,
                );

                let average_pixel = Self::pixel_average(image_sample).unwrap();

                let cell_character = self.pixel_to_char(average_pixel.to_luma());
                let cell_rgb_channels: [u8; 4] = average_pixel.to_rgba().channels()
                    .into_iter()
                    .map(|x| x.to_u8().unwrap())
                    .collect::<Vec<u8>>()
                    .try_into()
                    .expect("Failed to extract colour from pixel");

                cells.push(
                    Cell {
                        character: cell_character,
                        colour: Rgba(cell_rgb_channels)
                    }
                );
            }
        }

        AsciiArt {
            cells,
            dimensions: ascii_dimensions
        }
    }
    
    fn pixel_average<T>(image: T) -> Result<P, Box<dyn Error>>
        where T: GenericImageView<Pixel = P> {
        let pixels = image.pixels();
        let pixel_count = image.width() * image.height();
        let mut pixels = pixels.peekable();
        let average_pixel = pixels.peek().ok_or("Error peeking")?.2;
        let mut average_pixel_channels = [S::zero(); 4];

        for (_, _, pixel) in pixels {
            let pixel_channels = pixel.channels();

            for i in 0..pixel_channels.len() {
                average_pixel_channels[i] =
                    S::from(
                        (average_pixel_channels[i])
                            .to_usize()
                            .unwrap()
                            .saturating_add(pixel_channels[i].to_usize().unwrap() / pixel_count as usize)
                    ).unwrap();
            }
        }

        Ok(average_pixel)
    }

    fn pixel_to_char<T>(&self, luma_pixel: Luma<T>) -> char
        where T: Primitive + Mul<Output = T> {
        self.char_map[
            (luma_pixel[0].to_usize().unwrap() as f32 * self.char_map.len() as f32 / (u8::MAX as f32 + 1.0).floor()) as usize
        ]
    }
}

impl<'a, S, P, I> From<&'a I> for AsciiArtBuilder<'a, S, P, I>
    where S: 'static + Primitive,
          P: Pixel<Subpixel = S>,
          I: GenericImageView<Pixel = P> {
    fn from(image: &'a I) -> Self {
        Self {
            image,
            sample_dimensions: Self::DEFAULT_SAMPLE_DIMENSIONS,
            char_map: Vec::from(char_maps::DEFAULT),
        }
    }
}