extern crate rusciify;
extern crate quicli;
extern crate structopt;
extern crate termcolor;
extern crate image;
extern crate rusttype;

use rusciify::{AsciiArtBuilder, char_maps};

use std::path::PathBuf;
use quicli::prelude::CliResult;
use rusttype::Font;
use structopt::StructOpt;
use image::ImageFormat;

const FONT_BYTES: &[u8] = include_bytes!("../../assets/Courier.ttf");

#[derive(StructOpt, Debug)]
#[structopt(name = "Rusciify", about = "Convert images to ASCII")]
struct Rusciify {
    input_path: PathBuf,

    #[structopt(long = "output", short = "o", help = "Path to output image file")]
    output_path: Option<PathBuf>,

    #[structopt(
        long = "char-map",
        short = "c",
        help = "The ASCII charaters to map the pixels of the image to"
    )]
    map: Option<String>,

    #[structopt(
        long = "font-size",
        short = "f",
        default_value = "25",
        help = "The font size of the ASCII characters in the output image"
    )]
    font_size: usize,

    #[structopt(
        long = "ascii-scale",
        short = "a",
        default_value = "5",
        help = "The N of all NxN pixel samples to be converted to ASCII characters"
    )]
    sample_scale: usize,

    #[structopt(
        long = "solid",
        short = "s",
        help = "A convenience flag for setting a solid ASCII character map"
    )]
    solid: bool
}

fn main() -> CliResult {
    // Read CLI args
    let args = Rusciify::from_args();
    let sample_dimensions = {
        if args.output_path.is_some() {
            (args.sample_scale, args.sample_scale)
        } else {
            (args.sample_scale, args.sample_scale * 2)
        }
    };

    // Default, solid or custom character-map
    let char_map = if args.solid {
        Vec::from(char_maps::SOLID)
    } else {
        args.map.map_or(Vec::from(char_maps::DEFAULT), |s| {
            s.chars().collect::<Vec<char>>()
        })
    };

    // Read input image and build the ASCII
    let image = image::open(&args.input_path).expect("Failed to open image");
    let ascii = AsciiArtBuilder::from(&image)
        .sample_dimensions(sample_dimensions)
        .char_map(char_map)
        .build();

    // Save ASCII image or print ASCII
    if let Some(mut output_path) = args.output_path {
        let input_path_extension = args.input_path.extension().expect("Failed to parse extension from input image path");
        let ascii_image_format = ImageFormat::from_extension(input_path_extension).expect("Failed to infer image format");
        let font = Font::try_from_bytes(FONT_BYTES).expect("Failed to read font bytes");
        let ascii_art = ascii.to_image(font, args.font_size, None).expect("Failed to create ASCII image");

        output_path.set_extension(input_path_extension);
        ascii_art.save_with_format(output_path, ascii_image_format).expect("Failed to save ASCII image");
    } else {
        ascii.print().expect("Failed to print");
    }

    Ok(())
}
