//use crate::MyVertex;

use glam::{f32::{Mat4, Vec3}, Vec4};

pub struct Object {
    //pub model: Vec<MyVertex>,
    pub transform: Mat4,
}

impl Object {
    pub fn new(
        //model: Vec<MyVertex>,
    ) -> Self {
        Self {
            //model,
            transform: Mat4::IDENTITY,
        }
    }

    pub fn scale(&mut self, scale: Vec3) {
        self.transform = self.transform.mul_mat4(&Mat4::from_scale(scale))
    }

    pub fn rotate(&mut self, z: f32, x: f32, y: f32) {
        self.transform *= Mat4::from_rotation_z(z);
        self.transform *= Mat4::from_rotation_y(y);
        self.transform *= Mat4::from_rotation_x(x);
        println!("{}", self.transform.mul_vec4(Vec4::from_array([-0.5, 0.5, -0.5, 1.0])))
    }

    pub fn translation(&mut self, translation: Vec3) {
        self.transform = self.transform.mul_mat4(&Mat4::from_translation(translation));
    }
}
