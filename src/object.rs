//use crate::MyVertex;

use glam::f32::{Mat2, Vec2};
use vulkano::padded::Padded;

pub struct Object {
    //pub model: Vec<MyVertex>,
    transform: Mat2,
}

impl Object {
    pub fn new(
        //model: Vec<MyVertex>,
    ) -> Self {
        Self {
            //model,
            transform: Mat2::IDENTITY,
        }
    }

    pub fn manipulate(&mut self, scale: Vec2, rotation: f32) -> [[f32; 2]; 2]{
        let rotation_scale = Mat2::from_scale_angle(scale, rotation);

        self.transform *= rotation_scale;

        self.transform.to_cols_array_2d()
    }
}
