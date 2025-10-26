use crate::math;

pub struct Camera {
    pos: math::Vec2,
    rel_pos: math::Vec2,
    screen_size: math::Rect,
}

impl Camera {
    pub fn new(rel_pos: math::Vec2, screen_size: math::Rect) -> Self {
        Camera {
            pos: math::Vec2 { x: 0.0, y: 0.0 },
            rel_pos: rel_pos,
            screen_size: screen_size,
        }
    }

    #[inline]
    pub fn pos(&self) -> math::Vec2 {
        self.pos
    }

    #[inline]
    pub fn rel_pos(&self) -> math::Vec2 {
        self.rel_pos
    }

    #[inline]
    pub fn screen_size(&self) -> math::Rect {
        self.screen_size.clone()
    }

    #[inline]
    pub fn update(&mut self, focus: math::Vec2) {
        self.pos.x = focus.x + self.rel_pos.x - (self.screen_size.width / 2.0);
        self.pos.y = focus.y + self.rel_pos.y - (self.screen_size.height / 2.0);
    }
}
