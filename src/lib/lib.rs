#![feature(cstr_to_str)]

use std::borrow::Cow;
use std::ffi::{CStr};
use std::mem::transmute;

extern crate vpx_sys as ffi;

pub enum Error {
    Generic(u32),
    Mem,
    AbiMismatch,
    Incapable,
    UnsupportedBitstream,
    UnsupportedFrame,
    CorruptFrame,
    InvalidParam,
}
impl From<u32> for Error {
    fn from(v: u32) -> Error {
        match v {
            ffi::VPX_CODEC_MEM_ERROR => Error::Mem,
            ffi::VPX_CODEC_ABI_MISMATCH => Error::AbiMismatch,
            ffi::VPX_CODEC_INCAPABLE => Error::Incapable,
            ffi::VPX_CODEC_UNSUP_BITSTREAM => Error::UnsupportedBitstream,
            ffi::VPX_CODEC_UNSUP_FEATURE => Error::UnsupportedFrame,
            ffi::VPX_CODEC_CORRUPT_FRAME => Error::CorruptFrame,
            ffi::VPX_CODEC_INVALID_PARAM => Error::InvalidParam,
            n => Error::Generic(n),
        }
    }
}

pub type Rect = ffi::vpx_image_rect_t;

pub type Format = ffi::vpx_img_fmt_t;

const IMAGE_ABI_VERSION: u32 = 3;
pub struct Image<'a>(ffi::vpx_image_t, Cow<'a, [u8]>);

impl<'a> Image<'a> {
    /// XXX this function doesn't check that `data` is long enough for the
    /// format or view size.
    pub fn new(data: Cow<'a, [u8]>, fmt: Format,
               width: u32, height: u32,
               stride: u32) -> Image
    {
        let mut t: ffi::vpx_image_t = Default::default();
        unsafe {
            ffi::vpx_img_wrap(&mut t as *mut _,
                              fmt, width,
                              height, stride,
                              data.as_ptr() as *mut _);
        };
        Image(t, data)
    }

    pub fn get_format(&self) -> Format { self.0.fmt.clone() }

    pub fn set_rect(&mut self, rect: Rect) -> Result<(), ()> {
        let res = unsafe {
            ffi::vpx_img_set_rect(&mut self.0 as *mut _,
                                  rect.x, rect.y,
                                  rect.w, rect.h)
        };
        if res == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn flip(&mut self) {
        unsafe {
            ffi::vpx_img_flip(&mut self.0 as *mut _);
        }
    }
}
impl<'a> Drop for Image<'a> {
    fn drop(&mut self) {
        unsafe { ffi::vpx_img_free(&mut self.0 as *mut _) }
    }
}
#[derive(Debug, Clone)]
pub struct Frame<'a> {
    data: &'a [u8],
    pub pts: ffi::vpx_codec_pts_t,
    pub duration: usize,
    pub flags: ffi::vpx_codec_frame_flags_t,
    pub partition_id: i32,
}
pub const FRAME_IS_KEY: u32 = 0x1;
pub const FRAME_IS_DROPPABLE: u32 = 0x2;
pub const FRAME_IS_INVISIBLE: u32 = 0x4;
pub const FRAME_IS_FRAGMENT: u32 = 0x8;
impl<'a> Frame<'a> {
    pub fn data(&self) -> &'a [u8] { self.data }

    pub fn is_keyframe(&self) -> bool {
        self.flags & FRAME_IS_KEY != 0
    }
    pub fn is_droppable(&self) -> bool {
        self.flags & FRAME_IS_DROPPABLE != 0
    }
    pub fn is_invisible(&self) -> bool {
        self.flags & FRAME_IS_INVISIBLE != 0
    }
    pub fn is_fragment(&self) -> bool {
        self.flags & FRAME_IS_FRAGMENT != 0
    }
}
impl<'a> From<&'a ffi::Struct_Unnamed6> for Frame<'a> {
    fn from(v: &'a ffi::Struct_Unnamed6) -> Frame<'a> {
        let data: &'a [u8] = unsafe {
            ::std::slice::from_raw_parts(v.buf as *const u8, v.sz as usize)
        };

        Frame {
            data: data,
            pts: v.pts,
            duration: v.duration as usize,
            flags: v.flags,
            partition_id: v.partition_id,
        }
    }
}

const CODEC_ABI_VERSION: u32 = IMAGE_ABI_VERSION + 3;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Kind {
    Decoder,
    Encoder,
}

pub trait Interface: InternalInterface + Default {
    type Context;
    type Cfg;
    fn name(&self) -> &'static str {
        let pname = unsafe { ffi::vpx_codec_iface_name(self.iface()) };
        let str = unsafe { CStr::from_ptr(pname).to_str().unwrap() };
        unsafe { transmute(str) }
    }
    fn kind(&self) -> Kind;

    fn create(&self, cfg: <Self as Interface>::Cfg, flags: u64) ->
        Result<<Self as Interface>::Context, Error>;
}
#[doc(hidden)]
pub trait InternalInterface {
    fn iface(&self) -> *mut ffi::vpx_codec_iface_t;
}


/*#[derive(Copy, Clone)]
pub struct VP8DecoderInterface;
impl Interface for VP8DecoderInterface {
    fn kind(&self) -> Kind { Kind::Decoder }
}
impl InternalInterface for VP8DecoderInterface {
    fn iface(&self) -> *mut ffi::vpx_codec_iface_t {
        &mut ffi::vpx_codec_vp8_dx_algo as *mut _
    }
}
#[derive(Copy, Clone)]
pub struct VP9DecoderInterface;
impl Interface for VP9DecoderInterface {
    fn kind(&self) -> Kind { Kind::Decoder }
}
impl InternalInterface for VP9DecoderInterface {
    fn iface(&self) -> *mut ffi::vpx_codec_iface_t {
        &mut ffi::vpx_codec_vp9_dx_algo as *mut _
    }
}

#[derive(Copy, Clone)]
pub struct VP8EncoderInterface;
impl Interface for VP8EncoderInterface {
    fn kind(&self) -> Kind { Kind::Decoder }
}
impl InternalInterface for VP8EncoderInterface {
    fn iface(&self) -> *mut ffi::vpx_codec_iface_t {
        &mut ffi::vpx_codec_vp8_cx_algo as *mut _
    }
}*/
