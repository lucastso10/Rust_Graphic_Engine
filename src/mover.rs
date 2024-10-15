use glam::{Mat4, Vec3};

use crate::{
    keyboard::{Keyboard, Keys},
    object::Object,
};

pub struct Mover {
    move_speed: f32,
    look_speed: f32,
}

impl Default for Mover {
    fn default() -> Self {
        Mover {
            move_speed: 3.0,
            look_speed: 1.5,
        }
    }
}

impl Mover {
    pub fn movement(&self, delta_time: f32, object: &mut Object, keys: &Keyboard) {
        let mut rotate: Vec3 = Vec3::ZERO;

        for command in keys.active.clone() {
            match command {
                Keys::RotateUp => rotate.x += 1.0,
                Keys::RotateDown => rotate.x -= 1.0,
                Keys::RotateRight => rotate.y += 1.0,
                Keys::RotateLeft => rotate.y -= 1.0,
                _ => {}
            }
        }

        if rotate.dot(rotate) > std::f32::EPSILON {
            object.rotation += self.look_speed * delta_time * rotate.normalize();
        }

        object.rotation.x = object.rotation.x.clamp(-1.5, 1.5);
        object.rotation.y = object.rotation.y % std::f32::consts::TAU;

        let yaw = object.rotation.y;
        let forward_dir = Vec3::from_array([yaw.sin(), 0.0, yaw.cos()]);
        let right_dir = Vec3::from_array([forward_dir.z, 0.0, -forward_dir.x]);
        let up_dir = Vec3::from_array([0.0, -1.0, 0.0]);

        let mut move_dir = Vec3::ZERO;
        for command in keys.active.clone() {
            match command {
                Keys::MoveUp => move_dir += up_dir,
                Keys::MoveDown => move_dir -= up_dir,
                Keys::MoveRight => move_dir += right_dir,
                Keys::MoveLeft => move_dir -= right_dir,
                Keys::MoveForward => move_dir += forward_dir,
                Keys::MoveBackward => move_dir -= forward_dir,
                _ => {}
            }
        }

        if move_dir.dot(move_dir) > std::f32::EPSILON {
            object.translation += self.move_speed * delta_time * move_dir;
        }
    }
}
