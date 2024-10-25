use glam::f32::{Mat4, Vec3};

use crate::MyVertex;

pub struct Object {
    //pub model: Vec<MyVertex>,
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3,
    pub model: Model,
}

impl Object {
    pub fn new(file_name: &str) -> Object {
        let (models, _materials) = tobj::load_obj(file_name, &tobj::GPU_LOAD_OPTIONS).expect("Failed to load obj!");

        // suporte para apenas um modelo no arquivo
        let mesh = &models[0].mesh;

        let mut vertices: Vec<MyVertex> = vec![];
        let mut indices: Vec<u32> = vec![];
        for i in 0..mesh.indices.len() { 
            let index = mesh.indices[i] as usize;
            let vertex = MyVertex {
                position: [mesh.positions[index * 3], mesh.positions[index * 3 + 1], mesh.positions[index * 3 + 2]],
                color: [0.5, 0.5, 0.5],
                normal: [mesh.normals[index * 3], mesh.normals[index * 3 + 1], mesh.normals[index * 3 + 2]],
                texcoord: [mesh.texcoords[index * 2], mesh.texcoords[index * 2 + 1]],
            };

            if vertices.contains(&vertex) {
                indices.push(vertices.iter().position(|r| *r == vertex).unwrap().try_into().unwrap());
            } else {
                vertices.push(vertex.clone());
                indices.push(vertices.iter().position(|r| *r == vertex).unwrap().try_into().unwrap());
            }
        }

        let model = Model {
            vertices,
            indices,
        };
        Self {
            translation: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: Vec3::ZERO,
            model,
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
            self.translation.x,
            self.translation.y,
            self.translation.z,
            1.0,
        ])
        .to_cols_array_2d()
    }
}

pub struct Model {
    pub vertices: Vec<MyVertex>,
    pub indices: Vec<u32>,
}

impl Clone for Model {
    fn clone(&self) -> Self {
        Model {
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
        }
    }
}
