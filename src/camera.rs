use glam::{Mat4, Vec3};

use crate::keyboard::{Keyboard, Keys};

pub struct Camera {
    pub projection: Mat4,
    pub view: Mat4,
    pub translation: Vec3,
    pub rotation: Vec3,
    move_speed: f32,
    look_speed: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Camera {
        let mut camera = Camera {
            projection: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            move_speed: 3.0,
            look_speed: 1.5,
        };
        // 0.87266462599716 = 50 graus
        camera.perspective_view(0.87266462599716, aspect_ratio, 0.1, 100.0);

        camera.set_view_target(
            Vec3::from_array([0.0, 0.0, -10.0]),
            Vec3::from_array([0.0, 0.0, 2.5]),
            Vec3::from_array([0.0, -1.0, 0.0]),
        );
        camera
    }
    pub fn set_view_direction(&mut self, position: Vec3, direction: Vec3, up: Vec3) {
        let w = direction.normalize();
        let u = w.cross(up).normalize();
        let v = w.cross(u);

        self.view = Mat4::from_cols(
            [u.x, v.x, w.x, 0.0].into(),
            [u.y, v.y, w.y, 0.0].into(),
            [u.z, v.z, w.z, 0.0].into(),
            [-u.dot(position), -v.dot(position), -w.dot(position), 1.0].into(),
        );
    }

    pub fn set_view_target(&mut self, position: Vec3, target: Vec3, up: Vec3) {
        Self::set_view_direction(self, position, target - position, up);
    }

    pub fn set_view_yxz(&mut self, position: Vec3, rotation: Vec3) {
        let c3 = f32::cos(rotation.z);
        let s3 = f32::sin(rotation.z);
        let c2 = f32::cos(rotation.x);
        let s2 = f32::sin(rotation.x);
        let c1 = f32::cos(rotation.y);
        let s1 = f32::sin(rotation.y);

        let u = Vec3::from_array([
            (c1 * c3 + s1 * s2 * s3),
            (c2 * s3),
            (c1 * s2 * s3 - c3 * s1),
        ]);
        let v = Vec3::from_array([
            (c3 * s1 * s2 - c1 * s3),
            (c2 * c3),
            (c1 * c3 * s2 + s1 * s3),
        ]);
        let w = Vec3::from_array([(c2 * s1), (-s2), (c1 * c2)]);

        self.view = Mat4::from_cols(
            [u.x, v.x, w.x, 0.0].into(),
            [u.y, v.y, w.y, 0.0].into(),
            [u.z, v.z, w.z, 0.0].into(),
            [-u.dot(position), -v.dot(position), -w.dot(position), 1.0].into(),
        );
    }

    pub fn orthographic_view(
        &mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) {
        self.projection *= Mat4::orthographic_lh(left, right, bottom, top, near, far);
    }

    pub fn perspective_view(&mut self, fov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) {
        self.projection *= Mat4::perspective_lh(fov, aspect_ratio, z_near, z_far);
    }


    pub fn calculate_matrix(&mut self) -> [[f32; 4]; 4] {
        let c3 = f32::cos(self.rotation.z);
        let s3 = f32::sin(self.rotation.z);
        let c2 = f32::cos(self.rotation.x);
        let s2 = f32::sin(self.rotation.x);
        let c1 = f32::cos(self.rotation.y);
        let s1 = f32::sin(self.rotation.y);
        Mat4::from_cols_array(&[
            (c1 * c3 + s1 * s2 * s3),
            (c2 * s3),
            (c1 * s2 * s3 - c3 * s1),
            0.0,
            (c3 * s1 * s2 - c1 * s3),
            (c2 * c3),
            (c1 * c3 * s2 + s1 * s3),
            0.0,
            (c2 * s1),
            (-s2),
            (c1 * c2),
            0.0,
            self.translation.x,
            self.translation.y,
            self.translation.z,
            1.0,
        ])
        .to_cols_array_2d()
    }

    pub fn move_camera(&mut self, delta_time: f32, keys: &Keyboard ) {
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
            self.rotation += self.look_speed * delta_time * rotate.normalize();
        }

        self.rotation.x = self.rotation.x.clamp(-1.5, 1.5);
        self.rotation.y = self.rotation.y % std::f32::consts::TAU;

        let yaw = self.rotation.y;
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
            self.translation += self.move_speed * delta_time * move_dir;
        }
        self.set_view_yxz(self.translation, self.rotation);
    }
}
