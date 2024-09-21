use image::RgbaImage; 
use crate::color::Color;
use image::open;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 4],
    pub refractive_index: f32,
    pub texture: Option<RgbaImage>, 
}

impl Material {
    pub fn new(
        diffuse: Color,
        specular: f32,
        albedo: [f32; 4],
        refractive_index: f32,
        texture: Option<RgbaImage>,
    ) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            refractive_index,
            texture,
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Color::new(0, 0, 0),
            specular: 0.0,
            albedo: [0.0, 0.0, 0.0, 0.0],
            refractive_index: 0.0,
            texture: None, 
        }
    }

    pub fn with_texture(texture_path: &str) -> Self {
        let texture = open(texture_path).unwrap().to_rgba8();
        Material {
            diffuse: Color::new(255, 255, 255), 
            specular: 0.5,
            albedo: [0.8, 0.8, 0.8, 0.0],
            refractive_index: 1.0,
            texture: Some(texture),
        }
    }
}
