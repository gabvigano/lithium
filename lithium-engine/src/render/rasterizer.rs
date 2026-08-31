use crate::math::ApplyTransformationShape;
use crate::render::{BoundingBox, ToBoundingBox};
use crate::{base, ecs, math, render};

pub struct FrameBuffer {
    size: (usize, usize),
    buffer: Vec<u32>,
    areas_to_clean: Vec<BoundingBox>,
}

impl FrameBuffer {
    #[inline]
    pub fn new(size: (usize, usize), color: u32) -> Self {
        Self {
            size,
            buffer: vec![color; size.0 * size.1],
            areas_to_clean: vec![],
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.size.0
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.size.1
    }

    #[inline]
    pub fn size(&self) -> (usize, usize) {
        (self.size.0, self.size.1)
    }

    #[inline]
    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    #[inline]
    pub fn idx_at_pos(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.size.0 || y >= self.size.1 {
            return None;
        }

        y.checked_mul(self.size.0)?.checked_add(x)
    }

    #[inline]
    pub fn row_at_y(&self, y: usize) -> Option<&[u32]> {
        if y >= self.size.1 {
            return None;
        }

        let start_idx = y.checked_mul(self.size.0)?;
        let end_idx = start_idx.checked_add(self.size.0)?.min(self.buffer.len());

        self.buffer.get(start_idx..end_idx)
    }

    #[inline]
    pub fn row_at_y_mut(&mut self, y: usize) -> Option<&mut [u32]> {
        if y >= self.size.1 {
            return None;
        }

        let start_idx = y.checked_mul(self.size.0)?;
        let end_idx = start_idx.checked_add(self.size.0)?.min(self.buffer.len());

        self.buffer.get_mut(start_idx..end_idx)
    }

    #[inline]
    pub fn fill(&mut self, color: u32) {
        self.buffer.fill(color)
    }

    fn rasterize_convex_edges(&mut self, edges: &[(math::Vec2, math::Vec2)], bounding_box: &BoundingBox, color: u32) {
        for y in bounding_box.min_y..bounding_box.max_y {
            let centered_y = y as f32 + 0.5;

            let mut left = f32::INFINITY;
            let mut right = f32::NEG_INFINITY;
            let mut intersection_count = 0;

            for &(start, end) in edges {
                let min_y = start.y.min(end.y);
                let max_y = start.y.max(end.y);

                // skip if edge doesn't intersect
                if centered_y < min_y || centered_y >= max_y {
                    continue;
                }

                // evaluate x at intersection
                let x = start.x + (centered_y - start.y) * (end.x - start.x) / (end.y - start.y);

                left = left.min(x);
                right = right.max(x);

                intersection_count += 1;
            }

            if intersection_count < 2 {
                continue;
            }

            let start_x = ((left - 0.5).ceil() as usize).max(bounding_box.min_x);
            let end_x = ((right - 0.5).ceil() as usize).min(bounding_box.max_x);

            if start_x < end_x {
                if let Some(row) = self.row_at_y_mut(y) {
                    row[start_x..end_x].fill(color);
                }
            }
        }
    }

    pub fn rasterize_shape(&mut self, shape: &math::Shape, color: u32) -> Result<(), base::GeometryError> {
        fn compute_cvx_poly_edges(cvx_poly: &math::CvxPoly) -> Vec<(math::Vec2, math::Vec2)> {
            cvx_poly
                .verts()
                .iter()
                .enumerate()
                .map(|(idx, v)| (*v, cvx_poly.verts()[(idx + 1) % cvx_poly.verts().len()]))
                .collect()
        }

        let Some(bounding_box) = shape.to_bounding_box().crop_in_framebuffer(self) else {
            return Ok(());
        };

        match shape {
            math::Shape::Segment(_) => unimplemented!(),
            math::Shape::Triangle(math::Triangle { a, b, c }) => {
                self.rasterize_convex_edges(&[(*a, *b), (*b, *c), (*c, *a)], &bounding_box, color)
            }
            math::Shape::Quad(math::Quad { a, b, c, d }) => {
                self.rasterize_convex_edges(&[(*a, *b), (*b, *c), (*c, *d), (*d, *a)], &bounding_box, color)
            }
            math::Shape::CvxPoly(cvx_poly) => {
                self.rasterize_convex_edges(&compute_cvx_poly_edges(cvx_poly), &bounding_box, color);
            }
            math::Shape::CavePoly(cave_poly) => {
                for cvx_poly in cave_poly.cvx_polys()? {
                    self.rasterize_convex_edges(&compute_cvx_poly_edges(cvx_poly), &bounding_box, color);
                }
            }
            math::Shape::Circle(_) => unimplemented!(),
        }

        self.areas_to_clean.push(bounding_box);

        Ok(())
    }

    pub fn rasterize_all<const N: usize>(&mut self, world: &ecs::World<N>, camera: &render::Camera) -> Result<(), base::GeometryError> {
        let materials = world.engine.material.get_comps();
        let ents = world.engine.material.get_ents(); // entities implementing material
        let mut pairs: Vec<(&ecs::Material, &u32)> = materials.iter().zip(ents).collect();

        // sort by layer
        pairs.sort_by_key(|(m, _)| m.layer);

        for (material, &entity) in pairs {
            if !material.show {
                continue;
            }

            let Some(&ecs::Transform { pos, .. }) = world.engine.transform.get(entity) else {
                continue;
            };
            let Some(body) = world.engine.body.get(entity) else {
                continue;
            };

            let color_hex = material.color().to_hex();

            let transformed_shape = match world.engine.rotation_matrix.get(entity) {
                Some(ecs::RotationMatrix { rot_mat: rm }) => body.shape.apply_mat2x3_then_vec2_unchecked(pos.sub(camera.pos()), rm),
                None => body.shape.apply_vec2_unchecked(pos.sub(camera.pos())),
            };

            self.rasterize_shape(&transformed_shape, color_hex)?;
        }

        Ok(())
    }

    pub fn rasterize_text(&mut self, text: &str, pos: (usize, usize), scale: usize, color: u32) {
        let glyph_size = render::GLYPH_SIZE * scale;

        if scale == 0 || pos.0 + glyph_size > self.size.0 || pos.1 + glyph_size > self.size.1 {
            return;
        }

        let mut current_pos = pos;
        let mut line_start = pos;

        for character in text.to_lowercase().bytes() {
            let new_line = character == b'\n';
            if new_line || current_pos.0 + glyph_size > self.size.0 {
                // \n or character is outside the screen

                // add bounding box for the last line
                if current_pos.0 > line_start.0 {
                    if let Some(bounding_box) = (BoundingBox {
                        min_x: line_start.0,
                        min_y: line_start.1,
                        max_x: current_pos.0,
                        max_y: line_start.1 + glyph_size,
                    })
                    .crop_in_framebuffer(self)
                    {
                        self.areas_to_clean.push(bounding_box);
                    }
                }

                if current_pos.1 + glyph_size * 2 > self.height() {
                    return;
                }

                current_pos.0 = pos.0;
                current_pos.1 += glyph_size;
                line_start.1 = current_pos.1;

                if new_line {
                    continue;
                }
            }

            let glyph = render::match_glyph(character);

            if scale == 1 {
                for glyph_y in 0..render::GLYPH_SIZE {
                    let bits = glyph[glyph_y];

                    if bits == 0 {
                        // row is empty
                        continue;
                    }

                    let start_idx = (current_pos.1 + glyph_y) * self.width() + current_pos.0;
                    let row = &mut self.buffer_mut()[start_idx..start_idx + render::GLYPH_SIZE];

                    for glyph_x in 0..render::GLYPH_SIZE {
                        if bits & (0x80 >> glyph_x) != 0 {
                            row[glyph_x] = color;
                        }
                    }
                }
            } else {
                for glyph_y in 0..render::GLYPH_SIZE {
                    let bits = glyph[glyph_y];

                    if bits == 0 {
                        // row is empty
                        continue;
                    }

                    for scale_y in 0..scale {
                        let y = current_pos.1 + glyph_y * scale + scale_y;
                        let start_idx = y * self.width() + current_pos.0;
                        let row = &mut self.buffer_mut()[start_idx..start_idx + glyph_size];

                        for source_x in 0..render::GLYPH_SIZE {
                            if bits & (0x80 >> source_x) != 0 {
                                let x = source_x * scale;
                                row[x..x + scale].fill(color);
                            }
                        }
                    }
                }
            }

            current_pos.0 += glyph_size;
        }

        if current_pos.0 > line_start.0 {
            if let Some(bounding_box) = (BoundingBox {
                min_x: line_start.0,
                min_y: line_start.1,
                max_x: current_pos.0,
                max_y: line_start.1 + glyph_size,
            })
            .crop_in_framebuffer(self)
            {
                self.areas_to_clean.push(bounding_box);
            }
        }
    }

    #[inline]
    pub fn clean_screen(&mut self, color: u32) {
        let areas_to_clean = std::mem::take(&mut self.areas_to_clean); // this already clears areas_to_clean vector

        for bounding_box in areas_to_clean {
            for y in bounding_box.min_y..bounding_box.max_y {
                if let Some(row) = self.row_at_y_mut(y) {
                    row[bounding_box.min_x..bounding_box.max_x].fill(color);
                }
            }
        }
    }
}
