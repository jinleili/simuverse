use super::*;

impl PolygonShaders {
    /// Creates default polygon shaders.
    #[inline(always)]
    pub fn default(device: &Device) -> Self {
        let source = include_str!("shaders/microfacet-module.wgsl").to_string()
            // + include_str!("../../../assets/wgsl/noise/fn_perlin_noise.wgsl")
            + include_str!("shaders/polygon.wgsl");
        let shader_module = Arc::new(device.create_shader_module(ShaderModuleDescriptor {
            source: ShaderSource::Wgsl(source.into()),
            label: None,
        }));
        Self{
            vertex_module:Arc::clone(&shader_module),
            vertex_entry:"vs_main",
            fragment_module:Arc::clone(&shader_module),
            fragment_entry:"nontex_main",
            tex_fragment_module: Arc::clone(&shader_module),
            tex_fragment_entry:"tex_main",
            procedural_fragment_module: Arc::clone(&shader_module),
            procedural_fragment_entry:"procedural_main",
        }
    }
}

impl WireShaders {
    /// Constructor
    /// # Parameters
    /// - `vertex_module`: vertex shader module
    /// - `vertex_entry`: entry point of vertex shader module
    /// - `fragment_module`: fragment shader module without texture
    /// - `fragment_entry`: entry point of fragment shader module without texture
    #[inline(always)]
    pub const fn new(
        vertex_module: Arc<ShaderModule>,
        vertex_entry: &'static str,
        fragment_module: Arc<ShaderModule>,
        fragment_entry: &'static str,
    ) -> Self {
        Self {
            vertex_module,
            vertex_entry,
            fragment_module,
            fragment_entry,
        }
    }

    /// Creates default wireframe shaders
    #[inline(always)]
    fn default(device: &Device) -> Self {
        let shader_module = Arc::new(device.create_shader_module(ShaderModuleDescriptor {
            source: ShaderSource::Wgsl(include_str!("shaders/line.wgsl").into()),
            label: None,
        }));
        Self::new(
            Arc::clone(&shader_module),
            "vs_main",
            shader_module,
            "fs_main",
        )
    }
}

impl CreatorCreator for DeviceHandler {
    #[inline(always)]
    fn instance_creator(&self) -> InstanceCreator {
        InstanceCreator {
            handler: self.clone(),
            polygon_shaders: PolygonShaders::default(&self.device),
            wire_shaders: WireShaders::default(&self.device),
        }
    }
}

impl CreatorCreator for Scene {
    #[inline(always)]
    fn instance_creator(&self) -> InstanceCreator {
        self.device_handler().instance_creator()
    }
}

impl InstanceCreator {
    /// Creates Instance from object.
    #[inline(always)]
    pub fn create_instance<I, T>(&self, object: &T, state: &T::State) -> I
    where
        T: ToInstance<I>,
        I: Instance,
    {
        object.to_instance(&self.handler, &I::standard_shaders(self), state)
    }
}
