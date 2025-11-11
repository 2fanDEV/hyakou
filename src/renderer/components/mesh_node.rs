use log::debug;
use nalgebra::{Matrix4, Perspective3, Point3, Vector3};

use crate::renderer::geometry::mesh::Mesh;

pub struct MeshNode {
    mesh: Mesh,
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    projection: Matrix4<f32>
}

impl MeshNode {
    pub fn new(mesh: Mesh, m: [[f32; 4]; 4]) -> MeshNode {
        let eye = Point3::<f32>::new(0.0, 0.0, 3.0);
        let target = Point3::<f32>::new(0.0,0.0,0.0);
        let up = Vector3::<f32>::y_axis();
        let mesh_node = MeshNode {
            mesh,
            #[rustfmt::skip]
            model: Matrix4::new(
                 m[0][0], m[0][1], m[0][2], m[0][3],
                 m[1][0], m[1][1], m[1][2], m[1][3], 
                 m[2][0], m[2][1], m[2][2], m[2][3], 
                 m[3][0], m[3][1], m[3][2], m[3][3]
                ),
            view: Perspective3::new(0.1, 1.0, 10.0, 0.1).to_homogeneous(),
            projection: Matrix4::look_at_rh(&eye, &target, &up)
        };
        debug!("{:?}", mesh_node.model);
        debug!("{:?}", mesh_node.view);
        debug!("{:?}", mesh_node.projection);
        let mvp = mesh_node.projection * mesh_node.view * mesh_node.model;
        debug!("{:?}", mvp);
        mesh_node
    }
}