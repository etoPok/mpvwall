use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

#[link(name = "EGL")]
extern "C" {
    pub fn eglGetDisplay(native_display: *mut c_void) -> *mut c_void;
    pub fn eglInitialize(display: *mut c_void, major: *mut c_int, minor: *mut c_int) -> u32;
    pub fn eglBindAPI(api: u32) -> u32;
    pub fn eglChooseConfig(
        display: *mut c_void,
        attrib_list: *const c_int,
        configs: *mut *mut c_void,
        config_size: c_int,
        num_config: *mut c_int,
    ) -> u32;
    pub fn eglCreateWindowSurface(
        display: *mut c_void,
        config: *mut c_void,
        native_window: *mut c_void,
        attrib_list: *const c_int,
    ) -> *mut c_void;
    pub fn eglCreateContext(
        display: *mut c_void,
        config: *mut c_void,
        share_context: *mut c_void,
        attrib_list: *const c_int,
    ) -> *mut c_void;
    pub fn eglMakeCurrent(
        display: *mut c_void,
        draw: *mut c_void,
        read: *mut c_void,
        ctx: *mut c_void,
    ) -> u32;
    pub fn eglSwapBuffers(display: *mut c_void, surface: *mut c_void) -> u32;
    pub fn eglSwapInterval(display: *mut c_void, interval: c_int) -> u32;
    pub fn eglGetProcAddress(procname: *const c_char) -> *mut c_void;
    pub fn eglDestroyContext(display: *mut c_void, ctx: *mut c_void) -> u32;
    pub fn eglDestroySurface(display: *mut c_void, surface: *mut c_void) -> u32;
    pub fn eglTerminate(display: *mut c_void) -> u32;
}

pub const EGL_OPENGL_API: u32 = 0x30A2;
pub const EGL_NONE: c_int = 0x3038;
pub const EGL_SURFACE_TYPE: c_int = 0x3033;
pub const EGL_WINDOW_BIT: c_int = 0x0004;
pub const EGL_RENDERABLE_TYPE: c_int = 0x3040;
pub const EGL_OPENGL_BIT: c_int = 0x0008;
pub const EGL_RED_SIZE: c_int = 0x3024;
pub const EGL_GREEN_SIZE: c_int = 0x3023;
pub const EGL_BLUE_SIZE: c_int = 0x3022;
pub const EGL_ALPHA_SIZE: c_int = 0x3021;
pub const EGL_DEPTH_SIZE: c_int = 0x3025;
pub const EGL_CONTEXT_MAJOR_VERSION: c_int = 0x3098;
pub const EGL_CONTEXT_MINOR_VERSION: c_int = 0x30FB;
pub const EGL_NO_DISPLAY: *mut c_void = ptr::null_mut();
pub const EGL_NO_CONTEXT: *mut c_void = ptr::null_mut();
pub const EGL_NO_SURFACE: *mut c_void = ptr::null_mut();
