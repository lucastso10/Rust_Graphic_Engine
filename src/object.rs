//use crate::MyVertex;

use glam::f32::{Mat4, Vec3};

pub struct Object {
    //pub model: Vec<MyVertex>,
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3,
}

impl Object {
    pub fn new(
        translation: Vec3,
        scale: Vec3,
        rotation: Vec3,
        //model: Vec<MyVertex>,
    ) -> Self {
        Self {
            //model,
            translation,
            scale,
            rotation,
        }
    }

    pub fn calculate_matrix(&mut self) -> [[f32; 4]; 4] {
        let c3 = f32::cos(self.rotation.z);
        let s3 = f32::sin(self.rotation.z);
        let c2 = f32::cos(self.rotation.x);
        let s2 = f32::sin(self.rotation.x);
        let c1 = f32::cos(self.rotation.y);
        let s1 = f32::sin(self.rotation.y);
        Mat4::from_cols_array(&[
            self.scale.x * (c1 * c3 + s1 * s2 * s3),
            self.scale.x * (c2 * s3),
            self.scale.x * (c1 * s2 * s3 - c3 * s1),
            0.0,
            self.scale.y * (c3 * s1 * s2 - c1 * s3),
            self.scale.y * (c2 * c3),
            self.scale.y * (c1 * c3 * s2 + s1 * s3),
            0.0,
            self.scale.z * (c2 * s1),
            self.scale.z * (-s2),
            self.scale.z * (c1 * c2),
            0.0,
            self.translation.x, self.translation.y, self.translation.z, 1.0
        ]).to_cols_array_2d()
    }
}
