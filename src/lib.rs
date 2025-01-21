#![feature(allocator_api)]
#![feature(iter_array_chunks)]

pub mod texture;
pub mod cimm;

pub(crate) mod common;
pub(crate) mod input;
pub(crate) mod texdelta;
pub(crate) mod ime;
pub(crate) mod create_viewports;
pub(crate) mod render;
pub(crate) mod configure_texenvs;

use std::{collections::HashMap, ops::Deref};

use ctru::prelude::Hid;
use derive_more::derive::From;

use crate::{common::AllPass, texture::Texture};

pub struct TexAndData {
    tex: Texture,
    data: ImgDat,
}

#[derive(From)]
enum ImgDat {
    Rgba8(Vec<u32>),
    Alpha8(Vec<u8>),
}

impl Deref for ImgDat {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match self {
            ImgDat::Rgba8(vec) => bytemuck::cast_slice(&vec[..]),
            ImgDat::Alpha8(vec) => bytemuck::cast_slice(&vec[..]),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Specifics<'a> {
    pub hid: &'a Hid,
    pub top_viewport_id: egui::ViewportId,
    pub bottom_viewport_id: egui::ViewportId,
}

pub fn run_egui(mut run_ui: impl FnMut(&egui::Context, Specifics)) {
    let AllPass {
        gfx,
        mut hid,
        apt,
        mut instance,
        shader,
        program,
    } = AllPass::new();
    #[cfg(feature = "dbg_printlns")]
    println!("Waow");
    let ctx = egui::Context::default();
    // ctx.tessellation_options_mut(|opts| {
    // });
    ctx.options_mut(|opts| {
        opts.reduce_texture_memory = true;
        opts.theme_preference = egui::ThemePreference::Dark;
    });
    ctx.set_embed_viewports(false);
    // egui_extras::install_image_loaders(&ctx);

    let mut texmap: HashMap<egui::TextureId, TexAndData> = HashMap::new();
    let twovecs_bottom = [[1.0, 0.0, -2.0 / 240.0, 0.0], [1.0, 0.0, 0.0, -2.0 / 320.0]];
    let twovecs_top = [[1.0, 0.0, -2.0 / 240.0, 0.0], [1.0, 0.0, 0.0, -2.0 / 400.0]];
    instance.bind_program(&program);
    let projection_uniform_idx = program
        .get_uniform("transform")
        .expect("No transform uniform?");
    let attr_info = common::prepare_attr_info();

    let (mut bottom_target, bottom_height, bottom_width) = common::bottom_target(&gfx, &instance);
    let (mut top_target, top_height, top_width) = common::top_target(&gfx, &instance);

    let bottom_screen_size = egui::vec2(bottom_width as f32, bottom_height as f32);
    let bottom_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, bottom_screen_size);
    let top_screen_size = egui::vec2(top_width as f32, top_height as f32);
    let top_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, top_screen_size);
    let bottom_viewport_id = egui::ViewportId::from_hash_of("bottom_viewport");
    let top_viewport_id = egui::ViewportId::from_hash_of("top_viewport");
    let viewports = create_viewports::create_viewports(
        bottom_screen_size,
        bottom_rect,
        top_screen_size,
        top_rect,
        bottom_viewport_id,
        top_viewport_id,
    );

    let mut ime: Option<egui::output::IMEOutput> = None;
    let mut ime_stage = ime::ImeStage::Nothing;
    let mut current_text_value: Option<String> = None;
    let mut current_float_value: Option<f64> = None;
    let mut last_pos: egui::Pos2 = Default::default();
    unsafe {
        //If you delete this call, faces *will* be culled
        citro3d_sys::C3D_CullFace(ctru_sys::GPU_CULL_NONE);
    }

    while apt.main_loop() {
        gfx.wait_for_vblank();
        //TODO: Split input handling into Top and Bottom segments
        //FOR NOW: Just don't send any inputs to the top screen
        hid.scan_input();
        let (mut events, start_button) = input::handle_input(&hid, &mut last_pos);
        if start_button {
            break;
        }
        ime::ime_part_a(&gfx, &apt, ime, &mut ime_stage, &mut current_text_value, &mut current_float_value, &mut events);
        let out = ctx.run(
            egui::RawInput {
                events,
                viewport_id: bottom_viewport_id,
                viewports: viewports.clone(),
                focused: true,
                max_texture_side: Some(1024),
                screen_rect: Some(bottom_rect),
                ..Default::default()
            },
            |c| {
                run_ui(c, Specifics {
                    hid: &hid,
                    top_viewport_id,
                    bottom_viewport_id,
                });
            },
        );
        ime::ime_part_b(&mut ime, &ime_stage, &mut current_text_value, &mut current_float_value, &out);
        render::everything_that_happens_after_out(
            #[cfg(feature = "dbg_printlns")] &hid,
            &mut instance,
            &ctx,
            &mut texmap,
            twovecs_bottom,
            projection_uniform_idx,
            &attr_info,
            &mut bottom_target,
            out,
        );
        let out = ctx.run(
            egui::RawInput {
                viewport_id: top_viewport_id,
                viewports: viewports.clone(),
                focused: false,
                max_texture_side: Some(1024),
                screen_rect: Some(top_rect),
                ..Default::default()
            },
            |c| {
                run_ui(c, Specifics {
                    hid: &hid,
                    top_viewport_id,
                    bottom_viewport_id,
                });
            },
        );
        render::everything_that_happens_after_out(
            #[cfg(feature = "dbg_printlns")] &hid,
            &mut instance,
            &ctx,
            &mut texmap,
            twovecs_top,
            projection_uniform_idx,
            &attr_info,
            &mut top_target,
            out,
        );
    }
    drop(shader);
}
