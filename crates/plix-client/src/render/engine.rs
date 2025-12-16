//! wgpu rendering engine

use std::collections::HashMap;
use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use plix_common::chunk::ChunkCoord;
use plix_common::math::Vec3;
use plix_common::types::BlockType;
use plix_common::ChunkedWorld;

use super::Camera;
use crate::chunk_mesher::{ChunkMesh, ChunkMesher};

/// Vertex for 3D rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// T032: Vertex for 2D UI rendering (screen-space quads)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UIVertex {
    pub position: [f32; 2], // Clip space position (-1 to 1)
    pub color: [f32; 4],    // RGBA color
}

impl UIVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UIVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// A UI quad for 2D rendering
#[derive(Clone, Debug)]
pub struct UIQuad {
    /// Center position in pixels (relative to screen center)
    pub x: f32,
    pub y: f32,
    /// Size in pixels
    pub width: f32,
    pub height: f32,
    /// RGBA color
    pub color: [f32; 4],
}

/// Uniform buffer for camera matrices
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_proj = camera.view_projection_matrix().to_cols_array_2d();
    }
}

/// A mesh with vertices and indices
pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Mesh {
    fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u32]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

/// Render engine with wgpu state
pub struct RenderEngine {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: wgpu::TextureView,
    // Meshes
    arena_mesh: Option<Mesh>,
    player_mesh: Mesh,
    fallback_mesh: Mesh,
    // Player instances for this frame
    player_instances: Vec<PlayerInstance>,
    // T032: UI render pipeline for 2D overlays
    ui_pipeline: wgpu::RenderPipeline,
    // T035 [US1]: Per-chunk mesh storage for chunked rendering
    chunk_meshes: HashMap<ChunkCoord, ChunkMesh>,
}

/// Player instance data for rendering
pub struct PlayerInstance {
    pub position: Vec3,
    pub color: [f32; 3],
}

impl RenderEngine {
    /// Create a new render engine
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Camera uniform buffer
        let camera_uniform = CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // Pipeline layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Depth texture
        let depth_texture = Self::create_depth_texture(&device, &config);

        // Render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Create fallback scene (floor grid + cubes)
        let (vertices, indices) = Self::create_fallback_geometry();
        let fallback_mesh = Mesh::new(&device, &vertices, &indices);

        // Create player capsule mesh
        let (vertices, indices) = Self::create_player_geometry([0.8, 0.2, 0.2]);
        let player_mesh = Mesh::new(&device, &vertices, &indices);

        // T032: Create UI shader and pipeline for 2D overlays
        let ui_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ui.wgsl").into()),
        });

        let ui_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&ui_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &ui_shader,
                entry_point: Some("vs_main"),
                buffers: &[UIVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ui_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for UI
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // No depth testing for UI
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            arena_mesh: None,
            player_mesh,
            fallback_mesh,
            player_instances: Vec::new(),
            ui_pipeline,
            chunk_meshes: HashMap::new(),
        }
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    /// Get block color
    fn block_color(block: BlockType) -> [f32; 3] {
        match block {
            BlockType::STONE => [0.5, 0.5, 0.5],
            BlockType::BRICK => [0.7, 0.3, 0.2],
            BlockType::METAL => [0.6, 0.6, 0.7],
            _ => [0.4, 0.4, 0.4],
        }
    }

    /// Load arena and generate mesh
    pub fn load_arena(&mut self, size: [u32; 3], blocks: &[BlockType]) {
        let (vertices, indices) = Self::generate_arena_mesh(size, blocks);
        if !vertices.is_empty() {
            self.arena_mesh = Some(Mesh::new(&self.device, &vertices, &indices));
        }
    }

    /// Generate mesh from arena blocks (simple per-face approach)
    fn generate_arena_mesh(size: [u32; 3], blocks: &[BlockType]) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let [sx, sy, sz] = size;

        let get_block = |x: i32, y: i32, z: i32| -> BlockType {
            if x < 0 || y < 0 || z < 0 || x >= sx as i32 || y >= sy as i32 || z >= sz as i32 {
                return BlockType::AIR;
            }
            let idx = (z as u32 * sy * sx + y as u32 * sx + x as u32) as usize;
            blocks.get(idx).copied().unwrap_or(BlockType::AIR)
        };

        for z in 0..sz {
            for y in 0..sy {
                for x in 0..sx {
                    let block = get_block(x as i32, y as i32, z as i32);
                    if block == BlockType::AIR {
                        continue;
                    }

                    let color = Self::block_color(block);
                    let fx = x as f32;
                    let fy = y as f32;
                    let fz = z as f32;

                    // Check each face for visibility
                    // Top face (y+1)
                    if !get_block(x as i32, y as i32 + 1, z as i32).is_solid() {
                        Self::add_face(
                            &mut vertices,
                            &mut indices,
                            [fx, fy + 1.0, fz],
                            [fx + 1.0, fy + 1.0, fz],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                            [fx, fy + 1.0, fz + 1.0],
                            Self::lighten(color, 0.1),
                        );
                    }
                    // Bottom face (y-1)
                    if !get_block(x as i32, y as i32 - 1, z as i32).is_solid() {
                        Self::add_face(
                            &mut vertices,
                            &mut indices,
                            [fx, fy, fz + 1.0],
                            [fx + 1.0, fy, fz + 1.0],
                            [fx + 1.0, fy, fz],
                            [fx, fy, fz],
                            Self::darken(color, 0.2),
                        );
                    }
                    // Front face (z+1)
                    if !get_block(x as i32, y as i32, z as i32 + 1).is_solid() {
                        Self::add_face(
                            &mut vertices,
                            &mut indices,
                            [fx, fy, fz + 1.0],
                            [fx, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy, fz + 1.0],
                            color,
                        );
                    }
                    // Back face (z-1)
                    if !get_block(x as i32, y as i32, z as i32 - 1).is_solid() {
                        Self::add_face(
                            &mut vertices,
                            &mut indices,
                            [fx + 1.0, fy, fz],
                            [fx + 1.0, fy + 1.0, fz],
                            [fx, fy + 1.0, fz],
                            [fx, fy, fz],
                            Self::darken(color, 0.05),
                        );
                    }
                    // Right face (x+1)
                    if !get_block(x as i32 + 1, y as i32, z as i32).is_solid() {
                        Self::add_face(
                            &mut vertices,
                            &mut indices,
                            [fx + 1.0, fy, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz],
                            [fx + 1.0, fy, fz],
                            Self::darken(color, 0.1),
                        );
                    }
                    // Left face (x-1)
                    if !get_block(x as i32 - 1, y as i32, z as i32).is_solid() {
                        Self::add_face(
                            &mut vertices,
                            &mut indices,
                            [fx, fy, fz],
                            [fx, fy + 1.0, fz],
                            [fx, fy + 1.0, fz + 1.0],
                            [fx, fy, fz + 1.0],
                            Self::lighten(color, 0.05),
                        );
                    }
                }
            }
        }

        (vertices, indices)
    }

    fn lighten(color: [f32; 3], amount: f32) -> [f32; 3] {
        [
            (color[0] + amount).min(1.0),
            (color[1] + amount).min(1.0),
            (color[2] + amount).min(1.0),
        ]
    }

    fn darken(color: [f32; 3], amount: f32) -> [f32; 3] {
        [
            (color[0] - amount).max(0.0),
            (color[1] - amount).max(0.0),
            (color[2] - amount).max(0.0),
        ]
    }

    fn add_face(
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        p0: [f32; 3],
        p1: [f32; 3],
        p2: [f32; 3],
        p3: [f32; 3],
        color: [f32; 3],
    ) {
        let base = vertices.len() as u32;
        vertices.push(Vertex {
            position: p0,
            color,
        });
        vertices.push(Vertex {
            position: p1,
            color,
        });
        vertices.push(Vertex {
            position: p2,
            color,
        });
        vertices.push(Vertex {
            position: p3,
            color,
        });
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Create fallback scene: floor grid + some cubes
    fn create_fallback_geometry() -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Floor grid (gray)
        let floor_color = [0.3, 0.3, 0.35];
        let grid_size = 32;
        let grid_y = 0.0;

        for x in -grid_size..grid_size {
            for z in -grid_size..grid_size {
                let fx = x as f32;
                let fz = z as f32;

                // Checkerboard pattern
                let color = if (x + z) % 2 == 0 {
                    floor_color
                } else {
                    [0.25, 0.25, 0.3]
                };

                Self::add_face(
                    &mut vertices,
                    &mut indices,
                    [fx, grid_y, fz],
                    [fx, grid_y, fz + 1.0],
                    [fx + 1.0, grid_y, fz + 1.0],
                    [fx + 1.0, grid_y, fz],
                    color,
                );
            }
        }

        // Add some colored cubes as landmarks
        let cubes = [
            ([5.0, 0.5, 5.0], [0.8, 0.2, 0.2]),
            ([-5.0, 0.5, 5.0], [0.2, 0.8, 0.2]),
            ([5.0, 0.5, -5.0], [0.2, 0.2, 0.8]),
            ([-5.0, 0.5, -5.0], [0.8, 0.8, 0.2]),
            ([0.0, 1.0, 0.0], [0.8, 0.4, 0.8]),
        ];

        for (pos, color) in cubes {
            Self::add_cube(&mut vertices, &mut indices, pos, color, 1.0);
        }

        (vertices, indices)
    }

    /// Create player geometry (simple box/capsule)
    fn create_player_geometry(color: [f32; 3]) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Player is 0.6 wide, 1.8 tall, centered at feet
        Self::add_cube(&mut vertices, &mut indices, [0.0, 0.9, 0.0], color, 0.6);

        (vertices, indices)
    }

    fn add_cube(
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        center: [f32; 3],
        color: [f32; 3],
        size: f32,
    ) {
        let s = size / 2.0;
        let [cx, cy, cz] = center;

        // Top
        Self::add_face(
            vertices,
            indices,
            [cx - s, cy + s, cz - s],
            [cx - s, cy + s, cz + s],
            [cx + s, cy + s, cz + s],
            [cx + s, cy + s, cz - s],
            Self::lighten(color, 0.1),
        );
        // Bottom
        Self::add_face(
            vertices,
            indices,
            [cx - s, cy - s, cz + s],
            [cx - s, cy - s, cz - s],
            [cx + s, cy - s, cz - s],
            [cx + s, cy - s, cz + s],
            Self::darken(color, 0.2),
        );
        // Front
        Self::add_face(
            vertices,
            indices,
            [cx - s, cy - s, cz + s],
            [cx + s, cy - s, cz + s],
            [cx + s, cy + s, cz + s],
            [cx - s, cy + s, cz + s],
            color,
        );
        // Back
        Self::add_face(
            vertices,
            indices,
            [cx + s, cy - s, cz - s],
            [cx - s, cy - s, cz - s],
            [cx - s, cy + s, cz - s],
            [cx + s, cy + s, cz - s],
            Self::darken(color, 0.05),
        );
        // Right
        Self::add_face(
            vertices,
            indices,
            [cx + s, cy - s, cz + s],
            [cx + s, cy - s, cz - s],
            [cx + s, cy + s, cz - s],
            [cx + s, cy + s, cz + s],
            Self::darken(color, 0.1),
        );
        // Left
        Self::add_face(
            vertices,
            indices,
            [cx - s, cy - s, cz - s],
            [cx - s, cy - s, cz + s],
            [cx - s, cy + s, cz + s],
            [cx - s, cy + s, cz - s],
            Self::lighten(color, 0.05),
        );
    }

    /// Set players to render this frame
    pub fn set_players(&mut self, players: Vec<PlayerInstance>) {
        self.player_instances = players;
    }

    /// Resize the surface
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Self::create_depth_texture(&self.device, &self.config);
        }
    }

    /// Get current size
    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    /// Check if arena is loaded (either monolithic or chunked)
    pub fn has_arena(&self) -> bool {
        self.arena_mesh.is_some() || !self.chunk_meshes.is_empty()
    }

    /// T035 [US1]: Load a chunked world, building meshes for all chunks.
    ///
    /// This replaces the monolithic arena mesh with per-chunk meshes.
    pub fn load_chunked_world(&mut self, world: &ChunkedWorld) {
        // Clear existing meshes
        self.arena_mesh = None;
        self.chunk_meshes.clear();

        // Build mesh for each chunk
        for (coord, _chunk) in world.iter_chunks() {
            let mesh = ChunkMesher::create_mesh(&self.device, *coord, world);
            if !mesh.is_empty {
                self.chunk_meshes.insert(*coord, mesh);
            }
        }
    }

    /// T035 [US1]: Rebuild mesh for specific chunks.
    ///
    /// Called when blocks are edited to update only affected chunks.
    pub fn rebuild_chunk_meshes(&mut self, coords: &[ChunkCoord], world: &ChunkedWorld) {
        for coord in coords {
            // Remove old mesh
            self.chunk_meshes.remove(coord);

            // Only build if chunk exists in world
            if world.has_chunk(*coord) {
                let mesh = ChunkMesher::create_mesh(&self.device, *coord, world);
                if !mesh.is_empty {
                    self.chunk_meshes.insert(*coord, mesh);
                }
            }
        }
    }

    /// T035 [US1]: Get number of loaded chunk meshes.
    pub fn chunk_mesh_count(&self) -> usize {
        self.chunk_meshes.len()
    }

    /// Get reference to device (for external mesh creation).
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Update camera uniform
    pub fn update_camera(&mut self, camera: &Camera) {
        self.camera_uniform.update(camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    /// T033: Convert UI quads to vertices for rendering
    fn quads_to_vertices(&self, quads: &[UIQuad]) -> (Vec<UIVertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let screen_w = self.size.width as f32;
        let screen_h = self.size.height as f32;

        for quad in quads {
            // Convert pixel coordinates to clip space (-1 to 1)
            // x, y are offsets from screen center
            let half_w = quad.width / 2.0;
            let half_h = quad.height / 2.0;

            let left = (quad.x - half_w) / (screen_w / 2.0);
            let right = (quad.x + half_w) / (screen_w / 2.0);
            let top = -(quad.y - half_h) / (screen_h / 2.0); // Y is inverted in clip space
            let bottom = -(quad.y + half_h) / (screen_h / 2.0);

            let base = vertices.len() as u32;

            vertices.push(UIVertex {
                position: [left, top],
                color: quad.color,
            });
            vertices.push(UIVertex {
                position: [right, top],
                color: quad.color,
            });
            vertices.push(UIVertex {
                position: [right, bottom],
                color: quad.color,
            });
            vertices.push(UIVertex {
                position: [left, bottom],
                color: quad.color,
            });

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        (vertices, indices)
    }

    /// Render a frame with optional UI quads
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.render_with_ui(&[])
    }

    /// T033: Render a frame with UI quads overlay
    pub fn render_with_ui(&mut self, ui_quads: &[UIQuad]) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Create player meshes for this frame
        let player_meshes: Vec<Mesh> = self
            .player_instances
            .iter()
            .map(|p| {
                let mut vertices = Vec::new();
                let mut indices = Vec::new();
                Self::add_cube(
                    &mut vertices,
                    &mut indices,
                    [p.position.x, p.position.y + 0.9, p.position.z],
                    p.color,
                    0.6,
                );
                Mesh::new(&self.device, &vertices, &indices)
            })
            .collect();

        // Convert UI quads to vertex data
        let (ui_vertices, ui_indices) = self.quads_to_vertices(ui_quads);
        let ui_vertex_buffer = if !ui_vertices.is_empty() {
            Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("UI Vertex Buffer"),
                        contents: bytemuck::cast_slice(&ui_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            )
        } else {
            None
        };
        let ui_index_buffer = if !ui_indices.is_empty() {
            Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("UI Index Buffer"),
                        contents: bytemuck::cast_slice(&ui_indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
            )
        } else {
            None
        };

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.4,
                            g: 0.6,
                            b: 0.9,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // T035 [US1]: Render chunk meshes if available, otherwise arena or fallback
            if !self.chunk_meshes.is_empty() {
                // Render all chunk meshes
                for mesh in self.chunk_meshes.values() {
                    if mesh.num_indices > 0 {
                        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
                    }
                }
            } else if let Some(mesh) = &self.arena_mesh {
                // Legacy monolithic arena mesh
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
            } else {
                // Fallback scene
                render_pass.set_vertex_buffer(0, self.fallback_mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.fallback_mesh.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..self.fallback_mesh.num_indices, 0, 0..1);
            }

            // Render players
            for mesh in &player_meshes {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
            }
        }

        // T033: UI render pass (after 3D, no depth testing)
        if let (Some(vertex_buf), Some(index_buf)) = (&ui_vertex_buffer, &ui_index_buffer) {
            let mut ui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear, overlay on 3D
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None, // No depth testing for UI
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            ui_pass.set_pipeline(&self.ui_pipeline);
            ui_pass.set_vertex_buffer(0, vertex_buf.slice(..));
            ui_pass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint32);
            ui_pass.draw_indexed(0..ui_indices.len() as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
