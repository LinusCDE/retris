pub use libremarkable::framebuffer::{
    common::mxcfb_rect,
    cgmath::Point2,
    cgmath::Vector2,
};

use libremarkable::framebuffer::{
    FramebufferBase,
    FramebufferDraw,
    FramebufferIO,
    FramebufferRefresh,
    core::Framebuffer,
    common::waveform_mode,
    common::display_temp,
    common::dither_mode,
    refresh::PartialRefreshMode,
    common::color,
    common::DISPLAYWIDTH,
    common::DISPLAYHEIGHT,
    common::DRAWING_QUANT_BIT,
    common::DRAWING_QUANT_BIT_2,
    common::DRAWING_QUANT_BIT_3,
};
use tetris_core::{Randomizer, Game, Size, Block};
use std::ops::DerefMut;
use std::collections::HashSet;

pub struct Canvas<'a> {
    game_size: Size,
    block_size: usize,
    framebuffer: Box::<Framebuffer<'a>>,
    last_block_positions: HashSet<Point2<i32>>,
}

impl<'a> Canvas<'a> {
    pub fn new(game_size: Size) -> Self {
        let mut instance = Self {
            game_size,
            block_size: 50,
            framebuffer: Box::new(Framebuffer::new("/dev/fb0")),
            last_block_positions: HashSet::new(),
        };
        instance.clear();
        instance.update_full();
        instance
    }

    pub fn framebuffer_mut(&mut self) -> &'static mut Framebuffer<'static> {
        unsafe {
            std::mem::transmute::<_, &'static mut Framebuffer<'static>>(
                self.framebuffer.deref_mut(),
            )
        }
    }

    fn clear(&mut self) {
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

    pub fn to_coords(&self, pos: (usize, usize)) -> (usize, usize) {
        let start_x = DISPLAYWIDTH as usize / 2 - self.game_size().width * self.block_size / 2;
        let start_y = DISPLAYHEIGHT as usize / 2 - self.game_size().height * self.block_size / 2;
        let offset = self.to_size(pos);
        (start_x + offset.0, start_y + offset.1)
    }
    
    pub fn to_size(&self, pos: (usize, usize)) -> (usize, usize) {
        (pos.0 * self.block_size, pos.1 * self.block_size)
    }

    pub fn draw_game(&mut self, game: &Game) {
        self.clear();
        {
            let point = self.field_start_i32();
            let vec = self.field_size();
            self.framebuffer_mut().draw_rect(point, vec, 1, color::BLACK);
        }

        for block in game.draw() {
            let block_start = self.to_coords((block.rect.origin.x as usize, block.rect.origin.y as usize));
            let block_size = self.to_size((block.rect.size.width as usize, block.rect.size.height as usize));
            self.framebuffer_mut().fill_rect(
                Point2 { x: block_start.0 as i32, y: block_start.1 as i32 },
                Vector2 { x: block_size.0 as u32, y: block_size.1 as u32 },
                color::BLACK
            );
        }

        let field_start = self.field_start_u32();
        let field_size = self.field_size();
        const FONT_SIZE: u32 = 40;
        self.framebuffer_mut().draw_text(
            Point2 { x: (field_start.x + 10) as f32, y: (field_start.y + field_size.y + FONT_SIZE + 5) as f32 },
            format!("Score: {}", game.get_score()),
            FONT_SIZE as f32,
            color::BLACK,
            false
        );

        let point = Point2 { x: self.field_start_u32().x - 5, y: self.field_start_u32().y - 5 };
        let vec = self.field_size() + Vector2 { x: 5 + 5, y: 5 + FONT_SIZE + 5 };
        self.update_partial(&mxcfb_rect::from(point, vec));
    }

    pub fn game_size(&self) -> Size {
        self.game_size.clone()
    }

    pub fn field_start_i32(&self) -> Point2<i32> {
        let field_start = self.to_coords((0, 0));
        Point2 { x: field_start.0 as i32, y: field_start.1 as i32 }
    }

    pub fn field_start_u32(&self) -> Point2<u32> {
        let field_start = self.to_coords((0, 0));
        Point2 { x: field_start.0 as u32, y: field_start.1 as u32 }
    }

    pub fn field_size(&self) -> Vector2<u32> {
        let field_size = self.to_size((self.game_size.width, self.game_size.height));
        Vector2 { x: field_size.0 as u32, y: field_size.1 as u32 }
    }

}