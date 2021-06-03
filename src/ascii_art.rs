use std::error::Error;
use std::ops::Index;

use image::{RgbaImage, Rgba};
use rusttype::{Font, Scale};
use termcolor::{Color, ColorChoice, ColorSpec, WriteColor};

#[derive(Copy, Clone, Debug)]
pub struct Cell {
    pub character: char,
    pub colour: Rgba<u8>,
}

pub struct AsciiArt {
    pub cells: Vec<Cell>,
    pub dimensions: (usize, usize)
}

impl Index<(usize, usize)> for AsciiArt {
    type Output = Cell;
    
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.cells[index.0 + self.dimensions.0 * index.1]
    }
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

    pub fn as_image(&self, font: Font, font_size: usize, background_colour: Option<Rgba<u8>>)
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
                        cell.colour[3] = (u8::MAX as f32 * coverage) as u8;

                        image.put_pixel(
                            glyph_x + (x * font_size) as u32,
                            glyph_y + (y * font_size) as u32,
                            cell.colour
                        );
                    });
            }
        }

        Ok(image)
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