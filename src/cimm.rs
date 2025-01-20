use citro3d_sys::{C3D_ImmDrawBegin,C3D_ImmDrawEnd,C3D_ImmSendAttrib};
use ctru_sys::GPU_TRIANGLES;

pub fn imm(f: impl FnOnce()) {
    unsafe {
        C3D_ImmDrawBegin(GPU_TRIANGLES);
    }
    f();
    unsafe {
        C3D_ImmDrawEnd();
    }
}

pub fn attr(xyzw: [f32;4]) {
    unsafe {
        C3D_ImmSendAttrib(xyzw[0],xyzw[1],xyzw[2],xyzw[3]);
    }
}
