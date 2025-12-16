# API Contract: Chunk System

**Feature**: 011-chunked-world
**Date**: 2025-12-16
**Type**: Internal Rust API

## Overview

This document defines the internal API contracts for the chunk system. These are Rust trait and struct interfaces, not network protocols.

---

## Core Types (plix-common)

### ChunkCoord

```rust
/// Type alias for chunk coordinates (same as ChunkPos)
pub type ChunkCoord = ChunkPos;

/// Chunk coordinate in chunk space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self;

    /// Returns the 6 face-adjacent neighbor coordinates
    pub fn neighbors(&self) -> [ChunkPos; 6];

    /// World-space center of this chunk
    pub fn world_center(&self) -> glam::Vec3;
}
```

### Chunk

```rust
/// A 16x16x16 block region
pub struct Chunk {
    blocks: [BlockType; 4096],
    dirty: bool,
    aabb: AABB,
}

impl Chunk {
    /// Size of chunk in each dimension
    pub const SIZE: usize = 16;

    /// Total block count per chunk
    pub const BLOCK_COUNT: usize = 4096;

    /// Create new chunk filled with AIR
    pub fn new(coord: ChunkCoord) -> Self;

    /// Create chunk from existing block data
    pub fn from_blocks(coord: ChunkCoord, blocks: [BlockType; 4096]) -> Self;

    /// Get block at local coordinate
    /// Panics if local coords out of range [0, 15]
    pub fn get_block(&self, local_x: usize, local_y: usize, local_z: usize) -> BlockType;

    /// Set block at local coordinate
    /// Panics if local coords out of range [0, 15]
    pub fn set_block(&mut self, local_x: usize, local_y: usize, local_z: usize, block: BlockType);

    /// Mark chunk as needing mesh rebuild
    pub fn mark_dirty(&mut self);

    /// Clear dirty flag after rebuild
    pub fn clear_dirty(&mut self);

    /// Check if chunk needs rebuild
    pub fn is_dirty(&self) -> bool;

    /// Get chunk's world-space AABB
    pub fn aabb(&self) -> &AABB;

    /// Check if chunk contains only AIR blocks
    pub fn is_empty(&self) -> bool;
}
```

### ChunkedWorld

```rust
/// Container for all chunks in the world
pub struct ChunkedWorld {
    chunks: HashMap<ChunkCoord, Chunk>,
}

impl ChunkedWorld {
    /// Create empty world
    pub fn new() -> Self;

    /// Get block at world position
    /// Returns AIR if chunk not loaded
    pub fn get_block(&self, pos: BlockPos) -> BlockType;

    /// Set block at world position
    /// Creates chunk if not present
    /// Returns affected chunk coordinates (may include neighbors for boundary edits)
    pub fn set_block(&mut self, pos: BlockPos, block: BlockType) -> Vec<ChunkCoord>;

    /// Get immutable reference to chunk
    pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&Chunk>;

    /// Get mutable reference to chunk
    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk>;

    /// Ensure chunk exists, creating if needed
    pub fn ensure_chunk(&mut self, coord: ChunkCoord) -> &mut Chunk;

    /// Remove and return chunk
    pub fn remove_chunk(&mut self, coord: ChunkCoord) -> Option<Chunk>;

    /// Iterate all loaded chunks
    pub fn iter_chunks(&self) -> impl Iterator<Item = (&ChunkCoord, &Chunk)>;

    /// Iterate chunk coordinates only
    pub fn chunk_coords(&self) -> impl Iterator<Item = &ChunkCoord>;

    /// Count of loaded chunks
    pub fn chunk_count(&self) -> usize;

    /// Check if chunk is loaded
    pub fn has_chunk(&self, coord: ChunkCoord) -> bool;
}
```

### AABB

```rust
/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl AABB {
    /// Create AABB from chunk coordinate
    pub fn from_chunk_coord(coord: ChunkCoord) -> Self;

    /// Center point of the box
    pub fn center(&self) -> glam::Vec3;

    /// Half-extents (size / 2)
    pub fn half_extents(&self) -> glam::Vec3;

    /// Test intersection with frustum planes
    /// Returns true if AABB is at least partially inside frustum
    pub fn intersects_frustum(&self, planes: &[Plane; 6]) -> bool;

    /// Test if point is inside AABB
    pub fn contains_point(&self, point: glam::Vec3) -> bool;
}
```

---

## Coordinate Conversion (plix-common)

```rust
/// Convert world position to chunk coordinate and local position
pub fn world_to_chunk(pos: BlockPos) -> (ChunkCoord, (usize, usize, usize));

/// Convert chunk coordinate and local position to world position
pub fn chunk_to_world(coord: ChunkCoord, local: (usize, usize, usize)) -> BlockPos;

/// Check if local position is on chunk boundary
pub fn is_boundary_local(local: (usize, usize, usize)) -> bool;

/// Get which neighbor chunk is affected by boundary local position
/// Returns None if not on boundary
pub fn boundary_neighbor(coord: ChunkCoord, local: (usize, usize, usize)) -> Option<ChunkCoord>;
```

---

## Client APIs (plix-client)

### WorldView Trait

```rust
/// Abstraction for reading blocks across chunk boundaries
pub trait WorldView {
    /// Get block at world position
    fn get_block(&self, pos: BlockPos) -> BlockType;

    /// Check if position is within loaded chunks
    fn is_loaded(&self, pos: BlockPos) -> bool;
}

impl WorldView for ChunkedWorld {
    // ... implementation
}
```

### ChunkMesher

```rust
/// Generates GPU mesh data for a chunk
pub struct ChunkMesher;

impl ChunkMesher {
    /// Generate mesh vertices and indices for a chunk
    /// Uses world_view for cross-chunk neighbor lookups
    pub fn mesh_chunk(
        chunk: &Chunk,
        coord: ChunkCoord,
        world_view: &impl WorldView,
    ) -> (Vec<Vertex>, Vec<u32>);

    /// Create GPU buffers from mesh data
    pub fn create_mesh(
        device: &wgpu::Device,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> ChunkMesh;
}
```

### ChunkMesh

```rust
/// GPU resources for a chunk's mesh
pub struct ChunkMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl ChunkMesh {
    /// Number of indices to draw
    pub fn num_indices(&self) -> u32;

    /// Get vertex buffer for binding
    pub fn vertex_buffer(&self) -> &wgpu::Buffer;

    /// Get index buffer for binding
    pub fn index_buffer(&self) -> &wgpu::Buffer;
}
```

### ChunkManager

```rust
/// Configuration for chunk manager
pub struct ChunkManagerConfig {
    pub view_distance: u8,       // Default: 8
    pub mesh_budget: u32,        // Default: 2
    pub culling_enabled: bool,   // Default: true
}

impl Default for ChunkManagerConfig {
    fn default() -> Self;
}

/// Manages chunk streaming and mesh scheduling
pub struct ChunkManager {
    config: ChunkManagerConfig,
    loaded: HashSet<ChunkCoord>,
    dirty_queue: VecDeque<ChunkCoord>,
    dirty_set: HashSet<ChunkCoord>,  // For O(1) dedup check
    meshes: HashMap<ChunkCoord, ChunkMesh>,
}

impl ChunkManager {
    /// Create with configuration
    pub fn new(config: ChunkManagerConfig) -> Self;

    /// Create with defaults
    pub fn with_defaults() -> Self;

    /// Update chunk loading and mesh rebuilding
    /// Call once per frame
    pub fn update(
        &mut self,
        player_pos: glam::Vec3,
        world: &mut ChunkedWorld,
        device: &wgpu::Device,
    );

    /// Mark chunk as needing mesh rebuild
    pub fn mark_dirty(&mut self, coord: ChunkCoord);

    /// Get mesh for rendering (None if not yet built or chunk empty)
    pub fn get_mesh(&self, coord: &ChunkCoord) -> Option<&ChunkMesh>;

    /// Iterate visible chunks after culling
    pub fn visible_chunks<'a>(
        &'a self,
        frustum: &Frustum,
        world: &'a ChunkedWorld,
    ) -> impl Iterator<Item = (&'a ChunkCoord, &'a ChunkMesh)>;

    /// Get view distance
    pub fn view_distance(&self) -> u8;

    /// Set view distance (triggers reload)
    pub fn set_view_distance(&mut self, distance: u8);

    /// Number of chunks pending mesh rebuild
    pub fn dirty_count(&self) -> usize;

    /// Number of loaded chunks
    pub fn loaded_count(&self) -> usize;
}
```

### Frustum

```rust
/// View frustum for culling
pub struct Frustum {
    planes: [Plane; 6],  // Left, Right, Bottom, Top, Near, Far
}

impl Frustum {
    /// Extract frustum from view-projection matrix
    pub fn from_view_proj(view_proj: glam::Mat4) -> Self;

    /// Test if AABB intersects frustum
    pub fn intersects_aabb(&self, aabb: &AABB) -> bool;

    /// Test if point is inside frustum
    pub fn contains_point(&self, point: glam::Vec3) -> bool;
}

/// A plane in 3D space (ax + by + cz + d = 0)
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: glam::Vec3,
    pub distance: f32,
}

impl Plane {
    /// Distance from point to plane (positive = in front)
    pub fn distance_to_point(&self, point: glam::Vec3) -> f32;
}
```

---

## Arena Integration (plix-arena)

```rust
impl LoadedArena {
    /// Convert flat block storage to chunked world
    pub fn to_chunked_world(&self) -> ChunkedWorld;
}
```

---

## Error Handling

All chunk operations use the following error handling patterns:

1. **Out-of-bounds local coordinates**: Panic (programming error)
2. **Missing chunk for world access**: Return AIR (graceful degradation)
3. **GPU buffer creation failure**: Propagate wgpu error

---

## Thread Safety

- `Chunk`, `ChunkedWorld`: Not `Send`/`Sync` (single-threaded client)
- `ChunkMesh`: Contains wgpu handles, follows wgpu threading model
- `ChunkManager`: Single-threaded, called from main render loop

Future multi-threaded meshing would require `Arc<Mutex<>>` or channel-based design.
