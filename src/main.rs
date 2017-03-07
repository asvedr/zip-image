#![allow(dead_code)]
extern crate image;

mod dpix;
mod rect;
mod fpic;

use std::env;
use std::io::*;
use std::fs::File;
use image::DynamicImage;
use fpic::*;

/*
fn main() {
    let args : Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("args: in-image out-image");
        return;
    }
    //let inp : &str = &*args[1];
    let out : &str = &*args[2];
    let mut img : image::RgbImage = image::ImageBuffer::new(500, 500);
    macro_rules! clr {($v:expr, $m:expr) => {(($v as f64) / $m * 255.0) as u8};};
    for (x, y, px) in img.enumerate_pixels_mut() {
        *px = image::Rgb([clr!(x, 500.0), clr!(y, 500.0), 255 as u8])
    }
    match img.save(out) {
        Ok(_) => (),
        Err(e) => println!("error: {:?}", e)
    }
}
*/

fn main() {
    let args : Vec<String> = env::args().collect();
    let buf = BufReader::new(File::open(&*args[1]).unwrap());
    macro_rules! test {($img:expr) => {{
        let zpd = ZImage::zip(&$img);
        let img = zpd.unzip();
        match img.save("out.png") {
            Ok(_) => (),
            Err(e) => println!("save err {:?}", e)
        }
    }};}
    match image::load(buf, image::ImageFormat::PNG) {
        Ok(img) =>
            match img {
                DynamicImage::ImageLuma8(g) => test!(g),
                DynamicImage::ImageLumaA8(ga) => test!(ga),
                DynamicImage::ImageRgb8(rgb) => test!(rgb),
                DynamicImage::ImageRgba8(rgba) => test!(rgba)
            },
        Err(e) =>
            println!("load err {:?}", e)
    }
}
