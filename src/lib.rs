#![feature(allocator_api)]
#![feature(iter_array_chunks)]

pub mod texture;

pub mod input;
pub mod texdelta;
pub mod common;
pub mod cimm;

use std::{collections::HashMap, ops::Deref};

use citro3d::{math::FVec4, Instance};
use ctru::prelude::{Hid, KeyPad};
use derive_more::derive::From;
use egui::ahash::HashMapExt;

use crate::{
    common::AllPass,
    texture::Texture,
};

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ImeStage {
    Nothing,
    SelectAllDown,
    SelectAllUp,
    BackSpaceDown,
    BackSpaceUp,
    PutText,
    EscapeDown,
    EscapeUp,
}

impl ImeStage {
    const START:ImeStage=ImeStage::SelectAllDown;
    const CANCEL:ImeStage=ImeStage::EscapeDown;
    fn next(self) -> Self {
        use ImeStage::*;
        match self {
            Nothing => Nothing,
            SelectAllDown => SelectAllUp,
            SelectAllUp => BackSpaceDown,
            BackSpaceDown => BackSpaceUp,
            BackSpaceUp => PutText,
            PutText => EscapeDown,
            EscapeDown => EscapeUp,
            EscapeUp => Nothing,
        }
    }
    fn add_event(self, events: &mut Vec<egui::Event>) -> bool {
        use ImeStage::*;
        match self {
            Nothing => false,
            SelectAllDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::A,
                    pressed: true,
                    modifiers: egui::Modifiers::COMMAND,
                });
                false
            }
            SelectAllUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::A,
                    pressed: false,
                    modifiers: egui::Modifiers::COMMAND,
                });
                false
            }
            BackSpaceDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Backspace,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            BackSpaceUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Backspace,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            PutText => {
                true
            }
            EscapeDown => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Escape,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
            EscapeUp => {
                events.push(egui::Event::Key {
                    physical_key: None,
                    repeat: false,
                    key: egui::Key::Escape,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                });
                false
            }
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
    let mut viewports = egui::ViewportIdMap::new();
    viewports.insert(
        bottom_viewport_id,
        egui::ViewportInfo {
            native_pixels_per_point: Some(1.0),
            parent: None,
            title: None,
            events: vec![],
            monitor_size: Some(bottom_screen_size),
            inner_rect: Some(bottom_rect),
            outer_rect: Some(bottom_rect),
            minimized: Some(false),
            maximized: Some(true),
            fullscreen: Some(true),
            focused: Some(true),
        },
    );
    viewports.insert(
        top_viewport_id,
        egui::ViewportInfo {
            native_pixels_per_point: Some(1.0),
            parent: None,
            title: None,
            events: vec![],
            monitor_size: Some(top_screen_size),
            inner_rect: Some(top_rect),
            outer_rect: Some(top_rect),
            minimized: Some(false),
            maximized: Some(true),
            fullscreen: Some(true),
            focused: Some(true),
        },
    );

    let mut ime: Option<egui::output::IMEOutput> = None;
    let mut ime_stage = ImeStage::Nothing;
    let mut current_text_value: Option<String> = None;
    let mut current_float_value: Option<f64> = None;
    let mut last_pos: egui::Pos2 = Default::default();
    unsafe {
        citro3d_sys::C3D_CullFace(ctru_sys::GPU_CULL_NONE);
        citro3d_sys::C3D_DepthTest(false, ctru_sys::GPU_NEVER, ctru_sys::GPU_WRITE_ALL);
        citro3d_sys::C3D_EarlyDepthTest(false, ctru_sys::GPU_NEVER, 0);
        citro3d_sys::C3D_AlphaBlend(
            ctru_sys::GPU_BLEND_ADD,
            ctru_sys::GPU_BLEND_ADD,
            ctru_sys::GPU_SRC_ALPHA,
            ctru_sys::GPU_ONE_MINUS_SRC_ALPHA,
            ctru_sys::GPU_SRC_ALPHA,
            ctru_sys::GPU_ONE_MINUS_SRC_ALPHA,
        );
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
        if let Some(_) = ime {
            if ime_stage == ImeStage::Nothing {
                use ctru::applets::swkbd;
                let mut kbd =
                    swkbd::SoftwareKeyboard::new(swkbd::Kind::Normal, swkbd::ButtonConfig::LeftRight);
                kbd.set_initial_text(
                    current_text_value.take()
                        .map(|x| std::borrow::Cow::Owned(x))
                        .or(current_float_value.take().map(|x| std::borrow::Cow::Owned(x.to_string()))),
                );
                let (text, button) = kbd.launch(&apt, &gfx).unwrap();
                if button == swkbd::Button::Right {
                    current_text_value = Some(text);
                    ime_stage = ImeStage::START;
                } else {
                    ime_stage = ImeStage::CANCEL;
                }
            }
        }
        if ime_stage.add_event(&mut events) {
            events.push(egui::Event::Text(current_text_value.take().unwrap_or_default()));
        }
        ime_stage = ime_stage.next();
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
                run_ui(c, Specifics {hid: &hid,top_viewport_id,bottom_viewport_id});
            },
        );
        for e in &out.platform_output.events {
            match e {
                egui::output::OutputEvent::Clicked(widget_info) => {
                    if ime_stage == ImeStage::Nothing {
                        current_text_value = widget_info.current_text_value.clone();
                        current_float_value = widget_info.value.clone();
                    }
                }
                _ => (),
            }
        }
        ime = out.platform_output.ime;
        everything_that_happens_after_out(
            &hid,
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
                run_ui(c, Specifics {hid: &hid,top_viewport_id,bottom_viewport_id});
            },
        );
        everything_that_happens_after_out(
            &hid,
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
    println!("whaaaa?");
    drop(shader);
}

fn everything_that_happens_after_out(
    hid: &Hid,
    instance: &mut Instance,
    ctx: &egui::Context,
    texmap: &mut HashMap<egui::TextureId, TexAndData>,
    twovecs: [[f32; 4]; 2],
    projection_uniform_idx: citro3d::uniform::Index,
    attr_info: &citro3d::attrib::Info,
    render_target: &mut citro3d::render::Target<'_>,
    out: egui::FullOutput,
) {
    if !out.textures_delta.set.is_empty() {
        println!("Adding/Patching {} Textures", out.textures_delta.set.len());
    }
    if hid.keys_down().contains(KeyPad::B) {
        println!("Rendering {} shapes", out.shapes.len());
    }
    if hid.keys_down().contains(KeyPad::Y) {
        println!("{:#?}", out.shapes);
    }

    texdelta::texdelta(texmap, out.textures_delta.set);
    let tessel = ctx.tessellate(out.shapes, 1.0);

    instance.render_frame_with(|instance| {
        render_target.clear(citro3d::render::ClearFlags::ALL, 0xFF_00_00_00, 0);
        // let mut last_christmas_i_gave_you_my = None;

        instance
            .select_render_target(&*render_target)
            .expect("wharg");

        instance.bind_vertex_uniform(projection_uniform_idx, twovecs_to_uniform(twovecs));
        instance.set_attr_info(attr_info);
        if hid.keys_down().contains(KeyPad::B) {
            println!("Rendering {} prims", tessel.len());
        }
        for t in tessel.into_iter() {
            let mesh = match t.primitive {
                egui::epaint::Primitive::Mesh(mesh) => mesh,
                egui::epaint::Primitive::Callback(_) => {
                    continue;
                }
            };
            let TexAndData { tex, data } = texmap.get_mut(&mesh.texture_id).unwrap();
            tex.bind(0);
            configure_texenv(instance, data);
            for mesh in mesh.split_to_u16() {
                if hid.keys_down().contains(KeyPad::X) {
                    println!("Tex  : {}x{}@{}", tex.width, tex.height, tex.format);
                    println!("Verts: ");
                    for vert in &mesh.vertices {
                        println!("{:?}", vert);
                    }
                    println!("Indices: ");
                    for arr in mesh.indices.chunks_exact(3) {
                        println!("({} {} {})", arr[0], arr[1], arr[2]);
                    }
                }
                use cimm::imm;
                use cimm::attr;
                imm(|| {
                    for i in mesh.indices {
                        let egui::epaint::Vertex { pos, uv, color } = mesh.vertices[i as usize];
                        attr([pos.x, pos.y, 0.0, 0.0]);
                        attr([uv.x, uv.y, 0.0, 0.0]);
                        attr([
                            color.r() as f32 / 255.0,
                            color.g() as f32 / 255.0,
                            color.b() as f32 / 255.0,
                            color.a() as f32 / 255.0,
                        ]);
                    }
                });
            }
            unsafe {
                use citro3d_sys::{C3D_DirtyTexEnv, C3D_GetTexEnv};
                let te = C3D_GetTexEnv(0);
                C3D_DirtyTexEnv(te);
            }
        }
    });
    for remove in out.textures_delta.free {
        texmap.remove(&remove);
    }
}

fn twovecs_to_uniform(twovecs_bottom: [[f32; 4]; 2]) -> citro3d::uniform::Uniform {
    citro3d::uniform::Uniform::Float2([
        FVec4::from_raw(citro3d_sys::C3D_FVec {
            c: twovecs_bottom[0],
        }),
        FVec4::from_raw(citro3d_sys::C3D_FVec {
            c: twovecs_bottom[1],
        }),
    ])
}

fn configure_font_texenv(instance: &mut Instance) {
    use citro3d::texenv;
    let stage0 = texenv::Stage::new(0).unwrap();
    let texenv0 = instance.texenv(stage0);
    texenv0
        .src(texenv::Mode::RGB, texenv::Source::PrimaryColor, None, None)
        .func(texenv::Mode::RGB, texenv::CombineFunc::Replace);
    texenv0
        .src(
            texenv::Mode::ALPHA,
            texenv::Source::Texture0,
            Some(texenv::Source::PrimaryColor),
            None,
        )
        .func(texenv::Mode::ALPHA, texenv::CombineFunc::Modulate);
}

fn configure_rgba8_texenv(instance: &mut Instance) {
    use citro3d::texenv;
    let stage0 = texenv::Stage::new(0).unwrap();
    let texenv0 = instance.texenv(stage0);
    texenv0
        .src(texenv::Mode::RGB, texenv::Source::Texture0, None, None)
        .func(texenv::Mode::RGB, texenv::CombineFunc::Replace);
    texenv0
        .src(
            texenv::Mode::ALPHA,
            texenv::Source::Texture0,
            Some(texenv::Source::PrimaryColor),
            None,
        )
        .func(texenv::Mode::ALPHA, texenv::CombineFunc::Modulate);
}

/// REMEMBER TO HAVE A BOUND BUFFER
pub unsafe fn draw_elements_u16(indices: &[u16]) {
    unsafe {
        citro3d_sys::C3D_DrawElements(
            ctru_sys::GPU_TRIANGLES,
            indices.len() as i32,
            citro3d_sys::C3D_UNSIGNED_SHORT as i32,
            indices.as_ptr().cast(),
        );
    }
}

fn configure_texenv(instance: &mut Instance, data: &ImgDat) {
    match data {
        ImgDat::Rgba8(..) => {
            configure_rgba8_texenv(instance);
        }
        ImgDat::Alpha8(..) => {
            configure_font_texenv(instance);
        }
    }
}

fn egui_filter_to_3ds(t: egui::TextureFilter) -> ctru_sys::GPU_TEXTURE_FILTER_PARAM {
    match t {
        egui::TextureFilter::Nearest => ctru_sys::GPU_NEAREST,
        egui::TextureFilter::Linear => ctru_sys::GPU_LINEAR,
    }
}
