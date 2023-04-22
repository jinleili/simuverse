use super::*;

#[allow(dead_code)]
impl BufferHandler {
    /// Creates a buffer handler from a slice.
    #[inline(always)]
    pub fn from_slice<T: Sized + Pod + Zeroable, A: AsRef<[T]>>(
        vec: &A,
        device: &Device,
        usage: BufferUsages,
        label: Option<&str>,
    ) -> Self {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(vec.as_ref()),
            usage,
            label,
        });
        let stride = std::mem::size_of::<T>() as u64;
        let size = vec.as_ref().len() as u64 * stride;
        BufferHandler {
            buffer,
            size,
            stride,
        }
    }
    /// Returns the reference of the buffer.
    #[inline(always)]
    pub const fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Returns the size of the buffer.
    #[inline(always)]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Creates a binding resource from buffer slice.
    #[inline(always)]
    pub const fn binding_resource(&self) -> BindingResource<'_> {
        BindingResource::Buffer(BufferBinding {
            buffer: &self.buffer,
            offset: 0,
            size: None,
        })
    }
}
