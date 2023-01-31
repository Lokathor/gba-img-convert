use std::path::Path;

use imagine::{image::Palmap, pixel_formats::RGBA8888};

#[derive(Clone, Copy)]
#[repr(transparent)]
struct GbaColor(u16);
impl From<RGBA8888> for GbaColor {
  fn from(value: RGBA8888) -> Self {
    let r = value.r as u16 >> 3;
    let g = value.g as u16 >> 3;
    let b = value.b as u16 >> 3;
    Self(r | (g << 5) | (b << 10))
  }
}
impl core::fmt::Debug for GbaColor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "0x{:04X}", self.0)
  }
}

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// Name(s) of the files to process.
  file_names: Vec<String>,

  #[arg(short, long, default_value_t = 3)]
  out_mode: u8,
}

fn main() {
  let args = Args::parse();
  println!("INPUT ARGS: {args:?}");

  for file_name in args.file_names.iter() {
    let in_path = Path::new(file_name.as_str());
    let out_path = in_path.with_extension("rs");
    //
    println!("in_path: {}", in_path.display());
    println!("out_path: {}", out_path.display());
    //
    let in_bytes = std::fs::read(in_path).unwrap();
    let palmap: Palmap<u8, RGBA8888> = Palmap::try_from_png_bytes(&in_bytes)
      .or_else(|| Palmap::try_from_bmp_bytes(&in_bytes).ok())
      .expect("Couldn't parse file as paletted PNG or BMP");

    let out = match args.out_mode {
      3 => palmap_output_mode3(&palmap),
      4 => palmap_output_mode4(&palmap),
      _ => panic!("Unsupported output mode! Only 3 and 4 work right now."),
    };
    std::fs::write(out_path, out.as_str()).unwrap();
  }
}

/// Output 240 x 160 direct-color image data, suitable for Video Mode 3
fn palmap_output_mode3(palmap: &Palmap<u8, RGBA8888>) -> String {
  let out_palette: Vec<GbaColor> =
    palmap.palette.iter().copied().map(GbaColor::from).collect();
  let out_pixels: Vec<GbaColor> = palmap
    .indexes
    .iter()
    .copied()
    .map(|i| out_palette[usize::from(i)])
    .collect();

  use core::fmt::Write;
  let mut out = String::new();
  writeln!(out, "pub const PIXELS: &[u16] = &{out_pixels:?};").ok();
  out
}

/// Output 240 x 160 8bpp indexed-color image data, suitable for Video Mode 4
fn palmap_output_mode4(palmap: &Palmap<u8, RGBA8888>) -> String {
  let out_palette: Vec<GbaColor> =
    palmap.palette.iter().copied().map(GbaColor::from).collect();

  use core::fmt::Write;
  let mut out = String::new();
  writeln!(out, "pub const INDEXES: &[u8] = &{:?};", palmap.indexes).ok();
  writeln!(out).ok();
  writeln!(out, "pub const PALETTE: &[u16] = &{out_palette:?};").ok();
  out
}
