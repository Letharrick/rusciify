extern crate image;
extern crate imageproc;
extern crate quicli;
extern crate rusttype;
extern crate structopt;
extern crate termcolor;

use image::{buffer::Pixels, DynamicImage, GenericImageView, ImageBuffer, Luma, Pixel, Rgba};
use quicli::prelude::CliResult;
use rusttype::{Font, Scale};
use structopt::StructOpt;
use termcolor::{Color, ColorChoice, ColorSpec, WriteColor};

const FONT_BYTES: &[u8] = include_bytes!("../assets/Courier.ttf");

#[derive(StructOpt, Debug)]
#[structopt(name = "Rusciify", about = "Convert images to ASCII")]
struct Rusciify {
    input_image: String,

    #[structopt(long = "output", short = "o", help = "Path to output image file")]
    output_image: Option<String>,

    #[structopt(
        long = "font-size",
        short = "f",
        default_value = "25.0",
        help = "The font size of the ASCII characters in the output image"
    )]
    font_size: f32,

    #[structopt(
        long = "sample-scale",
        short = "s",
        default_value = "5",
        help = "The scale of the NxN pixel sample that is used for ASCII conversion"
    )]
    sample_scale: u32,

    #[structopt(
        long = "colour",
        short = "c",
        help = "A custom colour for the ASCII output"
    )]
    colour: Vec<u8>,

    #[structopt(
        long = "map",
        short = "m",
        default_value = " .:-=+*#%@",
        help = "The ASCII mapping to use for converting samples to characters"
    )]
    map: String,
}

#[derive(Copy, Clone, Debug)]
struct Cell {
    position: (u32, u32),
    colour: Rgba<u8>,
}

impl Cell {
    fn as_ascii(&self, map: &[char]) -> char {
        let grayscale_colour = self.colour.to_luma();

        Ascii::pixel_to_ascii(&grayscale_colour, map)
    }
}

struct Ascii {
    cells: Vec<Cell>,
    dimensions: (u32, u32),
}

impl Ascii {
    fn new(image: &DynamicImage, sample_size: (u32, u32)) -> Self {
        let image_dimensions = image.dimensions();

        let ascii_dimensions = (
            image_dimensions.0 / sample_size.0,
            image_dimensions.1 / sample_size.1,
        );

        let mut cells = Vec::new();

        for y in 0..ascii_dimensions.1 {
            for x in 0..ascii_dimensions.0 {
                let sample = image
                    .view(
                        x * sample_size.0,
                        y * sample_size.1,
                        sample_size.0 as u32,
                        sample_size.1 as u32,
                    )
                    .to_image();

                let average_pixel = Ascii::average_pixels(sample.pixels()).unwrap();

                cells.push(Cell {
                    position: (x, y),
                    colour: average_pixel.to_rgba(),
                });
            }
        }

        Ascii {
            cells,
            dimensions: ascii_dimensions,
        }
    }

    fn print(&self, ascii_map: &[char], colour: Option<[u8; 4]>) {
        let mut stdout = termcolor::StandardStream::stdout(ColorChoice::Always);

        for cell in &self.cells {
            let mut stdout = termcolor::StandardStream::stdout(ColorChoice::Always);
            let colour = colour.unwrap_or(cell.colour.0);
            let colour = Some(Color::Rgb(colour[0], colour[1], colour[2]));

            stdout.set_color(&ColorSpec::new().set_fg(colour));

            print!("{}", cell.as_ascii(ascii_map));

            if cell.position.0 == self.dimensions.0 - 1 {
                println!();
            }
        }

        stdout.reset();
    }

    fn as_image(
        &self,
        ascii_map: &[char],
        colour: Option<[u8; 4]>,
        font: Font,
        font_size: f32,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
        let mut img = ImageBuffer::from_pixel(
            self.dimensions.0 * font_size as u32,
            self.dimensions.1 * font_size as u32,
            [0, 0, 0, 255].into(),
        );

        for cell in &self.cells {
            let colour = colour.unwrap_or(cell.colour.0);
            let colour = Rgba(colour);

            imageproc::drawing::draw_text_mut(
                &mut img,
                colour,
                cell.position.0 * font_size as u32,
                cell.position.1 * font_size as u32,
                Scale::uniform(font_size),
                &font,
                &cell.as_ascii(&ascii_map).to_string(),
            );
        }

        Ok(img)
    }

    fn average_pixels<P: Pixel<Subpixel = u8>>(
        pixels: Pixels<P>,
    ) -> Result<P, Box<dyn std::error::Error>> {
        let pixel_count = pixels.len();
        let mut pixels = pixels.peekable();
        let mut average_pixel: P = pixels.peek().ok_or("Error peeking")?.map(|p| p);
        let average_pixel_channels = average_pixel.channels_mut();

        for pixel in pixels {
            let pixel = pixel.to_rgba();
            let pixel_channels = pixel.channels();

            for i in 0..pixel_channels.len() {
                average_pixel_channels[i] =
                    average_pixel_channels[i].saturating_add(pixel_channels[i] / pixel_count as u8);
            }
        }

        Ok(average_pixel)
    }

    fn pixel_to_ascii(pixel: &Luma<u8>, map: &[char]) -> char {
        map[(pixel.channels()[0] as u32 * map.len() as u32 / 256) as usize]
    }
}

fn main() -> CliResult {
    let args = Rusciify::from_args();

    let sample_size = {
        if args.output_image.is_some() {
            (args.sample_scale, args.sample_scale)
        } else {
            (args.sample_scale, args.sample_scale * 2)
        }
    };
    let map: Vec<char> = args.map.chars().collect();
    let colour = {
        if args.colour.len() == 3 {
            Some([args.colour[0], args.colour[1], args.colour[2], 255])
        } else {
            None
        }
    };

    println!("Converting image to ASCII...");
    let image = image::open(args.input_image).expect("Error - Failed to open image ");
    let ascii = Ascii::new(&image, sample_size);
    println!("Success!");

    match args.output_image {
        Some(output_path) => {
            println!("Converting ASCII to image...");
            let font = Font::try_from_bytes(FONT_BYTES).expect("Errzor - Cannot read font bytes");
            let ascii_image = ascii
                .as_image(&map, colour, font, args.font_size)
                .expect("Error - Cannot create ASCII image");
            ascii_image
                .save(output_path)
                .expect("Error - Cannot save ASCII image");
            println!("Success!");
        }

        None => ascii.print(&map, colour),
    }

    Ok(())
}
