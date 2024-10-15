use glam::{Mat4, Vec3};

pub struct Camera {
    pub projection: Mat4,
    pub view: Mat4,
}

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            projection: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
        }
    }
}

impl Camera {
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
}
