use crate::color::Color;
use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use nalgebra_glm::{Vec3};
use image::RgbaImage;

pub struct Cube {
    pub min_corner: Vec3, 
    pub max_corner: Vec3, 
    pub material: Material,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let mut tmin = (self.min_corner.x - ray_origin.x) / ray_direction.x;
        let mut tmax = (self.max_corner.x - ray_origin.x) / ray_direction.x;

        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (self.min_corner.y - ray_origin.y) / ray_direction.y;
        let mut tymax = (self.max_corner.y - ray_origin.y) / ray_direction.y;

        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if (tmin > tymax) || (tymin > tmax) {
            return Intersect::empty();
        }

        if tymin > tmin {
            tmin = tymin;
        }

        if tymax < tmax {
            tmax = tymax;
        }

        let mut tzmin = (self.min_corner.z - ray_origin.z) / ray_direction.z;
        let mut tzmax = (self.max_corner.z - ray_origin.z) / ray_direction.z;

        if tzmin > tzmax {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }
        if (tmin > tzmax) || (tzmin > tmax) {
            return Intersect::empty();
        }

        if tzmin > tmin {
            tmin = tzmin;
        }

        if tzmax < tmax {
            tmax = tzmax;
        }

        if tmin < 0.0 {
            return Intersect::empty();
        }

        let intersection_point = ray_origin + ray_direction * tmin;
        let normal = self.calculate_normal(&intersection_point);
        let (u, v) = self.get_texture_coordinates(&intersection_point);
        let distance = tmin;

        // Obtener el color de la textura si está disponible
        let texture_color = if let Some(texture) = &self.material.texture {
            // Clampear u y v para evitar desbordes de la textura
            let u_clamped = u.clamp(0.0, 1.0 - f32::EPSILON);
            let v_clamped = v.clamp(0.0, 1.0 - f32::EPSILON);

            let tex_x = (u_clamped * texture.width() as f32) as u32;
            let tex_y = (v_clamped * texture.height() as f32) as u32;

            let pixel = texture.get_pixel(tex_x, tex_y);
            Color::new(pixel[0], pixel[1], pixel[2])
        } else {
            self.material.diffuse
        };

        Intersect::new(
            intersection_point,
            normal,
            distance,
            Material::new(
                texture_color,
                self.material.specular,
                self.material.albedo,
                self.material.refractive_index,
                self.material.texture.clone(),
            ),
        )
    }
}

impl Cube {
    fn get_texture_coordinates(&self, point: &Vec3) -> (f32, f32) {
        let epsilon = 1e-4;

        if (point.x - self.min_corner.x).abs() < epsilon { // Cara izquierda
            let u = (point.z - self.min_corner.z) / (self.max_corner.z - self.min_corner.z);
            let v = (point.y - self.min_corner.y) / (self.max_corner.y - self.min_corner.y);
            (u, v)
        } else if (point.x - self.max_corner.x).abs() < epsilon { // Cara derecha
            let u = (point.z - self.min_corner.z) / (self.max_corner.z - self.min_corner.z);
            let v = (self.max_corner.y - point.y) / (self.max_corner.y - self.min_corner.y);
            (u, v)
        } else if (point.y - self.min_corner.y).abs() < epsilon { // Cara inferior
            let u = (point.x - self.min_corner.x) / (self.max_corner.x - self.min_corner.x);
            let v = (point.z - self.min_corner.z) / (self.max_corner.z - self.min_corner.z);
            (u, v)
        } else if (point.y - self.max_corner.y).abs() < epsilon { // Cara superior
            let u = (point.x - self.min_corner.x) / (self.max_corner.x - self.min_corner.x);
            let v = (self.max_corner.z - point.z) / (self.max_corner.z - self.min_corner.z);
            (u, v)
        } else if (point.z - self.min_corner.z).abs() < epsilon { // Cara trasera
            let u = (point.x - self.min_corner.x) / (self.max_corner.x - self.min_corner.x);
            let v = (point.y - self.min_corner.y) / (self.max_corner.y - self.min_corner.y);
            (u, v)
        } else { // Cara frontal
            let u = (point.x - self.min_corner.x) / (self.max_corner.x - self.min_corner.x);
            let v = (self.max_corner.y - point.y) / (self.max_corner.y - self.min_corner.y);
            (u, v)
        }
    }

    fn calculate_normal(&self, point: &Vec3) -> Vec3 {
        let epsilon = 1e-4;

        if (point.x - self.min_corner.x).abs() < epsilon {
            Vec3::new(-1.0, 0.0, 0.0)
        } else if (point.x - self.max_corner.x).abs() < epsilon {
            Vec3::new(1.0, 0.0, 0.0)
        } else if (point.y - self.min_corner.y).abs() < epsilon {
            Vec3::new(0.0, -1.0, 0.0)
        } else if (point.y - self.max_corner.y).abs() < epsilon {
            Vec3::new(0.0, 1.0, 0.0)
        } else if (point.z - self.min_corner.z).abs() < epsilon {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }
}
