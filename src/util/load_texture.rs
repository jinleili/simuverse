use image::GenericImageView;
use std::{num::NonZeroU32, path::PathBuf};
use wgpu::{Extent3d, Sampler, Texture, TextureFormat, TextureView};

pub struct AnyTexture {
    pub size: Extent3d,
    pub tex: Texture,
    pub tex_view: TextureView,
    pub format: TextureFormat,
    pub view_dimension: wgpu::TextureViewDimension,
}
#[allow(dead_code)]
pub fn from_path(
    image_path: &str,
    app_view: &app_surface::AppSurface,
    usage: wgpu::TextureUsages,
    set_to_grayscale: bool,
) -> (AnyTexture, Sampler) {
    let path = if image_path.split("/").count() > 5 {
        // is already a full path
        PathBuf::from(image_path)
    } else {
        app_surface::fs::get_texture_file_path(image_path)
    };
    let (texels, texture_extent, format) = load_from_path(path, set_to_grayscale);
    let pixel_bytes = single_pixel_bytes(format);
    let texture = app_view.device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        label: None,
        view_formats: &[format.remove_srgb_suffix()],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
        format: Some(format.remove_srgb_suffix()),
        ..Default::default()
    });
    app_view.queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &texels,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(NonZeroU32::new(pixel_bytes * texture_extent.width).unwrap()),
            rows_per_image: Some(NonZeroU32::new(texture_extent.height).unwrap()),
        },
        texture_extent,
    );
    let any_tex = AnyTexture {
        size: texture_extent,
        tex: texture,
        tex_view: texture_view,
        view_dimension: wgpu::TextureViewDimension::D2,
        format,
    };

    (any_tex, default_sampler(&app_view.device))
}

// from webgpu spec: R8 | R16 is not supported for storage use.
#[allow(dead_code)]
pub fn into_format_r32float(
    image_path: &str,
    app_view: &app_surface::AppSurface,
    usage: wgpu::TextureUsages,
    label: Option<&'static str>,
) -> AnyTexture {
    let path = app_surface::fs::get_texture_file_path(image_path);

    let (texels, texture_extent) = load_by_luma(path);
    let pixel_bytes = 4;
    let new_texels: Vec<f32> = texels.into_iter().map(|t| t as f32).collect();
    let tex_format = TextureFormat::R32Float;
    let texture = app_view.device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: tex_format,
        usage,
        label,
        view_formats: &[],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    app_view.queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(&new_texels),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(NonZeroU32::new(pixel_bytes * texture_extent.width).unwrap()),
            rows_per_image: Some(NonZeroU32::new(texture_extent.height).unwrap()),
        },
        texture_extent,
    );
    let any_tex = AnyTexture {
        size: texture_extent,
        tex: texture,
        tex_view: texture_view,
        view_dimension: wgpu::TextureViewDimension::D2,
        format: tex_format,
    };
    any_tex
}

#[allow(dead_code)]
pub fn update_by_path(
    image_path: &str,
    app_view: &app_surface::AppSurface,
    texture: &Texture,
    set_to_grayscale: bool,
) {
    let path = app_surface::fs::get_texture_file_path(image_path);

    let (texels, texture_extent, format) = load_from_path(path, set_to_grayscale);
    let pixel_bytes = single_pixel_bytes(format);

    app_view.queue.write_texture(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &texels,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(NonZeroU32::new(pixel_bytes * texture_extent.width).unwrap()),
            rows_per_image: Some(NonZeroU32::new(texture_extent.height).unwrap()),
        },
        texture_extent,
    );
}

#[allow(dead_code)]
pub fn from_buffer(
    buffer: &wgpu::Buffer,
    app_view: &app_surface::AppSurface,
    encoder: &mut wgpu::CommandEncoder,
    width: u32,
    height: u32,
    pixel_size: u32,
    format: TextureFormat,
    usage: wgpu::TextureUsages,
) -> (AnyTexture, Sampler) {
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = app_view.device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        label: None,
        view_formats: &[],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // BufferCopyView 必须 >= TextureCopyView
    encoder.copy_buffer_to_texture(
        wgpu::ImageCopyBuffer {
            buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(pixel_size * width).unwrap()),
                rows_per_image: Some(NonZeroU32::new(height).unwrap()),
            },
        },
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        texture_extent,
    );
    let any_tex = AnyTexture {
        size: texture_extent,
        tex: texture,
        tex_view: texture_view,
        view_dimension: wgpu::TextureViewDimension::D2,
        format,
    };

    (any_tex, default_sampler(&app_view.device))
}

fn load_from_path(
    path: PathBuf,
    set_to_grayscale: bool,
) -> (Vec<u8>, wgpu::Extent3d, TextureFormat) {
    let img = image::open(&path.as_path()).unwrap();
    let (width, height) = img.dimensions();
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let (format, texels) = if set_to_grayscale {
        // webgpu spec: R8 | R16 is not supported for storage use.
        // (TextureFormat::R8Unorm, DynamicImage::ImageLuma16(img.into_luma8()).into_bytes())
        (TextureFormat::R8Unorm, img.into_luma8().into_raw())
    } else {
        match img.color() {
            image::ColorType::L8 => (TextureFormat::R8Unorm, img.into_bytes()),
            image::ColorType::Rgb8 | image::ColorType::Rgba8 => {
                (TextureFormat::Rgba8UnormSrgb, img.into_rgba8().into_raw())
            }
            _ => panic!("unsupported color type"),
        }
    };

    (texels, texture_extent, format)
}

fn load_by_luma(path: PathBuf) -> (Vec<u8>, wgpu::Extent3d) {
    let img = image::open(&path.as_path()).unwrap();
    let (width, height) = img.dimensions();
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    (img.to_luma8().into_raw(), texture_extent)
}

pub fn empty(
    device: &wgpu::Device,
    format: TextureFormat,
    extent: Extent3d,
    view_dimension: Option<wgpu::TextureViewDimension>,
    usage: Option<wgpu::TextureUsages>,
    label: Option<&'static str>,
) -> AnyTexture {
    let usage = if let Some(u) = usage {
        u
    } else {
        wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
    };
    let view_dimension = if let Some(vd) = view_dimension {
        vd
    } else {
        wgpu::TextureViewDimension::D2
    };
    let (tex_dimension, array_layer_count) = if view_dimension == wgpu::TextureViewDimension::D3 {
        (wgpu::TextureDimension::D3, 1)
    } else {
        (wgpu::TextureDimension::D2, extent.depth_or_array_layers)
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: tex_dimension,
        format,
        usage,
        label,
        view_formats: &[],
    });
    let mut view_label: String = String::from("view");
    if let Some(lb) = label {
        view_label = lb.to_string() + "_" + &view_label;
    }
    let tex_view_descriptor = wgpu::TextureViewDescriptor {
        label: Some(&view_label),
        format: Some(format),
        dimension: Some(view_dimension),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: std::num::NonZeroU32::new(array_layer_count),
    };
    let texture_view = texture.create_view(&tex_view_descriptor);
    // let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    AnyTexture {
        size: extent,
        tex: texture,
        tex_view: texture_view,
        view_dimension,
        format,
    }
}

// 32位浮点纹理
#[allow(dead_code)]
pub fn empty_f32_view(device: &wgpu::Device, width: u32, height: u32) -> AnyTexture {
    crate::util::load_texture::empty(
        device,
        TextureFormat::Rgba32Float,
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        None,
        None,
        None,
    )
}

#[allow(dead_code)]
pub fn default_sampler(device: &wgpu::Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

#[allow(dead_code)]
pub fn repeate_sampler(device: &wgpu::Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

#[allow(dead_code)]
pub fn mirror_repeate_sampler(device: &wgpu::Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::MirrorRepeat,
        address_mode_v: wgpu::AddressMode::MirrorRepeat,
        address_mode_w: wgpu::AddressMode::MirrorRepeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

// 瓦片式平铺采样
#[allow(dead_code)]
pub fn tile_sampler(device: &wgpu::Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

// 双线性插值
// https://vulkan-tutorial.com/Texture_mapping/Image_view_and_sampler
#[allow(dead_code)]
pub fn bilinear_sampler(device: &wgpu::Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        // iOS 上设置了 compare 值会 crash
        // compare: Some(wgpu::CompareFunction::LessEqual),
        // compare: wgpu::CompareFunction::Undefined,
        ..Default::default()
    })
}

fn single_pixel_bytes(format: TextureFormat) -> u32 {
    match format {
        TextureFormat::R8Sint
        | TextureFormat::R8Snorm
        | TextureFormat::R8Uint
        | TextureFormat::R8Unorm => 1,
        TextureFormat::R16Float
        | TextureFormat::R16Sint
        | TextureFormat::R16Snorm
        | TextureFormat::R16Uint
        | TextureFormat::R16Unorm
        | TextureFormat::Rg8Sint
        | TextureFormat::Rg8Snorm
        | TextureFormat::Rg8Uint
        | TextureFormat::Rg8Unorm => 2,
        TextureFormat::Rgba8Sint
        | TextureFormat::Rgba8Uint
        | TextureFormat::Bgra8Unorm
        | TextureFormat::Bgra8UnormSrgb
        | TextureFormat::Rgba8Snorm
        | TextureFormat::Rgba8Unorm
        | TextureFormat::Rgba8UnormSrgb => 4,
        TextureFormat::Rgba16Float
        | TextureFormat::Rgba16Sint
        | TextureFormat::Rgba16Snorm
        | TextureFormat::Rgba16Uint
        | TextureFormat::Rgba16Unorm => 8,
        TextureFormat::Rgba32Float | TextureFormat::Rgba32Sint | TextureFormat::Rgba32Uint => 16,
        _ => 0,
    }
}
