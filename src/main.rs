mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod material;
mod ray_intersect;
mod skybox; 

use image::open;
use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{normalize, Vec3};
use std::f32::consts::PI;
use std::time::{Duration, Instant};

use crate::camera::Camera;
use crate::color::Color;
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::skybox::Skybox; 

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


fn fresnel(incident: &Vec3, normal: &Vec3, ior: f32) -> f32 {
    let mut cosi = incident.dot(normal).clamp(-1.0, 1.0);
    let etai = 1.0;
    let etat = ior;
    let sint = etai / etat * (1.0 - cosi * cosi).sqrt();

    if sint >= 1.0 {
        return 1.0;
    } else {
        let cost = (1.0 - sint * sint).sqrt();
        cosi = cosi.abs();
        let rs = ((etat * cosi) - (etai * cost)) / ((etat * cosi) + (etai * cost));
        let rp = ((etai * cosi) - (etat * cost)) / ((etai * cosi) + (etat * cost));
        return (rs * rs + rp * rp) / 2.0;
    }
}

fn cast_shadow(
    intersect: &Intersect,
    lights: &[Light],
    objects: &[Cube],
    light_index: usize,
) -> f32 {
    let light = &lights[light_index];
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
    lights: &[Light],
    depth: u32,
    skybox: &Skybox,
) -> Color {
    if depth > 3 {
        return skybox.get_color_from_direction(ray_direction);
    }

    let mut closest_intersect = Intersect::empty();
    let mut min_distance = f32::INFINITY;

    for object in objects {
        let intersect = object.ray_intersect(ray_origin, ray_direction);
        if intersect.is_intersecting && intersect.distance < min_distance {
            min_distance = intersect.distance;
            closest_intersect = intersect;
        }
    }

    if !closest_intersect.is_intersecting {
        return skybox.get_color_from_direction(ray_direction);
    }

    let intersect = closest_intersect;

    let mut color = intersect.material.emission;

    let mut diffuse = Color::black();
    let mut specular = Color::black();

    for (i, light) in lights.iter().enumerate() {
        let light_dir = (light.position - intersect.point).normalize();
        let view_dir = (ray_origin - intersect.point).normalize();
        let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

        let shadow_intensity = cast_shadow(&intersect, lights, objects, i);
        let light_intensity = light.intensity * (1.0 - shadow_intensity);

        let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0);
        diffuse = diffuse
            + (intersect.material.diffuse * light.color) * diffuse_intensity * light_intensity;

        let specular_intensity = view_dir
            .dot(&reflect_dir)
            .max(0.0)
            .powf(intersect.material.specular);
        specular = specular + light.color * specular_intensity * light_intensity;
    }

    let kr = fresnel(
        ray_direction,
        &intersect.normal,
        intersect.material.refractive_index,
    );
    let reflectivity = kr * intersect.material.albedo[2];
    let transparency = (1.0 - kr) * intersect.material.albedo[3];

    let mut reflect_color = Color::black();
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = cast_ray(
            &reflect_origin,
            &reflect_dir,
            objects,
            lights,
            depth + 1,
            skybox,
        );
    }

    let mut refract_color = Color::black();
    if transparency > 0.0 {
        let refract_dir = refract(
            &ray_direction,
            &intersect.normal,
            intersect.material.refractive_index,
        )
        .normalize();
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(
            &refract_origin,
            &refract_dir,
            objects,
            lights,
            depth + 1,
            skybox,
        );
    }

    color = color
        + (diffuse * intersect.material.albedo[0] + specular * intersect.material.albedo[1])
            * (1.0 - reflectivity - transparency)
        + (reflect_color * reflectivity)
        + (refract_color * transparency);

    color.clamp()
}

pub fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Cube],
    camera: &Camera,
    lights: &[Light],
    skybox: &Skybox,
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
                lights,
                0,
                skybox,
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
  let mut last_frame = Instant::now();
  let mut time_of_day = 0.0;
  let day_duration = 20.0;

  let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

  let mut window = Window::new(
      "Minecraft",
      window_width,
      window_height,
      WindowOptions::default(),
  )
  .unwrap();

  let stone_texture = open("./src/textures/old-cobblestone-texture.png")
      .unwrap()
      .to_rgba8();

  let grass_texture = open("./src/textures/grass.png").unwrap().to_rgba8();
  let wood_texture = open("./src/textures/wood.png").unwrap().to_rgba8();
  let glowstone_texture = open("./src/textures/glowstone.png").unwrap().to_rgba8();

  let skybox = Skybox::new(
      open("./src/textures/sky.jpg").unwrap().to_rgba8(),
      open("./src/textures/sky.jpg").unwrap().to_rgba8(),
      open("./src/textures/sky.jpg").unwrap().to_rgba8(),
      open("./src/textures/sky.jpg").unwrap().to_rgba8(),
      open("./src/textures/sky.jpg").unwrap().to_rgba8(),
      open("./src/textures/sky.jpg").unwrap().to_rgba8(),
  );

  let stone = Material::new(
    Color::from_u8(90, 90, 90),
    10.0,
    [0.6, 0.1, 0.1, 0.0], 
    1.0,
    Some(stone_texture),
    None,
    Color::black(),
);

// Material de Césped
let grass = Material::new(
    Color::from_u8(100, 200, 100),
    10.0,
    [0.6, 0.1, 0.1, 0.0], 
    1.0,
    Some(grass_texture),
    None,
    Color::black(),
);


  let water_textures = vec![
      open("./src/textures/water1.png").unwrap().to_rgba8(),
      open("./src/textures/water2.png").unwrap().to_rgba8(),
  ];

  let water = Material::new(
    Color::from_u8(50, 50, 200),
    50.0,
    [0.1, 0.7, 0.4, 0.7], 
    1.33,
    Some(water_textures[0].clone()),
    None,
    Color::black(),
);

let wood = Material::new(
  Color::from_u8(139, 69, 19),
  5.0,
  [0.6, 0.3, 0.1, 0.0], 
  1.0,
  Some(wood_texture),
  None,
  Color::black(),
);

let glowstone = Material::new(
  Color::from_u8(255, 223, 128),
  10.0,
  [0.7, 0.3, 0.0, 0.0], 
  1.0,
  Some(glowstone_texture),
  None,
  Color::from_u8(255, 223, 128),
);



  let mut objects = Vec::new();


  let water_positions = [(1, 2), (2, 2), (3, 2)];

  for x in 0..5 {
      for z in 0..5 {
          if water_positions.contains(&(x, z)) {
              objects.push(Cube {
                  min_corner: Vec3::new(x as f32, -1.0, z as f32),
                  max_corner: Vec3::new(x as f32 + 1.0, 0.0, z as f32 + 1.0),
                  material: water.clone(),
              });
          } else {
              // Añadir bloque de césped
              objects.push(Cube {
                  min_corner: Vec3::new(x as f32, -1.0, z as f32),
                  max_corner: Vec3::new(x as f32 + 1.0, 0.0, z as f32 + 1.0),
                  material: grass.clone(),
              });
          }
      }
  }

  for y in 0..=3 {
      objects.push(Cube {
          min_corner: Vec3::new(0.0, y as f32, 0.0),
          max_corner: Vec3::new(1.0, y as f32 + 1.0, 1.0),
          material: wood.clone(),
      });
  }

  objects.push(Cube {
      min_corner: Vec3::new(0.0, 0.0, 4.0),
      max_corner: Vec3::new(1.0, 1.0, 5.0),
      material: glowstone.clone(),
  });
  objects.push(Cube {
      min_corner: Vec3::new(4.0, 0.0, 0.0),
      max_corner: Vec3::new(5.0, 1.0, 1.0),
      material: glowstone.clone(),
  });


  for x in 1..=3 {
      for y in 0..=2 {
          if !(x == 2 && y == 1) {
              objects.push(Cube {
                  min_corner: Vec3::new(x as f32, y as f32, 4.0),
                  max_corner: Vec3::new(x as f32 + 1.0, y as f32 + 1.0, 5.0),
                  material: stone.clone(),
              });
          }
      }
  }

  let mut camera = Camera::new(
      Vec3::new(2.5, 2.0, 10.0), 
      Vec3::new(2.5, 0.0, 2.5),
      Vec3::new(0.0, 1.0, 0.0),
  );

  let mut lights = vec![Light::new(
      Vec3::new(0.0, 10.0, 5.0),
      Color::from_u8(255, 255, 255),
      1.0,
  )];

  let rotation_speed = PI / 16.0;

  while window.is_open() && !window.is_key_down(Key::Escape) {
      let current_frame = Instant::now();
      let delta_time = current_frame.duration_since(last_frame).as_secs_f32();
      last_frame = current_frame;

      time_of_day += delta_time;
      if time_of_day > day_duration {
          time_of_day -= day_duration;
      }

      let day_progress = time_of_day / day_duration;
      let sun_angle = day_progress * 2.0 * PI;

      let sun_position = Vec3::new(10.0 * sun_angle.cos(), 10.0 * sun_angle.sin(), 0.0);
      lights[0].position = sun_position;

      let (intensity, color) = if day_progress < 0.25 {
          let factor = day_progress / 0.25;
          (
              0.5 + 0.5 * factor,
              Color::from_u8(255, 183, 76) * factor
                  + Color::from_u8(50, 50, 100) * (1.0 - factor),
          )
      } else if day_progress < 0.5 {
          (1.0, Color::from_u8(255, 255, 255))
      } else if day_progress < 0.75 {
          let factor = (day_progress - 0.5) / 0.25;
          (
              1.0 - 0.5 * factor,
              Color::from_u8(255, 183, 76) * (1.0 - factor)
                  + Color::from_u8(50, 50, 100) * factor,
          )
      } else {

          (0.5, Color::from_u8(50, 50, 100))
      };
      lights[0].intensity = intensity;
      lights[0].color = color;

      window.set_title(&format!("Minecraft - FPS: {:.2}", 1.0 / delta_time));

      if let Some(scroll) = window.get_scroll_wheel() {
          if scroll.1 > 0.0 {
              camera.move_towards_target(0.2 * scroll.1);
          } else if scroll.1 < 0.0 {
              camera.move_away_from_target(-0.2 * scroll.1);
          }
      }

      if window.is_key_down(Key::A) {
          camera.rotate_around_target(rotation_speed, 0.0);
      }

      if window.is_key_down(Key::D) {
          camera.rotate_around_target(-rotation_speed, 0.0);
      }

      if window.is_key_down(Key::W) {
          camera.rotate_around_target(0.0, -rotation_speed);
      }

      if window.is_key_down(Key::S) {
          camera.rotate_around_target(0.0, rotation_speed);
      }


      render(&mut framebuffer, &objects, &camera, &lights, &skybox);

      window
          .update_with_buffer(
              &framebuffer
                  .buffer
                  .iter()
                  .map(|c| c.to_u32())
                  .collect::<Vec<u32>>(),
              framebuffer_width,
              framebuffer_height,
          )
          .unwrap();

      std::thread::sleep(frame_delay);
  }
}
