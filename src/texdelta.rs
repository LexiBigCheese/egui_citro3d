use std::{collections::HashMap, ops::Deref};

use ctru_sys::{GPU_A8, GPU_RGBA8};
use egui::{epaint, Color32};
use swizzle_3ds::pix::ImageView;

use crate::texture::Texture;

use crate::TexAndData;

use super::{egui_filter_to_3ds, ImgDat};

pub fn texdelta(
    texmap: &mut HashMap<egui::TextureId, TexAndData>,
    set_deltas: Vec<(egui::TextureId, epaint::ImageDelta)>,
) {
    for x in set_deltas {
        single_delta(texmap, x);
    }
}

#[inline]
fn single_delta(
    texmap: &mut HashMap<egui::TextureId, TexAndData>,
    (texid, delta): (egui::TextureId, epaint::ImageDelta),
) {
    let tad = texmap.get_mut(&texid);
    let mut tad = match compare_tad_with_delta(tad, &delta) {
        ComparisonResult::PatchTexture { x, y } => {
            let Some(TexAndData { tex, data }) = texmap.remove(&texid) else {
                return;
            };
            let data = match delta_to_data(&delta) {
                super::ImgDat::Rgba8(vec) => {
                    let ImgDat::Rgba8(mut internal_dat) = data else {
                        return;
                    };
                    patch_texture(&delta, &tex, vec, &mut internal_dat, x, y);
                    ImgDat::Rgba8(internal_dat)
                }
                super::ImgDat::Alpha8(vec) => {
                    let ImgDat::Alpha8(mut internal_dat) = data else {
                        return;
                    };
                    patch_texture(&delta, &tex, vec, &mut internal_dat, x, y);
                    ImgDat::Alpha8(internal_dat)
                }
            };
            TexAndData { tex, data }
        }
        ComparisonResult::ReuseTexture => {
            let Some(TexAndData { tex, .. }) = texmap.remove(&texid) else {
                return;
            };
            TexAndData {
                tex,
                data: delta_to_data(&delta),
            }
        }
        ComparisonResult::CreateNewTexture => {
            let delta_to_data = delta_to_data(&delta);
            println!(
                "Texture is {}x{}",
                delta.image.width(),
                delta.image.height()
            );
            let tex = Texture::new(
                delta.image.width() as u16,
                delta.image.height() as u16,
                match delta_to_data {
                    super::ImgDat::Rgba8(..) => GPU_RGBA8,
                    super::ImgDat::Alpha8(..) => {
                        println!("Haiiii i'm a goob!");
                        GPU_A8
                    }
                },
                match delta_to_data {
                    super::ImgDat::Rgba8(..) => false,
                    super::ImgDat::Alpha8(..) => true
                },
            );

            TexAndData {
                tex,
                data: delta_to_data,
            }
        }
        ComparisonResult::Panic => panic!("TAD and epaint::ImageDelta are Incompatible"),
        ComparisonResult::Oob => panic!("ImageDelta trying to patch image out of texture bounds!"),
    };
    //Swizzle and upload
    let swizzled = swizzle_3ds::swizzle_image(&ImageView::new(
        tad.data.deref(),
        tad.tex.width as usize,
        tad.tex.height as usize,
        pixel_format_swizzle(&delta.image),
    ));
    unsafe {
        tad.tex.upload(swizzled.as_raw());
    };
    //Set Texture Options (Min and Mag filter)
    tad.tex.set_filter(
        egui_filter_to_3ds(delta.options.magnification),
        egui_filter_to_3ds(delta.options.minification),
    );
    tad.tex.set_wrap(
        match delta.options.wrap_mode {
            egui::TextureWrapMode::ClampToEdge => {
                ctru_sys::GPU_CLAMP_TO_EDGE
            },
            egui::TextureWrapMode::Repeat => {
                ctru_sys::GPU_REPEAT
            },
            egui::TextureWrapMode::MirroredRepeat => {
                ctru_sys::GPU_MIRRORED_REPEAT
            },
        }
    );
    texmap.insert(texid, tad);
}

fn patch_texture<T: Copy>(
    delta: &epaint::ImageDelta,
    tex: &Texture,
    vec: Vec<T>,
    to_patch: &mut Vec<T>,
    x: usize,
    y: usize,
) {
    for (write_chunk, read_chunk) in to_patch
        .chunks_exact_mut(tex.width as usize)
        .skip(y)
        .zip(vec.chunks_exact(delta.image.width()))
    {
        write_chunk[x..(x + delta.image.width())].copy_from_slice(read_chunk);
    }
}

fn delta_to_data(delta: &epaint::ImageDelta) -> super::ImgDat {
    match &delta.image {
        egui::ImageData::Color(color_image) => convert_color32_to_rgba8(&color_image.pixels).into(),
        egui::ImageData::Font(font_image) => convert_font_to_lum8(&font_image.pixels).into(),
    }
}

fn convert_font_to_lum8(x: &Vec<f32>) -> Vec<u8> {
    let mut max = (-1isize, 0.0);
    for (i, &x) in x.into_iter().enumerate() {
        if x > max.1 {
            max = (i as isize, x);
        }
    }
    println!("Max Pixel: #{} with value {}", max.0, max.1);
    x.into_iter().map(|&x| (x * 255.0).floor() as u8).collect()
}

fn convert_color32_to_rgba8(x: &Vec<Color32>) -> Vec<u32> {
    x.into_iter()
        .map(|&x| {
            // let a = 255.0 / (x.a() as f32);
            // let (r, g, b) = (
            //     (x.r() as f32 * a) as u8,
            //     (x.g() as f32 * a) as u8,
            //     (x.b() as f32 * a) as u8,
            // );
            // u32::from_ne_bytes([r, g, b, x.a()])
            // if !(x.r() == x.g() && x.g() == x.b() && x.b() == x.a()) {
            //     println!("Halleleuja!");
            // }
            u32::from_le_bytes([x.r(),x.g(),x.b(),x.a()])
        })
        .collect()
}

enum ComparisonResult {
    ReuseTexture,
    PatchTexture { x: usize, y: usize },
    CreateNewTexture,
    Panic,
    Oob,
}

fn pixel_format_swizzle(x: &egui::ImageData) -> swizzle_3ds::pix::ImageFormat {
    match x {
        egui::ImageData::Color(..) => swizzle_3ds::pix::ImageFormat::Rgba8,
        egui::ImageData::Font(..) => swizzle_3ds::pix::ImageFormat::Alpha8,
    }
}

fn compare_tad_with_delta(
    tad: Option<&mut TexAndData>,
    delta: &epaint::ImageDelta,
) -> ComparisonResult {
    let Some(tad) = tad else {
        return ComparisonResult::CreateNewTexture;
    };
    let same_data_type =
        matches!(tad.data, ImgDat::Rgba8(..)) == matches!(delta.image, egui::ImageData::Color(..));
    if let Some([x, y]) = delta.pos {
        return if same_data_type {
            if x + delta.image.width() > tad.tex.width as usize
                || y + delta.image.height() > tad.tex.height as usize
            {
                ComparisonResult::Oob
            } else {
                ComparisonResult::PatchTexture { x, y }
            }
        } else {
            ComparisonResult::Panic
        };
    }
    if same_data_type && delta.image.width() == tad.tex.width as usize && delta.image.height() == tad.tex.height as usize {
        ComparisonResult::ReuseTexture
    } else {
        ComparisonResult::CreateNewTexture
    }
}
