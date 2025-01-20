use super::ImgDat;

use citro3d::Instance;

pub(crate) fn configure_font_texenv(instance: &mut Instance) {
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

pub(crate) fn configure_rgba8_texenv(instance: &mut Instance) {
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

pub(crate) fn configure_texenv(instance: &mut Instance, data: &ImgDat) {
    match data {
        ImgDat::Rgba8(..) => {
            configure_rgba8_texenv(instance);
        }
        ImgDat::Alpha8(..) => {
            configure_font_texenv(instance);
        }
    }
}
