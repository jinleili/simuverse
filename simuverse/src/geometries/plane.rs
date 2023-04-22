#![allow(dead_code)]

use crate::util::vertex::{PosUv, PosUv2};
use app_surface::math::Rect;

pub struct Plane {
    width: f32,
    height: f32,
    x_offset: f32,
    y_offset: f32,
    h_segments: u32,
    v_segments: u32,
}

impl Plane {
    pub fn new(h_segments: u32, v_segments: u32) -> Self {
        Plane {
            width: 2.0,
            height: 2.0,
            x_offset: 0.0,
            y_offset: 0.0,
            h_segments,
            v_segments,
        }
    }

    pub fn new_by_pixel(width: f32, height: f32, h_segments: u32, v_segments: u32) -> Self {
        Plane {
            width,
            height,
            x_offset: 0.0,
            y_offset: 0.0,
            h_segments,
            v_segments,
        }
    }

    // 指定矩形坐标区域来生成平面对象
    pub fn new_by_rect(rect: Rect, h_segments: u32, v_segments: u32) -> Self {
        Plane {
            width: rect.width,
            height: rect.height,
            x_offset: rect.x,
            y_offset: rect.y,
            h_segments,
            v_segments,
        }
    }

    // 支持指定纹理区域
    pub fn generate_vertices_by_texcoord(&self, tex_rect: Rect) -> (Vec<PosUv>, Vec<u32>) {
        let segment_width = self.width / self.h_segments as f32;
        let segment_height = self.height / self.v_segments as f32;
        let h_gap = tex_rect.width / (self.h_segments as f32);
        let v_gap = tex_rect.height / (self.v_segments as f32);

        let mut vertices: Vec<PosUv> = Vec::new();

        // 从左下角开始，按列遍历
        // 下边的写法等同于 for (let h=0; h<(h_segments + 1); h++) {}
        for h in 0..=self.h_segments {
            let x: f32 = self.most_left_x() + segment_width * (h as f32);
            let tex_coord_u: f32 = tex_rect.x + h_gap * (h as f32);

            for v in 0..=self.v_segments {
                let y: f32 = self.most_bottom_y() + segment_height * (v as f32);
                let tex_coord_v: f32 = tex_rect.y + tex_rect.height - v_gap * (v as f32);
                vertices.push(PosUv {
                    pos: [x, y, 0.0],
                    uv: [tex_coord_u, tex_coord_v],
                });
            }
        }

        (vertices, self.get_element_indices())
    }

    //  最左边的 x 坐标
    fn most_left_x(&self) -> f32 {
        if self.x_offset != 0.0 {
            self.x_offset
        } else {
            -self.half_width()
        }
    }
    // 最下边的 y 坐标
    fn most_bottom_y(&self) -> f32 {
        if self.y_offset != 0.0 {
            self.y_offset - self.height
        } else {
            -self.half_height()
        }
    }

    // 支持指定纹理区域
    pub fn generate_vertices_by_texcoord2(
        &self,
        tex_rect: Rect,
        rect2: Option<Rect>,
    ) -> (Vec<PosUv2>, Vec<u32>) {
        let segment_width = self.width / self.h_segments as f32;
        let segment_height = self.height / self.v_segments as f32;
        let h_gap = tex_rect.width / (self.h_segments as f32);
        let v_gap = tex_rect.height / (self.v_segments as f32);
        let (h_gap1, v_gap1, rect2_x, rect2_y) = if let Some(rect2) = rect2 {
            (
                rect2.width / (self.h_segments as f32),
                rect2.height / (self.v_segments as f32),
                rect2.x,
                rect2.y,
            )
        } else {
            (
                1.0 / (self.h_segments as f32),
                1.0 / (self.v_segments as f32),
                0.0,
                0.0,
            )
        };

        let mut vertices: Vec<PosUv2> = Vec::new();

        // 从左下角开始，按列遍历
        // 下边的写法等同于 for (let h=0; h<(h_segments + 1); h++) {}
        for h in 0..=self.h_segments {
            let x: f32 = self.most_left_x() + segment_width * (h as f32);
            let tex_coord_u: f32 = tex_rect.x + h_gap * (h as f32);
            let tex_coord_u1: f32 = rect2_x + h_gap1 * (h as f32);

            for v in 0..=self.v_segments {
                let y: f32 = self.most_bottom_y() + segment_height * (v as f32);
                let tex_coord_v: f32 = tex_rect.y + v_gap * (self.v_segments - v) as f32;
                let tex_coord_v1: f32 = rect2_y + v_gap1 * (self.v_segments - v) as f32;

                vertices.push(PosUv2 {
                    pos: [x, y, 0.0],
                    uv0: [tex_coord_u, tex_coord_v],
                    uv1: [tex_coord_u1, tex_coord_v1],
                });
            }
        }

        (vertices, self.get_element_indices())
        // (vertices, self.get_line_indices())
    }

    pub fn generate_vertices(&self) -> (Vec<PosUv>, Vec<u32>) {
        self.generate_vertices_by_texcoord(Rect::from_origin_n_size(0.0, 0.0, 1.0, 1.0))
    }

    // 返回的是线段列表，而不是线段条带
    pub fn get_line_indices(&self) -> Vec<u32> {
        let mut indices: Vec<u32> = Vec::new();
        // 2 个线段有 3 个结点，所以需要 (self.v_segments + 1) * h
        let v_point_num = self.v_segments + 1;
        // 按列遍历
        for h in 0..=self.h_segments {
            if h == 0 {
                for v in 1..=self.v_segments {
                    let current: u32 = v;
                    // 找到同一行上个位置的索引
                    indices.push(current - 1);
                    indices.push(current);
                }
                continue;
            }
            let num: u32 = v_point_num * h;
            for v in 0..=self.v_segments {
                let current: u32 = num + v;
                // 找到上一列同一行位置的索引
                let left: u32 = current - v_point_num;
                if v == 0 {
                    indices.push(left);
                    indices.push(current);
                } else {
                    let mut lines: Vec<u32> =
                        vec![current, left, current, left - 1, current, current - 1];
                    indices.append(&mut lines);
                }
            }
        }

        indices
    }

    pub fn get_element_indices(&self) -> Vec<u32> {
        let mut indices: Vec<u32> = Vec::new();
        // 2 个线段有 3 个结点，所以需要 (self.v_segments + 1) * h
        let v_point_num = self.v_segments + 1;
        // 按列遍历
        for h in 1..=self.h_segments {
            let num: u32 = v_point_num * h;
            for v in 1..(self.v_segments + 1) {
                let current: u32 = num + v;
                // 找到上一列同一行位置的索引
                let left: u32 = current - v_point_num;
                let mut lines: Vec<u32> =
                    vec![current, left, left - 1, current, left - 1, current - 1];
                indices.append(&mut lines);
            }
        }

        indices
    }

    fn half_width(&self) -> f32 {
        self.width / 2.0
    }

    fn half_height(&self) -> f32 {
        self.height / 2.0
    }
}
