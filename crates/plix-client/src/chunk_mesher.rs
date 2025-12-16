//! Chunk meshing for per-chunk rendering.
//!
//! T027-T032 [US1]: This module provides chunk-based mesh generation for the
//! chunked world rendering pipeline.

use wgpu::util::DeviceExt;

use plix_common::chunk::{ChunkCoord, CHUNK_SIZE};
use plix_common::types::BlockType;
use plix_common::ChunkedWorld;

use crate::render::engine::Vertex;

/// T027 [US1]: Trait for cross-chunk block lookup during meshing.
///
/// This abstraction allows the mesher to query blocks outside the current
/// chunk (for face culling at chunk boundaries) without directly depending
/// on ChunkedWorld.
pub trait WorldView {
    /// Get block at world coordinates.
    fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType;
}

/// T028 [US1]: WorldView implementation for ChunkedWorld.
impl WorldView for ChunkedWorld {
    fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType {
        self.get_block_xyz(x, y, z)
    }
}

/// T029 [US1]: Per-chunk GPU mesh with vertex and index buffers.
pub struct ChunkMesh {
    /// GPU vertex buffer
    pub vertex_buffer: wgpu::Buffer,
    /// GPU index buffer
    pub index_buffer: wgpu::Buffer,
    /// Number of indices to draw
    pub num_indices: u32,
    /// Whether this mesh is empty (no visible faces)
    pub is_empty: bool,
}

impl ChunkMesh {
    /// Create a new chunk mesh from vertex/index data.
    pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u32]) -> Self {
        let is_empty = indices.is_empty();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            is_empty,
        }
    }
}

/// T030-T032 [US1]: Chunk mesher that generates per-chunk meshes.
pub struct ChunkMesher;

impl ChunkMesher {
    /// T030 [US1]: Generate mesh data for a single chunk.
    ///
    /// Iterates all blocks in the chunk and emits visible faces only
    /// (faces adjacent to air or out-of-bounds).
    pub fn mesh_chunk<W: WorldView>(coord: ChunkCoord, world: &W) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // World offset for this chunk
        let base_x = coord.x * CHUNK_SIZE as i32;
        let base_y = coord.y * CHUNK_SIZE as i32;
        let base_z = coord.z * CHUNK_SIZE as i32;

        for lz in 0..CHUNK_SIZE {
            for ly in 0..CHUNK_SIZE {
                for lx in 0..CHUNK_SIZE {
                    let wx = base_x + lx as i32;
                    let wy = base_y + ly as i32;
                    let wz = base_z + lz as i32;

                    let block = world.get_block(wx, wy, wz);
                    if !block.is_solid() {
                        continue;
                    }

                    let color = Self::block_color(block);
                    let fx = wx as f32;
                    let fy = wy as f32;
                    let fz = wz as f32;

                    // T031 [US1]: Check each face for visibility using WorldView
                    // (handles cross-chunk lookups transparently)

                    // Top face (y+1)
                    if !world.get_block(wx, wy + 1, wz).is_solid() {
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
                    if !world.get_block(wx, wy - 1, wz).is_solid() {
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
                    if !world.get_block(wx, wy, wz + 1).is_solid() {
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
                    if !world.get_block(wx, wy, wz - 1).is_solid() {
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
                    if !world.get_block(wx + 1, wy, wz).is_solid() {
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
                    if !world.get_block(wx - 1, wy, wz).is_solid() {
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

    /// T032 [US1]: Create a GPU mesh from mesh data.
    pub fn create_mesh(
        device: &wgpu::Device,
        coord: ChunkCoord,
        world: &impl WorldView,
    ) -> ChunkMesh {
        let (vertices, indices) = Self::mesh_chunk(coord, world);
        ChunkMesh::new(device, &vertices, &indices)
    }

    /// Get color for a block type (matches RenderEngine colors).
    fn block_color(block: BlockType) -> [f32; 3] {
        match block {
            BlockType::STONE => [0.5, 0.5, 0.5],
            BlockType::BRICK => [0.7, 0.3, 0.2],
            BlockType::METAL => [0.6, 0.6, 0.7],
            _ => [0.4, 0.4, 0.4],
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::types::BlockPos;

    /// Test WorldView for unit tests
    struct TestWorld {
        blocks: std::collections::HashMap<(i32, i32, i32), BlockType>,
    }

    impl TestWorld {
        fn new() -> Self {
            Self {
                blocks: std::collections::HashMap::new(),
            }
        }

        fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockType) {
            self.blocks.insert((x, y, z), block);
        }
    }

    impl WorldView for TestWorld {
        fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType {
            self.blocks
                .get(&(x, y, z))
                .copied()
                .unwrap_or(BlockType::AIR)
        }
    }

    #[test]
    fn test_single_block_six_faces() {
        // T037 [US1]: Single block should produce 6 faces (24 vertices, 36 indices)
        let mut world = TestWorld::new();
        world.set_block(0, 0, 0, BlockType::STONE);

        let coord = ChunkCoord::new(0, 0, 0);
        let (vertices, indices) = ChunkMesher::mesh_chunk(coord, &world);

        // 6 faces * 4 vertices = 24 vertices
        assert_eq!(vertices.len(), 24, "Single block should have 24 vertices");
        // 6 faces * 6 indices = 36 indices
        assert_eq!(indices.len(), 36, "Single block should have 36 indices");
    }

    #[test]
    fn test_two_adjacent_blocks_ten_faces() {
        // T037 [US1]: Two adjacent blocks share one face, so 12 - 2 = 10 visible faces
        let mut world = TestWorld::new();
        world.set_block(0, 0, 0, BlockType::STONE);
        world.set_block(1, 0, 0, BlockType::STONE); // Adjacent in X

        let coord = ChunkCoord::new(0, 0, 0);
        let (vertices, indices) = ChunkMesher::mesh_chunk(coord, &world);

        // 10 faces * 4 vertices = 40 vertices
        assert_eq!(
            vertices.len(),
            40,
            "Two adjacent blocks should have 40 vertices"
        );
        // 10 faces * 6 indices = 60 indices
        assert_eq!(
            indices.len(),
            60,
            "Two adjacent blocks should have 60 indices"
        );
    }

    #[test]
    fn test_empty_chunk_no_faces() {
        let world = TestWorld::new();
        let coord = ChunkCoord::new(0, 0, 0);
        let (vertices, indices) = ChunkMesher::mesh_chunk(coord, &world);

        assert!(vertices.is_empty(), "Empty chunk should have no vertices");
        assert!(indices.is_empty(), "Empty chunk should have no indices");
    }

    #[test]
    fn test_fully_enclosed_block_no_faces() {
        // A block completely surrounded by other blocks should have no visible faces
        let mut world = TestWorld::new();
        // Center block
        world.set_block(1, 1, 1, BlockType::STONE);
        // Surrounding blocks
        world.set_block(0, 1, 1, BlockType::STONE);
        world.set_block(2, 1, 1, BlockType::STONE);
        world.set_block(1, 0, 1, BlockType::STONE);
        world.set_block(1, 2, 1, BlockType::STONE);
        world.set_block(1, 1, 0, BlockType::STONE);
        world.set_block(1, 1, 2, BlockType::STONE);

        let coord = ChunkCoord::new(0, 0, 0);
        let (vertices, indices) = ChunkMesher::mesh_chunk(coord, &world);

        // 7 blocks, each with some hidden faces
        // The center block has 0 visible faces (fully enclosed)
        // Each surrounding block has 5 visible faces
        // Total: 6 * 5 = 30 faces
        let expected_faces = 30;
        assert_eq!(
            vertices.len(),
            expected_faces * 4,
            "Enclosed configuration should have {} vertices",
            expected_faces * 4
        );
    }

    #[test]
    fn test_chunked_world_as_world_view() {
        // Test that ChunkedWorld implements WorldView correctly
        let mut world = ChunkedWorld::new();
        world.set_block(BlockPos::new(5, 5, 5), BlockType::STONE);

        let block = <ChunkedWorld as WorldView>::get_block(&world, 5, 5, 5);
        assert_eq!(block, BlockType::STONE);

        let air = <ChunkedWorld as WorldView>::get_block(&world, 0, 0, 0);
        assert_eq!(air, BlockType::AIR);
    }
}
