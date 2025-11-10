use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AddressMode, FilterMode};
use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use noise::{NoiseFn, Perlin};

// Create a seamless terrain texture inspired by Bug_Game
pub fn create_seamless_terrain_texture(images: &mut ResMut<Assets<Image>>) -> Handle<Image> {
    let texture_size = 256u32; // Smaller for better performance
    let mut texture_data = Vec::with_capacity((texture_size * texture_size * 4) as usize);
    
    // Create seamless noise pattern like Bug_Game
    let noise = Perlin::new(42);
    
    for y in 0..texture_size {
        for x in 0..texture_size {
            // Seamless coordinates using sine/cosine wrapping
            let s = x as f64 / texture_size as f64 * 2.0 * std::f64::consts::PI;
            let t = y as f64 / texture_size as f64 * 2.0 * std::f64::consts::PI;
            
            // Create seamless noise using 4D sampling in a torus
            let nx = s.cos() * 0.5;
            let ny = s.sin() * 0.5;
            let nz = t.cos() * 0.5;
            let nw = t.sin() * 0.5;
            
            // Multi-octave seamless noise
            let mut value = 0.0;
            let mut amplitude = 1.0;
            let mut frequency = 4.0;
            
            for _ in 0..3 {
                value += noise.get([nx * frequency, ny * frequency, nz * frequency, nw * frequency]) * amplitude;
                amplitude *= 0.5;
                frequency *= 2.0;
            }
            
            // Normalize to 0-1
            value = (value + 1.0) * 0.5;
            
            // Create natural grass-like terrain texture
            let (r, g, b) = if value < 0.3 {
                // Dark grass/soil base
                let intensity = (value as f32 * 0.7 + 0.2).min(1.0);
                (intensity * 0.3, intensity * 0.5, intensity * 0.2)
            } else if value < 0.6 {
                // Medium grass
                let grass_mix = ((value - 0.3) / 0.3) as f32;
                (0.25 + grass_mix * 0.15, 0.55 + grass_mix * 0.25, 0.2 + grass_mix * 0.15)
            } else if value < 0.8 {
                // Light grass
                let light_grass = ((value - 0.6) / 0.2) as f32;
                (0.35 + light_grass * 0.15, 0.65 + light_grass * 0.2, 0.3 + light_grass * 0.1)
            } else {
                // Very light grass/dried areas
                let dried_intensity = ((value - 0.8) / 0.2) as f32;
                (0.5 + dried_intensity * 0.2, 0.65 + dried_intensity * 0.15, 0.35 + dried_intensity * 0.1)
            };
            
            // Add subtle variation
            let detail = ((value * 16.0).fract() - 0.5) as f32 * 0.1;
            
            texture_data.push(((r + detail) * 255.0).clamp(0.0, 255.0) as u8);
            texture_data.push(((g + detail) * 255.0).clamp(0.0, 255.0) as u8);
            texture_data.push(((b + detail) * 255.0).clamp(0.0, 180.0) as u8);
            texture_data.push(255); // Alpha
        }
    }
    
    // Create the image with proper wrapping for seamless tiling
    let mut image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        texture_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    // Configure sampler for repeating/tiling texture
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        ..Default::default()
    });

    images.add(image)
}