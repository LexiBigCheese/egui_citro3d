pub struct Texture {
    internal: citro3d_sys::C3D_Tex,
    pub width: u16,
    pub height: u16,
    pub format: ctru_sys::GPU_TEXCOLOR,
    pub mipmap: bool,
}

impl Texture {
    pub fn new(width: u16, height: u16, format: ctru_sys::GPU_TEXCOLOR, mipmap: bool) -> Texture {
        let mut internal = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
        if unsafe {
            if mipmap {
                citro3d_sys::C3D_TexInitMipmap(&mut internal, width, height, format)
            } else {
                citro3d_sys::C3D_TexInit(&mut internal, width, height, format)
            }
        } {

            Texture {
                internal,
                width,
                height,
                format,
                mipmap,
            }
        } else {
            println!("Whoops! you have to put the CD in your computer.");
            panic!();
        }
    }
    ///Ensure that the buffer's size matches the width, height, and format of the texture.
    ///If it is too small, the 3DS **WILL** start reading who knows what data!
    pub unsafe fn upload(&mut self, buffer: &[u8]) {
        unsafe {
            citro3d_sys::C3D_TexUpload(&mut self.internal, buffer.as_ptr().cast());
            if self.mipmap {
                citro3d_sys::C3D_TexGenerateMipmap(&mut self.internal, ctru_sys::GPU_TEXFACE_2D);
            }
            citro3d_sys::C3D_TexFlush(&mut self.internal);
        }
    }
    pub fn bind(&mut self, to: i32) {
        unsafe {
            citro3d_sys::C3D_TexBind(to, &mut self.internal);
        }
    }
    pub fn set_filter(
        &mut self,
        mag: ctru_sys::GPU_TEXTURE_FILTER_PARAM,
        min: ctru_sys::GPU_TEXTURE_FILTER_PARAM,
    ) {
        unsafe {
            citro3d_sys::C3D_TexSetFilter(&mut self.internal, mag, min);
        }
    }
    pub fn set_wrap(
        &mut self,
        wrap: ctru_sys::GPU_TEXTURE_WRAP_PARAM
    ) {
        unsafe {
            citro3d_sys::C3D_TexSetWrap(&mut self.internal, wrap, wrap);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            citro3d_sys::C3D_TexDelete(&mut self.internal);
        }
    }
}
