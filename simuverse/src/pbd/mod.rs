use alloc::{vec, vec::Vec};
use core::fmt::Debug;

mod cloth_fabric;
pub use cloth_fabric::ClothFabric;

mod point3d;

mod gen_cloth_constraints;

mod cloth;
use cloth::Cloth;

mod pbd_simulator;
pub use pbd_simulator::PBDSimulator;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrameUniform {
    // 帧绘制计数
    frame_index: i32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ClothUniform {
    // 粒子个数
    num_x: i32,
    num_y: i32,
    gravity: f32,
    damping: f32,
    compliance: f32,
    stiffness: f32,
    dt: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BinUniform {
    // bin hash 容器数
    bin_num: [i32; 4],
    // 容器各轴向上最大的索引数
    bin_max_index: [i32; 4],
    bin_size: [f32; 4],
    // 转换到 【0～n]坐标空间需要的偏移
    pos_offset: [f32; 4],
    max_bin_count: i32,
    padding: [f32; 3],
}

// 拉伸 | 距离约束
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StretchConstraintObj {
    pub rest_length: f32,
    pub lambda: f32,
    pub particle0: i32,
    pub particle1: i32,
}

impl StretchConstraintObj {
    pub fn shared_vertices(&self, other: &StretchConstraintObj) -> bool {
        if self.particle0 == other.particle0
            || self.particle0 == other.particle1
            || self.particle1 == other.particle0
            || self.particle1 == other.particle1
        {
            return true;
        }
        false
    }
}

// 弯曲约束
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BendingConstraintObj {
    pub v: i32,
    pub b0: i32,
    pub b1: i32,
    pub h0: f32,
}

impl BendingConstraintObj {
    pub fn shared_vertices(&self, other: &BendingConstraintObj) -> bool {
        let list0 = [self.v, self.b0, self.b1];
        let list1 = [other.v, other.b0, other.b1];
        for i in list1.iter() {
            if list0.contains(i) {
                return true;
            }
        }
        false
    }
}

impl Debug for BendingConstraintObj {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "({}, {}, {})", self.v, self.b0, self.b1)
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct BendingDynamicUniform {
    offset: i32,
    max_num_x: i32,
    // 当前 mesh coloring 分组的数据长度
    group_len: i32,
    // 迭代計數的倒數
    invert_iter: f32,
}

// 约束的网络着色分组
#[derive(Debug)]
pub struct MeshColoringObj {
    pub offset: u32,
    pub max_num_x: u32,
    pub max_num_y: u32,
    pub group_len: u32,
    pub thread_group: (u32, u32),
}

impl MeshColoringObj {
    pub fn get_push_constants_data(&self) -> Vec<u32> {
        vec![self.offset, self.max_num_x, self.max_num_y, self.group_len]
    }

    pub fn get_bending_dynamic_uniform(&self, iter_count: i32) -> BendingDynamicUniform {
        BendingDynamicUniform {
            offset: self.offset as i32,
            max_num_x: self.max_num_x as i32,
            group_len: self.group_len as i32,
            invert_iter: 1.0 / (iter_count + 1) as f32,
        }
    }
}
