extern crate image;
extern crate quicli;
extern crate rusttype;
extern crate structopt;
extern crate termcolor;

mod ascii_art;
mod ascii_art_builder;
mod character_map;

use crate::ascii_art_builder::AsciiArtBuilder;
use crate::character_map::CharacterMap;

use std::path::PathBuf;

use quicli::prelude::CliResult;
use rusttype::Font;
use structopt::StructOpt;

const FONT_BYTES: &[u8] = include_bytes!("../assets/Courier.ttf");

#[derive(StructOpt, Debug)]
#[structopt(name = "Rusciify", about = "Convert images to ASCII")]
struct Rusciify {
    input_path: PathBuf,

    #[structopt(long = "output", short = "o", help = "Path to output image file")]
    output_path: Option<PathBuf>,

    #[structopt(
        long = "map",
        short = "m",
        help = "The ASCII mapping to use for converting samples to characters"
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
        long = "sample-scale",
        short = "s",
        default_value = "5",
        help = "The scale of all NxN pixel samples to be converted to ASCII characters"
    )]
    sample_scale: usize,
}

fn main() -> CliResult {
    let args = Rusciify::from_args();

    let sample_dimensions = {
        if args.output_path.is_some() {
            (args.sample_scale, args.sample_scale)
        } else {
            (args.sample_scale, args.sample_scale * 2)
        }
    };

    let character_map = args.map.map_or(CharacterMap::default(), CharacterMap::from);

    let image = image::open(args.input_path).expect("Error - Failed to open image ");
    let ascii = AsciiArtBuilder::from(&image)
        .sample_dimensions(sample_dimensions)
        .character_map(character_map)
        .build();

    if let Some(mut output_path) = args.output_path {
        output_path.set_extension("png");

        println!("Writing image...");

        let font = Font::try_from_bytes(FONT_BYTES).expect("Errzor - Failed to read font bytes");

        let ascii_art = ascii.to_image(font, args.font_size, None).expect("Error - Failed to create ASCII image");
        ascii_art.save(output_path).expect("Error - Failed to save ASCII image");
        
        println!("Done.");
    } else {
        ascii.print().expect("Error - Failed to print");
    }

    Ok(())
}
