use ffi;
use super::{Error, Frame, Image};

use libc;

pub mod vp9;

const ENCODER_ABI_VERSION: i32 = super::CODEC_ABI_VERSION + 5;

pub const DL_REALTIME: u64 = 1;
pub const DL_GOOD_QUALITY: u64 = 1000000;
pub const DL_BEST_QUALITY: u64 = 0;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FrameFlags {
    keyframe: bool,
}
impl Default for FrameFlags {
    fn default() -> FrameFlags {
        FrameFlags {
            keyframe: false,
        }
    }
}
#[doc(hidden)]
impl Into<ffi::vpx_enc_frame_flags_t> for FrameFlags {
    fn into(self) -> ffi::vpx_enc_frame_flags_t {
        let mut flags: ffi::vpx_enc_frame_flags_t = 0;
        if self.keyframe { flags |= 1 << 0; }
        return flags;
    }
}
impl FrameFlags {
    pub fn new() -> FrameFlags { Default::default() }

    pub fn keyframe(mut self, keyframe: bool) -> FrameFlags {
        self.keyframe = keyframe;
        self
    }
}

pub trait Encoder: InternalEncoder
    where <Self as Encoder>::Cfg: AsRef<ffi::vpx_codec_enc_cfg_t>,
{
    type Cfg;
    fn set_cfg(&mut self, cfg: Self::Cfg) -> Result<(), Error> {
        let res = unsafe {
            ffi::vpx_codec_enc_config_set(self.get_mut_ctx(),
                                          cfg.as_ref() as *const _)
        };
        if res == 0 {
            Ok(())
        } else {
            Err(From::from(res))
        }
    }

    /// `duration` must be non-zero.
    fn encode(&mut self, image: &Image,
              pts: ffi::vpx_codec_pts_t,
              duration: u64,
              flags: FrameFlags,
              deadline: u64) -> Result<(), Error> {
        let res = unsafe {
            ffi::vpx_codec_encode(self.get_mut_ctx(),
                                  &image.0 as *const _,
                                  pts,
                                  duration as libc::c_ulong,
                                  flags.into(),
                                  deadline as libc::c_ulong)
        };
        if res != 0 {
            Err(From::from(res))
        } else {
            Ok(())
        }
    }

    /// Call once there are no more frames to encode.
    fn flush(&mut self,
             pts: ffi::vpx_codec_pts_t,
             duration: u64,
             flags: ffi::vpx_enc_frame_flags_t,
             deadline: u64) -> Result<(), Error>
    {
        let res = unsafe {
            ffi::vpx_codec_encode(self.get_mut_ctx(),
                                  0 as *const _,
                                  pts, duration as libc::c_ulong,
                                  flags, deadline as libc::c_ulong)
        };
        if res == 0 {
            Ok(())
        } else {
            Err(From::from(res))
        }
    }

    fn packets<T: PacketWriter>(&mut self, dest: &mut T) -> Result<(), ::std::io::Error> {
        use std::mem::transmute;
        use std::slice::from_raw_parts;
        let mut iter: ffi::vpx_codec_iter_t = 0 as *mut _;
        unsafe {
            loop {
                let pkt = ffi::vpx_codec_get_cx_data(self.get_mut_ctx(),
                                                     &mut iter as *mut _);
                if pkt.is_null() { return Ok(()); }

                let pkt: &ffi::vpx_codec_cx_pkt_t = transmute(pkt);
                match pkt.kind {
                    ffi::VPX_CODEC_CX_FRAME_PKT => {
                        let frame: &ffi::Struct_Unnamed6 = transmute(pkt.data.frame_ref());
                        let frame: Frame = From::from(frame);
                        try!(dest.write_frame(&frame));
                    },
                    ffi::VPX_CODEC_STATS_PKT => {
                        let buf: &ffi::vpx_fixed_buf_t = transmute(pkt.data.twopass_stats_ref());
                        let buf = from_raw_parts(buf.buf as *const u8, buf.sz as usize);
                        try!(dest.write_two_pass_stats(buf));
                    },
                    ffi::VPX_CODEC_FPMB_STATS_PKT => {
                        let buf: &ffi::vpx_fixed_buf_t = transmute(pkt.data.twopass_stats_ref());
                        let buf = from_raw_parts(buf.buf as *const u8, buf.sz as usize);
                        try!(dest.write_two_pass_stats(buf));
                    },
                    ffi::VPX_CODEC_PSNR_PKT => {
                        let psnr: &ffi::Struct_vpx_psnr_pkt = transmute(pkt.data.psnr_ref());
                        try!(dest.write_psnr(&psnr.samples, &psnr.sse, &psnr.psnr));
                    },
                    kind => {
                        try!(dest.write_custom(kind, &pkt.data));
                    },
                }
            }
        }
    }
}

#[doc(hidden)]
pub trait InternalEncoder {
    fn get_ref_ctx(&self) -> *const ffi::vpx_codec_ctx_t;
    fn get_mut_ctx(&mut self) -> *mut ffi::vpx_codec_ctx_t;
}

pub trait PacketWriter {
    fn write_frame<'a>(&mut self, _frame: &Frame<'a>) -> Result<(), ::std::io::Error> { Ok(()) }
    fn write_two_pass_stats(&mut self, _stats: &[u8]) -> Result<(), ::std::io::Error> { Ok(()) }
    fn write_first_pass_mb_stats(&mut self, _stats: &[u8]) -> Result<(), ::std::io::Error> { Ok(()) }
    fn write_psnr(&mut self, _samples: &[u32; 4], _sse: &[u64; 4],
                  _psnr: &[f64; 4]) -> Result<(), ::std::io::Error> { Ok(()) }
    fn write_custom(&mut self,
                    _kind: ffi::Enum_vpx_codec_cx_pkt_kind,
                    _data: &ffi::Union_Unnamed5) -> Result<(), ::std::io::Error> { Ok(()) }
}
