pub use libremarkable::framebuffer::{
    common::mxcfb_rect,
    cgmath::Point2,
    cgmath::Vector2,
    FramebufferBase,
    FramebufferDraw,
    FramebufferIO,
    FramebufferRefresh,
    core::Framebuffer,
    common::DISPLAYWIDTH,
    common::DISPLAYHEIGHT,
    common::color,
};
use libremarkable::framebuffer::{
    common::waveform_mode,
    common::display_temp,
    common::dither_mode,
    refresh::PartialRefreshMode,
    common::DRAWING_QUANT_BIT,
    common::DRAWING_QUANT_BIT_2,
    common::DRAWING_QUANT_BIT_3,
};
use std::ops::DerefMut;

pub struct Canvas<'a> {
    framebuffer: Box::<Framebuffer<'a>>,
}

impl<'a> Canvas<'a> {
    pub fn new() -> Self {
        let mut instance = Self {
            framebuffer: Box::new(Framebuffer::new("/dev/fb0")),
        };
        instance
    }

    pub fn framebuffer_mut(&mut self) -> &'static mut Framebuffer<'static> {
        unsafe {
            std::mem::transmute::<_, &'static mut Framebuffer<'static>>(
                self.framebuffer.deref_mut(),
            )
        }
    }

    pub fn clear(&mut self) {
        self.framebuffer_mut().clear();
    }

    pub fn update_full(&mut self) {
        self.framebuffer_mut().full_refresh(
            waveform_mode::WAVEFORM_MODE_INIT,
            display_temp::TEMP_USE_REMARKABLE_DRAW,
            dither_mode::EPDC_FLAG_USE_REMARKABLE_DITHER,
            0,
            true
        );
    }
    
    pub fn update_partial(&mut self, region: &mxcfb_rect) {
        self.framebuffer_mut().partial_refresh(
            region,
            PartialRefreshMode::Async,
            waveform_mode::WAVEFORM_MODE_GLR16,
            display_temp::TEMP_USE_REMARKABLE_DRAW,
            dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
            0,
            false
        );
        /*self.fb.deref_mut().partial_refresh(
            region,
            PartialRefreshMode::Async,
            waveform_mode::WAVEFORM_MODE_DU,
            display_temp::TEMP_USE_REMARKABLE_DRAW,
            dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
            DRAWING_QUANT_BIT,
            false
        );*/
    }
}