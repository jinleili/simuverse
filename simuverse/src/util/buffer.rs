use bytemuck::Pod;
use wgpu::util::DeviceExt;

pub struct BufferObj {
    pub buffer: wgpu::Buffer,
    pub size: wgpu::BufferAddress,
    pub min_binding_size: Option<wgpu::BufferSize>,
    pub has_dynamic_offset: bool,
    pub read_only: bool,
}

#[allow(dead_code)]
impl BufferObj {
    pub fn create_storage_buffer<T>(
        device: &wgpu::Device,
        slice: &[T],
        label: Option<&'static str>,
    ) -> Self
    where
        T: 'static + Pod + Copy,
    {
        BufferObj::create_buffer(
            device,
            Some(slice),
            None,
            wgpu::BufferUsages::STORAGE,
            label,
        )
    }

    pub fn create_empty_storage_buffer(
        device: &wgpu::Device,
        size: wgpu::BufferAddress,
        can_read_back: bool,
        label: Option<&'static str>,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size,
            usage: if can_read_back {
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC
            } else {
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
            },
            label,
            mapped_at_creation: false,
        });
        BufferObj {
            buffer,
            size,
            min_binding_size: None,
            has_dynamic_offset: false,
            read_only: false,
        }
    }

    pub fn create_empty_uniform_buffer(
        device: &wgpu::Device,
        size: wgpu::BufferAddress,
        min_binding_size: u64,
        is_dynamic: bool,
        label: Option<&'static str>,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            label,
            mapped_at_creation: false,
        });
        BufferObj {
            buffer,
            size,
            min_binding_size: wgpu::BufferSize::new(min_binding_size),
            has_dynamic_offset: is_dynamic,
            read_only: true,
        }
    }

    pub fn create_uniform_buffer<T>(
        device: &wgpu::Device,
        uniform: &T,
        label: Option<&'static str>,
    ) -> Self
    where
        T: 'static + Pod + Copy,
    {
        BufferObj::create_buffer(
            device,
            None,
            Some(uniform),
            wgpu::BufferUsages::UNIFORM,
            label,
        )
    }

    pub fn create_uniforms_buffer<T>(
        device: &wgpu::Device,
        slice: &[T],
        label: Option<&'static str>,
    ) -> Self
    where
        T: 'static + Pod + Copy,
    {
        BufferObj::create_buffer(
            device,
            Some(slice),
            None,
            wgpu::BufferUsages::UNIFORM,
            label,
        )
    }

    pub fn create_buffer<T>(
        device: &wgpu::Device,
        slice: Option<&[T]>,
        item: Option<&T>,
        usage: wgpu::BufferUsages,
        label: Option<&'static str>,
    ) -> Self
    where
        T: 'static + Pod + Copy,
    {
        let min_binding_size = core::mem::size_of::<T>() as wgpu::BufferAddress;
        let mut size = min_binding_size;
        let data: &[u8] = if let Some(slice) = slice {
            size *= slice.len() as wgpu::BufferAddress;
            bytemuck::cast_slice(slice)
        } else {
            bytemuck::bytes_of(item.unwrap())
        };
        // 移除staging buffer
        // 移动GPU通常是统一内存架构。这一内存架构下，CPU可以直接访问GPU所使用的内存
        // if cfg!(any(target_os = "ios", target_os = "android")) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: data,
            usage: usage | wgpu::BufferUsages::COPY_DST,
        });
        BufferObj {
            buffer,
            size,
            min_binding_size: wgpu::BufferSize::new(min_binding_size),
            has_dynamic_offset: false,
            read_only: false,
        }
    }
}
