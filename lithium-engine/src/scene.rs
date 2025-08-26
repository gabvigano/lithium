use crate::ecs::components;

pub struct Camera {
    pos: components::Vec2,
    rel_pos: components::Vec2,
    screen_size: components::Rect,
}

impl Camera {
    pub fn new(rel_pos: components::Vec2, screen_size: components::Rect) -> Self {
        Camera {
            pos: components::Vec2 { x: 0.0, y: 0.0 },
            rel_pos: rel_pos,
            screen_size: screen_size,
        }
    }

    #[inline]
    pub fn pos(&self) -> components::Vec2 {
        self.pos
    }

    #[inline]
    pub fn rel_pos(&self) -> components::Vec2 {
        self.rel_pos
    }

    #[inline]
    pub fn screen_size(&self) -> components::Rect {
        self.screen_size.clone()
    }

    #[inline]
    pub fn update(&mut self, focus: components::Vec2) {
        self.pos.x = focus.x + self.rel_pos.x - (self.screen_size.width / 2.0);
        self.pos.y = focus.y + self.rel_pos.y - (self.screen_size.height / 2.0);
    }
}
