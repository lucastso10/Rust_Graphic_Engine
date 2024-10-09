//use crate::MyVertex;

use glam::{f32::{Mat4, Vec3}, Quat};

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
        self.transform *= Mat4::from_scale(scale);
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.transform *= Mat4::from_quat(rotation);
    }

    pub fn translation(&mut self, translation: Vec3) {
        self.transform *= Mat4::from_translation(translation);
    }
}
