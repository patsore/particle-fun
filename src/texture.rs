use egui_wgpu::wgpu::{AddressMode, CompareFunction, Device, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, Sampler, SamplerDescriptor, SurfaceConfiguration, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};
use anyhow::Result;
use image::DynamicImage;

pub(crate) struct Texture {
    pub texture: egui_wgpu::wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn create_depth_texture(device: &Device, config: &SurfaceConfiguration, label: &str) -> Self {
        let size = Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT // 3.
                | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &SamplerDescriptor { // 4.
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Nearest,
                compare: Some(CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }


    pub fn from_bytes(
        device: &Device,
        queue: &Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }


    pub fn from_image(device: &Device, queue: &Queue, image: &DynamicImage, label: Option<&str>) -> Result<Self> {
        let image = image.to_rgba8();
        let (w, h) = image.dimensions();

        let size =  Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(
            &TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });

        queue.write_texture(
            ImageCopyTexture {
                aspect: TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &image,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * w),
                rows_per_image: Some(h),
            },
            size,
        );


        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}

