use super::cloth_fabric::ParticleBufferObj;
use super::{BendingConstraintObj, MeshColoringObj, StretchConstraintObj};
use alloc::{vec, vec::Vec};

#[allow(dead_code)]
pub fn generate_bend_constraints(
    horizontal_num: i32,
    vertical_num: i32,
    _particles: &[ParticleBufferObj],
) -> (Vec<MeshColoringObj>, Vec<BendingConstraintObj>) {
    let particle_num = horizontal_num * vertical_num;
    let mut list: Vec<BendingConstraintObj> = Vec::with_capacity(particle_num as usize * 4);
    for h in 0..vertical_num {
        let offset_y = h * horizontal_num;
        for w in 0..horizontal_num {
            let v = offset_y + w;

            if w > 0 && w < (horizontal_num - 1) {
                list.push(BendingConstraintObj {
                    v,
                    b0: v - 1,
                    b1: v + 1,
                    h0: 0.0,
                });
            }
            if h > 0 && h < (vertical_num - 1) {
                list.push(BendingConstraintObj {
                    v,
                    b0: v - horizontal_num,
                    b1: v + horizontal_num,
                    h0: 0.0,
                });
                if w > 0 && w < (horizontal_num - 1) {
                    list.push(BendingConstraintObj {
                        v,
                        b0: v - horizontal_num - 1,
                        b1: v + horizontal_num + 1,
                        h0: 0.0,
                    });
                    list.push(BendingConstraintObj {
                        v,
                        b0: v - horizontal_num + 1,
                        b1: v + horizontal_num - 1,
                        h0: 0.0,
                    });
                }
            }
        }
    }
    let mut groups: Vec<Vec<BendingConstraintObj>> = vec![vec![list[0]]];
    let mut all_c_indices: Vec<usize> = vec![0];
    for i in 1..list.len() {
        let c = list[i];
        // gather constraints that shared at least one vertex
        let mut gathered_indices: Vec<usize> = vec![];
        let mut gathered_constraints: Vec<BendingConstraintObj> = vec![];
        for j in 1..(horizontal_num * 4 * 4) {
            let k = i as i32 - j;
            if k < 0 {
                break;
            }
            let c1 = &list[k as usize];
            if c.shared_vertices(c1) {
                gathered_indices.push(all_c_indices[k as usize]);
                gathered_constraints.push(*c1);
            }
        }

        find_a_group_to_join(&mut groups, &mut all_c_indices, &gathered_indices, c);
    }

    gen_coloring_and_flat_data::<BendingConstraintObj>(groups)
}

// Workshop on Virtual Reality Interaction and Physical Simulation VRIPHYS (2010):
// A Triangle Bending Constraint Model for Position-Based Dynamics
#[allow(dead_code)]
pub fn generate_bend_constraints2(
    horizontal_num: usize,
    vertical_num: usize,
    particles: &[ParticleBufferObj],
) -> (Vec<MeshColoringObj>, Vec<BendingConstraintObj>) {
    let particle_num = horizontal_num * vertical_num;

    // dihedral angle constraints
    // ┌─────────┬─────────┐
    // │\        │\     ///│
    // │ \       │ \ ///   │
    // │  \     /// \      │
    // │   \ //  │   \     │
    // │  /\     │   \     │
    // │/───\────┼────\────┤
    // │    \    │    \ ///│
    // │     \   │  ///\   │
    // │      \ ///     \  │
    // │     / \ │       \ │
    // │  /     \│        \│
    // │/────────┴─────────┘
    let mut list: Vec<BendingConstraintObj> = Vec::with_capacity(particle_num * 2);
    for h in 0..(vertical_num - 1) {
        let offset_y = h * horizontal_num;
        for w in 1..horizontal_num {
            if w + 1 < horizontal_num {
                let v = offset_y + w + 1;
                list.push(gen_bend_constraint(particles, horizontal_num, v, false));
                if h == 0 {
                    // 给顶边加一组水平约束
                    // let v = (offset_y + w) as i32;
                    // list.push(BendingConstraintObj {
                    //     v,
                    //     b0: v - 1,
                    //     b1: v + 1,
                    //     h0: 0.0,
                    // });
                }
            }
            if h == 0 {
                continue;
            }
            let v = offset_y + w - 1 - horizontal_num;
            list.push(gen_bend_constraint(particles, horizontal_num, v, true));
            // 给两侧加一组垂直约束
            // if h < (vertical_num - 1) && (w == 1 || w == (horizontal_num - 2)) {
            //     let v = (offset_y + w) as i32;
            //     list.push(BendingConstraintObj {
            //         v,
            //         b0: v - horizontal_num as i32,
            //         b1: v + horizontal_num as i32,
            //         h0: 0.0,
            //     });
            // }
        }
    }

    let mut groups: Vec<Vec<BendingConstraintObj>> = vec![vec![list[0]]];
    let mut all_c_indices: Vec<usize> = vec![0];
    for i in 1..list.len() {
        let c = list[i];
        // gather constraints that shared at least one vertex
        let mut gathered_indices: Vec<usize> = vec![];
        let mut gathered_constraints: Vec<BendingConstraintObj> = vec![];
        for j in 1..(horizontal_num * 6) {
            let k = i as i32 - j as i32;
            if k < 0 {
                break;
            }
            let c1 = &list[k as usize];
            if c.shared_vertices(c1) {
                gathered_indices.push(all_c_indices[k as usize]);
                gathered_constraints.push(*c1);
            }
        }

        find_a_group_to_join(&mut groups, &mut all_c_indices, &gathered_indices, c);
    }

    gen_coloring_and_flat_data::<BendingConstraintObj>(groups)
}

pub fn generate_stretch_constraints(
    horizontal_num: usize,
    vertical_num: usize,
    particles: &[ParticleBufferObj],
) -> (Vec<MeshColoringObj>, Vec<StretchConstraintObj>) {
    let particle_num = horizontal_num * vertical_num;
    let mut constraints: Vec<StretchConstraintObj> = Vec::with_capacity(particle_num * 4);

    for h in 0..vertical_num {
        let offset_y = h * horizontal_num;
        for w in 0..horizontal_num {
            let index0 = offset_y + w;
            if h == 0 {
                if w < horizontal_num - 1 {
                    let index1 = index0 + 1;
                    constraints.push(get_constraint(particles, index0, index1));
                }
                continue;
            }
            // top
            let top_index = index0 - horizontal_num;
            constraints.push(get_constraint(particles, index0, top_index));

            if w > 0 {
                let top_left = top_index - 1;
                constraints.push(get_constraint(particles, index0, top_left));
                constraints.push(get_constraint(particles, index0, index0 - 1));
                constraints.push(get_constraint(particles, top_index, index0 - 1));
            }
        }
    }
    group_distance_constraints(horizontal_num, &constraints)
}

fn gen_bend_constraint(
    particles: &[ParticleBufferObj],
    horizontal_num: usize,
    v: usize,
    is_verticle: bool,
) -> BendingConstraintObj {
    let b0 = if is_verticle {
        v + 2 * horizontal_num
    } else {
        v - 2
    };
    let b1 = if is_verticle {
        b0 + 1
    } else {
        b0 + horizontal_num
    };
    let h0 = get_h0(&particles[v], &particles[b0], &particles[b1]);
    BendingConstraintObj {
        v: v as i32,
        b0: b0 as i32,
        b1: b1 as i32,
        h0,
    }
}

fn group_distance_constraints(
    horizontal_num: usize,
    constraints: &[StretchConstraintObj],
) -> (Vec<MeshColoringObj>, Vec<StretchConstraintObj>) {
    // Graph Coloring
    // Identifies independent sets of constraints (those not shareed by vertices)
    // All the constraints assigned to the same group are solved with a single dispatch
    let mut groups: Vec<Vec<StretchConstraintObj>> = vec![vec![constraints[0]]];
    let mut all_c_indices: Vec<usize> = vec![0];

    // first row constraints count
    let first_row_num = horizontal_num - 1;
    for i in 1..constraints.len() {
        let c = constraints[i];
        if i < first_row_num {
            let gathered_indices = if all_c_indices.len() <= 2 {
                all_c_indices.clone()
            } else {
                all_c_indices[all_c_indices.len() - 2..].to_vec()
            };
            find_a_group_to_join(&mut groups, &mut all_c_indices, &gathered_indices, c);

            continue;
        }
        // gather constraints that shared at least one vertex
        let mut gathered_indices: Vec<usize> = vec![];
        // 先检查前两行中可能有共享顶点的约束
        for j in 1..(horizontal_num * 2 * 4) {
            if i < j {
                continue;
            }
            let c1 = &constraints[i - j];
            if c.shared_vertices(c1) {
                gathered_indices.push(all_c_indices[i - j]);
            }
        }
        find_a_group_to_join(&mut groups, &mut all_c_indices, &gathered_indices, c);
    }
    gen_coloring_and_flat_data::<StretchConstraintObj>(groups)
}

fn get_constraint(
    particles: &[ParticleBufferObj],
    index0: usize,
    index1: usize,
) -> StretchConstraintObj {
    let particle0 = &particles[index0];
    let particle1 = &particles[index1];

    let rest_length: f32 = minus_length(&particle0.pos, &particle1.pos);
    StretchConstraintObj {
        rest_length,
        lambda: 0.0,
        particle1: index1 as i32,
        particle0: index0 as i32,
    }
}

fn gen_coloring_and_flat_data<T>(constraints: Vec<Vec<T>>) -> (Vec<MeshColoringObj>, Vec<T>)
where
    T: bytemuck::Pod + bytemuck::Zeroable,
{
    let mut mesh_colorings: Vec<MeshColoringObj> = vec![];
    let mut offset = 0;
    for a_group in constraints.iter() {
        let group_len = a_group.len() as u32;
        mesh_colorings.push(MeshColoringObj {
            offset,
            max_num_x: 0,
            max_num_y: 0,
            group_len,
            thread_group: (((group_len + 31) as f32 / 32.0).floor() as u32, 1),
        });
        offset += group_len;
    }
    (mesh_colorings, constraints.into_iter().flatten().collect())
}

// Find the unused group number of the connected constraints to assign to the current constraint
// Usually, 8 groups are enough to color all constraints
fn find_a_group_to_join<T>(
    groups: &mut Vec<Vec<T>>,
    all_c_indices: &mut Vec<usize>,
    gathered_indices: &[usize],
    c: T,
) where
    T: bytemuck::Pod + bytemuck::Zeroable,
{
    for g in 0..16 {
        if !gathered_indices.contains(&g) {
            all_c_indices.push(g);
            if groups.len() <= g {
                groups.push(vec![]);
            }
            groups[g].push(c);
            break;
        }
    }
}

fn minus_length(lh: &[f32; 4], rh: &[f32; 4]) -> f32 {
    let x = lh[0] - rh[0];
    let y = lh[1] - rh[1];
    let z = lh[2] - rh[2];

    (x * x + y * y + z * z).sqrt()
}

const ONE_THIRD: f32 = 1.0 / 3.0;
fn get_h0(v: &ParticleBufferObj, b0: &ParticleBufferObj, b1: &ParticleBufferObj) -> f32 {
    // eq. 3
    let centroid = [
        (v.pos[0] + b0.pos[0] + b1.pos[0]) * ONE_THIRD,
        (v.pos[1] + b0.pos[1] + b1.pos[1]) * ONE_THIRD,
        (v.pos[2] + b0.pos[2] + b1.pos[2]) * ONE_THIRD,
        v.pos[3],
    ];
    minus_length(&v.pos, &centroid)
}
