// skybox.rs

use crate::color::Color;
use nalgebra_glm::Vec3;
use image::RgbaImage;

pub struct Skybox {
    pub right: RgbaImage,
    pub left: RgbaImage,
    pub top: RgbaImage,
    pub bottom: RgbaImage,
    pub front: RgbaImage,
    pub back: RgbaImage,
}

impl Skybox {
    pub fn new(
        right: RgbaImage,
        left: RgbaImage,
        top: RgbaImage,
        bottom: RgbaImage,
        front: RgbaImage,
        back: RgbaImage,
    ) -> Self {
        Skybox {
            right,
            left,
            top,
            bottom,
            front,
            back,
        }
    }

    pub fn get_color_from_direction(&self, direction: &Vec3) -> Color {
        // Normalizar la dirección del rayo
        let dir = direction.normalize();

        // Obtener los valores absolutos de las componentes
        let abs_x = dir.x.abs();
        let abs_y = dir.y.abs();
        let abs_z = dir.z.abs();

        let is_x_positive = dir.x > 0.0;
        let is_y_positive = dir.y > 0.0;
        let is_z_positive = dir.z > 0.0;

        let max_axis;
        let uc;
        let vc;
        let face_texture;

        // Determinar qué cara del cubo se está mirando
        if is_x_positive && abs_x >= abs_y && abs_x >= abs_z {
            // Cara derecha (Positive X)
            max_axis = abs_x;
            uc = -dir.z;
            vc = dir.y;
            face_texture = &self.right;
        } else if !is_x_positive && abs_x >= abs_y && abs_x >= abs_z {
            // Cara izquierda (Negative X)
            max_axis = abs_x;
            uc = dir.z;
            vc = dir.y;
            face_texture = &self.left;
        } else if is_y_positive && abs_y >= abs_x && abs_y >= abs_z {
            // Cara superior (Positive Y)
            max_axis = abs_y;
            uc = dir.x;
            vc = -dir.z;
            face_texture = &self.top;
        } else if !is_y_positive && abs_y >= abs_x && abs_y >= abs_z {
            // Cara inferior (Negative Y)
            max_axis = abs_y;
            uc = dir.x;
            vc = dir.z;
            face_texture = &self.bottom;
        } else if is_z_positive && abs_z >= abs_x && abs_z >= abs_y {
            // Cara frontal (Positive Z)
            max_axis = abs_z;
            uc = dir.x;
            vc = dir.y;
            face_texture = &self.front;
        } else {
            // Cara trasera (Negative Z)
            max_axis = abs_z;
            uc = -dir.x;
            vc = dir.y;
            face_texture = &self.back;
        }

        // Convertir coordenadas UC y VC a UV en el rango [0, 1]
        let u = 0.5 * (uc / max_axis + 1.0);
        let v = 0.5 * (vc / max_axis + 1.0);

        // Mapear UV a coordenadas de textura
        let tex_x = (u * (face_texture.width() - 1) as f32) as u32;
        let tex_y = ((1.0 - v) * (face_texture.height() - 1) as f32) as u32; // Invertir V

        // Obtener el pixel de la textura
        let pixel = face_texture.get_pixel(tex_x.min(face_texture.width() - 1), tex_y.min(face_texture.height() - 1));

        Color::new(pixel[0] as f32 / 255.0, pixel[1] as f32 / 255.0, pixel[2] as f32 / 255.0)
    }
}
