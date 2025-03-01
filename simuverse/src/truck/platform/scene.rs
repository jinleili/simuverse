use super::*;
use ::core::sync::atomic::{AtomicUsize, Ordering};

static MAXID: AtomicUsize = AtomicUsize::new(0);

impl RenderID {
    /// Generate the unique `RenderID`.
    #[inline(always)]
    pub fn gen_id() -> Self {
        RenderID(MAXID.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for StudioConfig {
    #[inline(always)]
    fn default() -> StudioConfig {
        StudioConfig {
            background: Color::BLACK,
            camera: Camera::default(),
            lights: vec![Light::default()],
        }
    }
}

impl Default for BackendBufferConfig {
    #[inline(always)]
    fn default() -> BackendBufferConfig {
        BackendBufferConfig {
            depth_test: true,
            sample_count: 1,
        }
    }
}

impl Default for RenderTextureConfig {
    #[inline(always)]
    fn default() -> RenderTextureConfig {
        RenderTextureConfig {
            canvas_size: (1024, 768),
            format: TextureFormat::Rgba8Unorm,
        }
    }
}

impl SceneDescriptor {
    /// Creates a `UNIFORM` buffer of camera.
    ///
    /// The bind group provides [`Scene`] holds this uniform buffer.
    ///
    /// # Shader Example
    /// ```glsl
    /// layout(set = 0, binding = 0) uniform Camera {
    ///     mat4 camera_matrix;     // the camera matrix
    ///     mat4 camera_projection; // the projection into the normalized view volume
    /// };
    /// ```
    #[inline(always)]
    pub fn camera_buffer(&self, device: &Device) -> BufferHandler {
        let (width, height) = self.render_texture.canvas_size;
        let as_rat = width as f64 / height as f64;
        self.studio.camera.buffer(as_rat, device)
    }

    /// Creates a `STORAGE` buffer of all lights.
    ///
    /// The bind group provides [`Scene`] holds this uniform buffer.
    ///
    /// # Shader Example
    /// ```glsl
    /// struct Light {
    ///     vec4 position;      // the position of light, position.w == 1.0
    ///     vec4 color;         // the color of light, color.w == 1.0
    ///     uvec4 light_type;   // Point => uvec4(0, 0, 0, 0), Uniform => uvec4(1, 0, 0, 0)
    /// };
    ///
    /// layout(set = 0, binding = 1) buffer Lights {
    ///     Light lights[];
    /// };
    /// ```
    #[inline(always)]
    pub fn lights_buffer(&self, device: &Device) -> BufferHandler {
        let mut light_vec: Vec<_> = self.studio.lights.iter().map(Light::light_info).collect();
        light_vec.resize(LIGHT_MAX, LightInfo::zeroed());
        BufferHandler::from_slice(
            &light_vec,
            device,
            BufferUsages::UNIFORM,
            Some("lights_buffer"),
        )
    }

    #[inline(always)]
    fn sampling_buffer(
        device: &Device,
        render_texture: RenderTextureConfig,
        sample_count: u32,
    ) -> Texture {
        device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: render_texture.canvas_size.0,
                height: render_texture.canvas_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: TextureDimension::D2,
            format: render_texture.format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        })
    }

    #[inline(always)]
    fn depth_texture(device: &Device, size: (u32, u32), sample_count: u32) -> Texture {
        device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        })
    }

    fn backend_buffers(&self, device: &Device) -> (Option<Texture>, Option<Texture>) {
        let foward_depth = if self.backend_buffer.depth_test {
            Some(Self::depth_texture(
                device,
                self.render_texture.canvas_size,
                self.backend_buffer.sample_count,
            ))
        } else {
            None
        };
        let sampling_buffer = if self.backend_buffer.sample_count > 1 {
            Some(Self::sampling_buffer(
                device,
                self.render_texture,
                self.backend_buffer.sample_count,
            ))
        } else {
            None
        };
        (foward_depth, sampling_buffer)
    }
}

/// Mutable reference of `SceneDescriptor` in `Scene`.
///
/// When this struct is dropped, the backend buffers of scene will be updated.
pub struct SceneDescriptorMut<'a>(&'a mut Scene);

impl ::core::ops::Deref for SceneDescriptorMut<'_> {
    type Target = SceneDescriptor;
    #[inline(always)]
    fn deref(&self) -> &SceneDescriptor {
        &self.0.scene_desc
    }
}

impl ::core::ops::DerefMut for SceneDescriptorMut<'_> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut SceneDescriptor {
        &mut self.0.scene_desc
    }
}

impl Drop for SceneDescriptorMut<'_> {
    fn drop(&mut self) {
        let (forward_depth, sampling_buffer) = self.backend_buffers(self.0.device());
        self.0.foward_depth = forward_depth;
        self.0.sampling_buffer = sampling_buffer;
    }
}

impl Scene {
    #[inline(always)]
    fn camera_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    #[inline(always)]
    fn lights_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    #[inline(always)]
    fn scene_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    #[inline(always)]
    fn init_scene_bind_group_layout(device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(
            device,
            &[
                Self::camera_bgl_entry(),
                Self::lights_bgl_entry(),
                Self::scene_bgl_entry(),
            ],
        )
    }

    /// constructor
    // About `scene_desc`, entity is better than reference for the performance.
    // This is reference because only for as wgpu is.
    #[inline(always)]
    pub fn new(app: &AppSurface, scene_desc: &SceneDescriptor) -> Scene {
        let device_handler = DeviceHandler {
            device: app.device.clone(),
            queue: app.queue.clone(),
        };
        let (foward_depth, sampling_buffer) = scene_desc.backend_buffers(&device_handler.device);
        let bind_group_layout = Self::init_scene_bind_group_layout(&device_handler.device);
        Scene {
            device_handler,
            objects: Default::default(),
            bind_group: None,
            bind_group_layout,
            foward_depth,
            sampling_buffer,
            clock: instant::Instant::now(),
            scene_desc: scene_desc.clone(),
        }
    }

    /// Returns the reference of its own `AppSurface`.
    #[inline(always)]
    pub fn device_handler(&self) -> &DeviceHandler {
        &self.device_handler
    }

    /// Returns the reference of the device.
    #[inline(always)]
    pub fn device(&self) -> &Device {
        &self.device_handler.device
    }

    /// Returns the elapsed time since the scene was created.
    #[inline(always)]
    pub fn elapsed(&self) -> ::core::time::Duration {
        self.clock.elapsed()
    }

    /// Returns the mutable reference of the descriptor.
    ///
    /// # Remarks
    ///
    /// When the return value is dropped, the depth buffer and sampling buffer are automatically updated.
    /// Use `studio_config_mut` if you only want to update the colors of the camera, lights, and background.
    #[inline(always)]
    pub fn descriptor_mut(&mut self) -> SceneDescriptorMut<'_> {
        SceneDescriptorMut(self)
    }

    /// Returns the mutable reference of the studio configuration.
    #[inline(always)]
    pub fn studio_config_mut(&mut self) -> &mut StudioConfig {
        &mut self.scene_desc.studio
    }

    /// Creates a `UNIFORM` buffer of the camera.
    ///
    /// The bind group provides [`Scene`] holds this uniform buffer.
    ///
    /// # Shader Example
    /// ```glsl
    /// layout(set = 0, binding = 0) uniform Camera {
    ///     mat4 camera_matrix;     // the camera matrix
    ///     mat4 camera_projection; // the projection into the normalized view volume
    /// };
    /// ```
    #[inline(always)]
    pub fn camera_buffer(&self) -> BufferHandler {
        self.scene_desc.camera_buffer(self.device())
    }

    /// Creates a `STORAGE` buffer of all lights.
    ///
    /// The bind group provides [`Scene`] holds this uniform buffer.
    ///
    /// # Shader Example
    /// ```glsl
    /// struct Light {
    ///     vec4 position;      // the position of light, position.w == 1.0
    ///     vec4 color;         // the color of light, color.w == 1.0
    ///     uvec4 light_type;   // Point => uvec4(0, 0, 0, 0), Uniform => uvec4(1, 0, 0, 0)
    /// };
    ///
    /// layout(set = 0, binding = 1) buffer Lights {
    ///     Light lights[]; // the number of lights must be gotten from another place
    /// };
    /// ```
    #[inline(always)]
    pub fn lights_buffer(&self) -> BufferHandler {
        self.scene_desc.lights_buffer(self.device())
    }

    /// Creates a `UNIFORM` buffer of the scene status.
    ///
    /// The bind group provides [`Scene`] holds this uniform buffer.
    ///
    /// # Shader Example
    /// ```glsl
    /// layout(set = 0, binding = 2) uniform Scene {
    ///     vec4 bk_color;  // color of back ground
    ///     float time;     // elapsed time since the scene was created.
    ///     uint nlights;   // the number of lights
    /// };
    /// ```
    #[inline(always)]
    pub fn scene_status_buffer(&self) -> BufferHandler {
        let bk = self.scene_desc.studio.background;
        let size = self.scene_desc.render_texture.canvas_size;
        let scene_info = SceneInfo {
            background_color: [bk.r as f32, bk.g as f32, bk.b as f32, bk.a as f32],
            resolution: [size.0, size.1],
            time: self.elapsed().as_secs_f32(),
            num_of_lights: self.scene_desc.studio.lights.len() as u32,
        };
        BufferHandler::from_slice(
            &[scene_info],
            self.device(),
            BufferUsages::UNIFORM,
            Some("scene_info"),
        )
    }

    /// Creates bind group.
    /// # Shader Examples
    /// Suppose binded as `set = 0`.
    /// ```glsl
    /// layout(set = 0, binding = 0) uniform Camera {
    ///     mat4 camera_matrix;     // the camera matrix
    ///     mat4 camera_projection; // the projection into the normalized view volume
    /// };
    ///
    /// struct Light {
    ///     vec4 position;      // the position of light, position.w == 1.0
    ///     vec4 color;         // the color of light, color.w == 1.0
    ///     uvec4 light_type;   // Point => uvec4(0, 0, 0, 0), Uniform => uvec4(1, 0, 0, 0)
    /// };
    ///
    /// layout(set = 0, binding = 1) buffer Lights {
    ///     Light lights[];
    /// };
    ///
    /// layout(set = 0, binding = 2) uniform Scene {
    ///     float time;     // elapsed time since the scene was created.
    ///     uint nlights;   // the number of lights
    /// };
    /// ```
    #[inline(always)]
    pub fn reset_bind_group(&mut self) {
        self.bind_group = Some(bind_group_util::create_bind_group(
            self.device(),
            &self.bind_group_layout,
            vec![
                self.camera_buffer().binding_resource(),
                self.lights_buffer().binding_resource(),
                self.scene_status_buffer().binding_resource(),
            ],
        ));
    }

    /// Adds a render object to the scene.
    ///
    /// If there already exists a render object with the same ID,
    /// replaces the render object and returns false.
    #[inline(always)]
    pub fn add_object<R: Rendered>(&mut self, object: &R) -> bool {
        let render_object = object.render_object(self);
        self.objects
            .insert(object.render_id(), render_object)
            .is_none()
    }
    /// Sets the visibility of a render object.
    ///
    /// If there does not exist the render object in the scene, does nothing and returns `false`.
    #[inline(always)]
    pub fn set_visibility<R: Rendered>(&mut self, object: &R, visible: bool) -> bool {
        self.objects
            .get_mut(&object.render_id())
            .map(|obj| obj.visible = visible)
            .is_some()
    }
    /// Adds render objects to the scene.
    ///
    /// If there already exists a render object with the same ID,
    /// replaces the render object and returns false.
    #[inline(always)]
    #[allow(dead_code)]
    pub fn add_objects<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>,
    {
        let closure = move |flag, object| flag && self.add_object(object);
        objects.into_iter().fold(true, closure)
    }
    /// Removes a render object from the scene.
    ///
    /// If there does not exist the render object in the scene, does nothing and returns `false`.
    #[inline(always)]
    #[allow(dead_code)]
    pub fn remove_object<R: Rendered>(&mut self, object: &R) -> bool {
        self.objects.remove(&object.render_id()).is_some()
    }
    /// Removes render objects from the scene.
    ///
    /// If there exists a render object which does not exist in the scene, returns `false`.
    #[inline(always)]
    #[allow(dead_code)]
    pub fn remove_objects<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>,
    {
        let closure = move |flag, object| flag && self.remove_object(object);
        objects.into_iter().fold(true, closure)
    }

    /// Removes all render objects from the scene.
    #[inline(always)]
    #[allow(dead_code)]
    pub fn clear_objects(&mut self) {
        self.objects.clear()
    }

    /// Returns the number of the render objects in the scene.
    #[inline(always)]
    #[allow(dead_code)]
    pub fn number_of_objects(&self) -> usize {
        self.objects.len()
    }

    /// Synchronizes the information of vertices of `object` in the CPU memory
    /// and that in the GPU memory.
    ///
    /// If there does not exist the render object in the scene, does nothing and returns false.
    #[inline(always)]
    pub fn update_vertex_buffer<R: Rendered>(&mut self, object: &R) -> bool {
        let (handler, objects) = (&self.device_handler, &mut self.objects);
        match objects.get_mut(&object.render_id()) {
            None => false,
            Some(render_object) => {
                let (vb, ib) = object.vertex_buffer(handler);
                render_object.vertex_buffer = vb;
                render_object.index_buffer = ib;
                true
            }
        }
    }

    /// Synchronizes the information of bind group of `object` in the CPU memory
    /// and that in the GPU memory.
    ///
    /// If there does not exist the render object in the scene, does nothing and returns false.
    #[inline(always)]
    pub fn update_bind_group<R: Rendered>(&mut self, object: &R) -> bool {
        let (handler, objects) = (&self.device_handler, &mut self.objects);
        match objects.get_mut(&object.render_id()) {
            Some(render_object) => {
                let bind_group = object.bind_group(handler, &render_object.bind_group_layout);
                render_object.bind_group = bind_group;
                true
            }
            _ => false,
        }
    }

    /// Renders the scene to `view`.
    pub fn render_by_rpass<'b, 'a: 'b>(&'a mut self, rpass: &mut RenderPass<'b>) {
        self.reset_bind_group();

        rpass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
        for (_, object) in &self.objects {
            if !object.visible {
                continue;
            }
            rpass.set_pipeline(&object.pipeline);
            rpass.set_bind_group(1, object.bind_group.as_ref(), &[]);
            rpass.set_vertex_buffer(0, object.vertex_buffer.buffer.slice(..));
            match object.index_buffer {
                Some(ref index_buffer) => {
                    rpass.set_index_buffer(index_buffer.buffer.slice(..), IndexFormat::Uint32);
                    let index_size = index_buffer.size as u32 / size_of::<u32>() as u32;
                    rpass.draw_indexed(0..index_size, 0, 0..1);
                }
                None => rpass.draw(
                    0..(object.vertex_buffer.size / object.vertex_buffer.stride) as u32,
                    0..1,
                ),
            }
        }
    }
}
