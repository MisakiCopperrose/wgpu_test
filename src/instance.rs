pub struct Instance {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub colour: glam::Vec4,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceData {
        InstanceData {
            model: glam::Mat4::from_translation(self.position)
                * glam::Mat4::from_quat(self.rotation),
            colour: self.colour,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct InstanceData {
    pub model: glam::Mat4,
    pub colour: glam::Vec4,
}

unsafe impl bytemuck::Pod for InstanceData {}
unsafe impl bytemuck::Zeroable for InstanceData {}

impl InstanceData {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        2 => Float32x4,
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x4
    ];

    pub fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}
