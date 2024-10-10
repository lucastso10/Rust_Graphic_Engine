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
        let mut matrix = Mat4::IDENTITY * Mat4::from_translation(self.translation);

        matrix *= Mat4::from_rotation_y(self.rotation.y);
        matrix *= Mat4::from_rotation_x(self.rotation.x);
        matrix *= Mat4::from_rotation_z(self.rotation.z);

        matrix *= Mat4::from_scale(self.scale);

        matrix.to_cols_array_2d()
    }
}
