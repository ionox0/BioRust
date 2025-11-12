use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
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
                value += noise.get([
                    nx * frequency,
                    ny * frequency,
                    nz * frequency,
                    nw * frequency,
                ]) * amplitude;
                amplitude *= 0.5;
                frequency *= 2.0;
            }

            // Normalize to 0-1
            value = (value + 1.0) * 0.5;

            // Create VERY BROWN terrain texture - no green at all
            let (r, g, b) = if value < 0.3 {
                // Very dark brown/soil base
                let intensity = (value as f32 * 0.4 + 0.10).min(1.0);
                (intensity * 0.30, intensity * 0.24, intensity * 0.18) // Brown gradient
            } else if value < 0.6 {
                // Medium brown earth - more red-brown
                let earth_mix = ((value - 0.3) / 0.3) as f32;
                (
                    0.25 + earth_mix * 0.10,
                    0.20 + earth_mix * 0.08,
                    0.15 + earth_mix * 0.06,
                )
            } else if value < 0.8 {
                // Light brown dirt - sandy brown
                let light_brown = ((value - 0.6) / 0.2) as f32;
                (
                    0.35 + light_brown * 0.10,
                    0.28 + light_brown * 0.08,
                    0.21 + light_brown * 0.06,
                )
            } else {
                // Very light brown/tan areas
                let tan_intensity = ((value - 0.8) / 0.2) as f32;
                (
                    0.45 + tan_intensity * 0.08,
                    0.36 + tan_intensity * 0.06,
                    0.27 + tan_intensity * 0.04,
                )
            };

            // Add subtle variation and strong fading for very brown appearance
            let detail = ((value * 16.0).fract() - 0.5) as f32 * 0.03; // Even less variation
            let fade_factor = 0.6; // Much stronger fading for very muted brown appearance

            texture_data.push(((r + detail) * fade_factor * 255.0).clamp(0.0, 255.0) as u8);
            texture_data.push(((g + detail) * fade_factor * 255.0).clamp(0.0, 255.0) as u8);
            texture_data.push(((b + detail) * fade_factor * 255.0).clamp(0.0, 255.0) as u8);
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
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..Default::default()
    });

    images.add(image)
}
