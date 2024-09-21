use nalgebra_glm::Vec3;
use std::f32::consts::PI;

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up_direction: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, up_direction: Vec3) -> Self {
        Self {
            position,
            target,
            up_direction,
        }
    }

    pub fn transform_vector(&self, input_vector: &Vec3) -> Vec3 {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(&self.up_direction).normalize();
        let up = right.cross(&forward).normalize();
        let transformed = input_vector.x * right + input_vector.y * up - input_vector.z * forward;
        transformed.normalize()
    }

    pub fn rotate_around_target(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let offset = self.position - self.target;
        let radius = offset.magnitude();
        let current_yaw = offset.z.atan2(offset.x);
        let xz_distance = (offset.x * offset.x + offset.z * offset.z).sqrt();
        let current_pitch = (-offset.y).atan2(xz_distance);
        let adjusted_yaw = (current_yaw + delta_yaw) % (2.0 * PI);
        let adjusted_pitch = (current_pitch + delta_pitch).clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
        let new_position = self.target + Vec3::new(
            radius * adjusted_yaw.cos() * adjusted_pitch.cos(),
            -radius * adjusted_pitch.sin(),
            radius * adjusted_yaw.sin() * adjusted_pitch.cos(),
        );

        self.position = new_position;
    }

    pub fn move_towards_target(&mut self, distance: f32) {
        let forward = (self.target - self.position).normalize();
        self.position += forward * distance;
    }

    pub fn move_away_from_target(&mut self, distance: f32) {
        let forward = (self.target - self.position).normalize();
        self.position -= forward * distance;
    }
}
