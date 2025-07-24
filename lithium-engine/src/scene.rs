use crate::ecs::components;

pub struct Camera {
    pos: components::Pos,
    rel_pos: components::Pos,
    screen_size: components::Rect,
}

impl Camera {
    pub fn new(rel_pos: components::Pos, screen_size: components::Rect) -> Self {
        Camera {
            pos: components::Pos { x: 0.0, y: 0.0 },
            rel_pos: rel_pos,
            screen_size: screen_size,
        }
    }

    pub fn pos(&self) -> &components::Pos {
        &self.pos
    }

    pub fn rel_pos(&self) -> &components::Pos {
        &self.rel_pos
    }

    pub fn screen_size(&self) -> &components::Rect {
        &self.screen_size
    }

    pub fn update(&mut self, focus: &components::Pos) {
        self.pos.x = focus.x + self.rel_pos.x - (self.screen_size.width / 2.0);
        self.pos.y = focus.y + self.rel_pos.y - (self.screen_size.height / 2.0);
    }
}
