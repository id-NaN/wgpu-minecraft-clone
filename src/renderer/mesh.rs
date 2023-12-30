use color_eyre::eyre::ContextCompat;
use color_eyre::Result;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub tex_coords: glm::Vec2,
    pub normal: glm::Vec3,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

fn update_buffer(
    buffer: &mut Option<wgpu::Buffer>,
    data: &[u8],
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    usage: wgpu::BufferUsages,
) {
    if let Some(buffer) = buffer && buffer.size() >= data.len() as u64 {
        queue.write_buffer(buffer, 0, data);
    } else {
        *buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: data,
            usage
        }));
    }
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
        }
    }

    pub fn add_tri(&mut self, a: Vertex, b: Vertex, c: Vertex) {
        let base_index = self.vertices.len() as u32;
        self.vertices.push(a);
        self.vertices.push(b);
        self.vertices.push(c);
        self.indices.push(base_index);
        self.indices.push(base_index + 1);
        self.indices.push(base_index + 2);
    }

    pub fn add_quad(&mut self, a: Vertex, b: Vertex, c: Vertex, d: Vertex) {
        let base_index = self.vertices.len() as u32;
        self.vertices.push(a);
        self.vertices.push(b);
        self.vertices.push(c);
        self.vertices.push(d);
        self.indices.push(base_index);
        self.indices.push(base_index + 1);
        self.indices.push(base_index + 2);
        self.indices.push(base_index);
        self.indices.push(base_index + 2);
        self.indices.push(base_index + 3);
    }

    pub fn update_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        update_buffer(
            &mut self.vertex_buffer,
            bytemuck::cast_slice(&self.vertices),
            device,
            queue,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        );
        update_buffer(
            &mut self.index_buffer,
            bytemuck::cast_slice(&self.indices),
            device,
            queue,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
        );
    }

    pub fn vertex_buffer(&self) -> Result<&wgpu::Buffer> {
        self.vertex_buffer
            .as_ref()
            .wrap_err("No vertex buffer, call update_buffers() first!")
    }

    pub fn index_buffer(&self) -> Result<&wgpu::Buffer> {
        self.index_buffer
            .as_ref()
            .wrap_err("No index buffer, call update_buffers() first!")
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }
}
