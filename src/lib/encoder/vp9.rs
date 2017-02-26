use ffi;

use {InternalInterface, Error, Kind};

use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Cfg(ffi::vpx_codec_enc_cfg_t);
impl Default for Cfg {
    fn default() -> Cfg {
        let interface: Interface = Default::default();
        let mut cfg: ffi::vpx_codec_enc_cfg_t = Default::default();
        let err = unsafe {
            ffi::vpx_codec_enc_config_default(interface.iface(),
                                              &mut cfg as *mut _,
                                              0)
        };
        assert_eq!(err, ffi::VPX_CODEC_OK);
        Cfg(cfg)
    }
}
impl AsRef<ffi::vpx_codec_enc_cfg_t> for Cfg {
    fn as_ref(&self) -> &ffi::vpx_codec_enc_cfg_t {
        &self.0
    }
}
impl AsMut<ffi::vpx_codec_enc_cfg_t> for Cfg {
    fn as_mut(&mut self) -> &mut ffi::vpx_codec_enc_cfg_t {
        &mut self.0
    }
}
impl Deref for Cfg {
    type Target = ffi::vpx_codec_enc_cfg_t;
    fn deref(&self) -> &ffi::vpx_codec_enc_cfg_t {
        &self.0
    }
}
impl DerefMut for Cfg {
    fn deref_mut(&mut self) -> &mut ffi::vpx_codec_enc_cfg_t {
        &mut self.0
    }
}

#[derive(Copy, Clone)]
pub struct Interface;
impl Default for Interface {
    fn default() -> Interface {
        Interface
    }
}
impl ::Interface for Interface {
    type Context = Context;
    type Cfg = Cfg;
    fn kind(&self) -> Kind { Kind::Encoder }

    fn create(&self, cfg: <Self as ::Interface>::Cfg,
              flags: ffi::vpx_codec_flags_t) ->
        Result<<Self as ::Interface>::Context, Error>
    {
        let mut ctx: ffi::vpx_codec_ctx_t = Default::default();
        let err = unsafe {
            ffi::vpx_codec_enc_init_ver(&mut ctx as *mut _,
                                        self.iface(),
                                        &cfg.0 as *const _,
                                        flags,
                                        ffi::VPX_ENCODER_ABI_VERSION as i32)
        };
        if err != ffi::VPX_CODEC_OK {
            Err(From::from(err))
        } else {
            Ok(Context(ctx))
        }
    }
}
impl InternalInterface for Interface {
    fn iface(&self) -> *mut ffi::vpx_codec_iface_t {
        unsafe { &mut ffi::vpx_codec_vp9_cx_algo as *mut _ }
    }
}

pub struct Context(ffi::vpx_codec_ctx_t);
unsafe impl Send for Context {}
impl super::InternalEncoder for Context {
    fn get_ref_ctx(&self) -> *const ffi::vpx_codec_ctx_t {
        &self.0 as *const _
    }
    fn get_mut_ctx(&mut self) -> *mut ffi::vpx_codec_ctx_t {
        &mut self.0 as *mut _
    }
}
impl super::Encoder for Context {
    type Cfg = Cfg;
}
