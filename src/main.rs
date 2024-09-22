mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod material;
mod ray_intersect;

use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{normalize, Vec3};
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use image::open;

use crate::camera::Camera;
use crate::color::Color;
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};

const ORIGIN_BIAS: f32 = 1e-4;

fn offset_origin(intersect: &Intersect, direction: &Vec3) -> Vec3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if direction.dot(&intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
    let cosi = -incident.dot(normal).max(-1.0).min(1.0);
    let (n_cosi, eta, n_normal);

    if cosi < 0.0 {
        n_cosi = -cosi;
        eta = 1.0 / eta_t;
        n_normal = -normal;
    } else {
        n_cosi = cosi;
        eta = eta_t;
        n_normal = *normal;
    }

    let k = 1.0 - eta * eta * (1.0 - n_cosi * n_cosi);

    if k < 0.0 {
        reflect(incident, &n_normal)
    } else {
        eta * incident + (eta * n_cosi - k.sqrt()) * n_normal
    }
}

fn cast_shadow(intersect: &Intersect, light: &Light, objects: &[Cube]) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let shadow_ray_origin = offset_origin(intersect, &light_dir);
    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
            break;
        }
    }

    shadow_intensity
}


pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Cube],
    light: &Light,
    depth: u32,
    skybox_color: Color,
) -> Color {
    if depth > 3 {
        return skybox_color;
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return skybox_color;
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse
        * intersect.material.albedo[0]
        * diffuse_intensity
        * light_intensity;

    let specular_intensity = view_dir
        .dot(&reflect_dir)
        .max(0.0)
        .powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

    let mut reflect_color = Color::black();
    let reflectivity = intersect.material.albedo[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1, skybox_color);
    }

    let mut refract_color = Color::black();
    let transparency = intersect.material.albedo[3];
    if transparency > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, light, depth + 1, skybox_color);
    }

    (diffuse + specular) * (1.0 - reflectivity - transparency)
        + (reflect_color * reflectivity)
        + (refract_color * transparency)
}

pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Cube],
    camera: &Camera,
    light: &Light,
    skybox_color: Color,
) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let rotated_direction = camera.transform_vector(&ray_direction);

            let pixel_color = cast_ray(
                &camera.position,
                &rotated_direction,
                objects,
                light,
                0,
                skybox_color,
            );

            framebuffer.set_current_color(pixel_color);
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 600;
    let framebuffer_height = 400;
    let frame_delay = Duration::from_millis(16);
    let mut skybox_color = Color::new(135, 206, 235);
    let mut window_title = "Minecraft".to_string();
    let mut last_frame = Instant::now();

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        &window_title,
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    let deep_cobblestone_texture = open("./src/textures/old-cobblestone-texture.png")
        .unwrap()
        .to_rgba8();

        

    let deep_cobble = Material::new(
        Color::new(90, 90, 90),
        10.0,
        [0.3, 0.5, 0.3, 0.0],
        0.0,
        Some(deep_cobblestone_texture),
    );

    let cube = Cube {
        min_corner: Vec3::new(-1.0, -1.0, -1.0),
        max_corner: Vec3::new(1.0, 1.0, 1.0),
        material: deep_cobble,
    };

    let objects = [cube];

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 15.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut light = Light::new(Vec3::new(0.0, 10.0, 5.0), Color::new(135, 206, 235), 2.0);

    let rotation_speed = PI / 8.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let current_frame = Instant::now();
        let delta_time = current_frame.duration_since(last_frame);
        let fps = 1.0 / delta_time.as_secs_f32();
        last_frame = current_frame;

        window.set_title(&format!("{} - FPS: {:.2}", window_title, fps));

        if window.is_key_down(Key::W) {
            camera.move_towards_target(0.2);
        }

        if window.is_key_down(Key::S) {
            camera.move_away_from_target(0.2);
        }

        if window.is_key_down(Key::Left) {
            camera.rotate_around_target(rotation_speed, 0.0);
        }

        if window.is_key_down(Key::Right) {
            camera.rotate_around_target(-rotation_speed, 0.0);
        }

        if window.is_key_down(Key::Up) {
            camera.rotate_around_target(0.0, -rotation_speed);
        }

        if window.is_key_down(Key::Down) {
            camera.rotate_around_target(0.0, rotation_speed);
        }

        if window.is_key_down(Key::Key1) {
            skybox_color = Color::new(135, 206, 235);
            light.color = Color::new(135, 206, 235);
            light.position = Vec3::new(0.0, 10.0, 5.0);
            window_title = "Minecraft (Day)".to_string();
        }

        if window.is_key_down(Key::Key2) {
            skybox_color = Color::new(251, 144, 98);
            light.color = Color::new(251, 144, 98);
            light.position = Vec3::new(10.0, 2.0, 2.0);
            window_title = "Minecraft (Sunset)".to_string();
        }

        if window.is_key_down(Key::Key3) {
            skybox_color = Color::new(19, 24, 98);
            light.color = Color::new(19, 24, 98);
            light.position = Vec3::new(-5.0, 5.0, 10.0);
            window_title = "Minecraft (Night)".to_string();
        }

        render(&mut framebuffer, &objects, &camera, &light, skybox_color);

        window
    .update_with_buffer(
        &framebuffer.buffer.iter().map(|c| c.to_u32()).collect::<Vec<u32>>(),
        framebuffer_width,
        framebuffer_height,
    )
    .unwrap();

        std::thread::sleep(frame_delay);
    }
}
