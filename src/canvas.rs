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
            waveform_mode::WAVEFORM_MODE_GC16,
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

    pub fn draw_text(&mut self, pos: Point2<Option<i32>>, text: &str, size: f32) -> mxcfb_rect {
        let mut pos = pos;
        if pos.x.is_none() || pos.y.is_none() {
            // Do dryrun to get text size
            let rect = self.framebuffer_mut().draw_text(Point2 { x: 0.0, y: DISPLAYHEIGHT as f32 }, text.to_owned(), size, color::BLACK, true);
        
            if pos.x.is_none() {
                // Center vertically
                pos.x = Some(DISPLAYWIDTH as i32 / 2 - rect.width as i32 / 2);
            }
            
            if pos.y.is_none() {
                // Center horizontally
                pos.y = Some(DISPLAYHEIGHT as i32 / 2 - rect.height as i32 / 2);
            }
        }
        let pos = Point2 { x: pos.x.unwrap() as f32, y: pos.y.unwrap() as f32 };

        self.framebuffer_mut().draw_text(pos, text.to_owned(), size, color::BLACK, false)
    }

    pub fn draw_rect(&mut self, pos: Point2<Option<i32>>, size: Vector2<u32>, border_px: u32,) -> mxcfb_rect {
        let mut pos = pos; 
        if pos.x.is_none() || pos.y.is_none() {
            if pos.x.is_none() {
                // Center vertically
                pos.x = Some(DISPLAYWIDTH as i32 / 2 - size.x as i32 / 2);
            }
            
            if pos.y.is_none() {
                // Center horizontally
                pos.y = Some(DISPLAYHEIGHT as i32 / 2 - size.y as i32 / 2);
            }
        }
        let pos = Point2 { x: pos.x.unwrap(), y: pos.y.unwrap() };

        self.framebuffer_mut().draw_rect(pos, size, border_px, color::BLACK);
        mxcfb_rect {
            top: pos.y as u32,
            left: pos.x as u32,
            width: size.x,
            height: size.y
        }
    }
}