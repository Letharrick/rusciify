use crate::ascii_art::AsciiArt;
use crate::character_map::CharacterMap;
use crate::ascii_art::Cell;

use std::error::Error;
use std::convert::TryInto;

use image::{Primitive, GenericImageView, Pixel, Rgba};

pub struct AsciiArtBuilder<'a, S, P, I>
    where S: Primitive,
          P: Pixel<Subpixel = S>,
          I: GenericImageView<Pixel = P>, {
    image: &'a I,
    sample_dimensions: (usize, usize),
    character_map: CharacterMap,
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
    
    pub fn character_map(mut self, character_map: CharacterMap) -> Self {
        self.character_map = character_map;
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

                let cell_character = self.character_map[average_pixel.to_luma()];
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
}

impl<'a, S, P, I> From<&'a I> for AsciiArtBuilder<'a, S, P, I>
    where S: 'static + Primitive,
          P: Pixel<Subpixel = S>,
          I: GenericImageView<Pixel = P> {
    fn from(image: &'a I) -> Self {
        Self {
            image,
            sample_dimensions: Self::DEFAULT_SAMPLE_DIMENSIONS,
            character_map: CharacterMap::default(),
        }
    }
}