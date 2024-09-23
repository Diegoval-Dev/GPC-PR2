use crate::color::Color;
use image::RgbaImage;

#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 4],
    pub refractive_index: f32,
    pub texture: Option<RgbaImage>,
    pub normal_map: Option<RgbaImage>, 
    pub emission: Color,               
}

impl Material {
    pub fn new(
        diffuse: Color,
        specular: f32,
        albedo: [f32; 4],
        refractive_index: f32,
        texture: Option<RgbaImage>,
        normal_map: Option<RgbaImage>, 
        emission: Color,               
    ) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            refractive_index,
            texture,
            normal_map,
            emission,
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Color::black(),
            specular: 0.0,
            albedo: [0.0, 0.0, 0.0, 0.0],
            refractive_index: 1.0,
            texture: None,
            normal_map: None,
            emission: Color::black(),
        }
    }
}
