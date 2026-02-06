#### `src/lib.rs`

The root module defines the core architectural components and error types shared across the visualization engine.

```rust
pub mod math;
pub mod animation;
pub mod steel;
pub mod labels;
pub mod vr;
pub mod rendering;

pub mod prelude { }
```

### Error Handling

Engine-wide errors are consolidated into the `MathAnimationError` enum, providing a unified result type for fallible operations.

```rust
#[derive(Debug, thiserror::Error)]
pub enum MathAnimationError {
    SteelError(String),
    SurfaceError(String),
    LatexError(String),
    AnimationError(String),
    VRError(String),
}

pub type Result<T> = std::result::Result<T, MathAnimationError>;
```

### Core Components

Common ECS components use marker structs and stateful data for scene management.

```rust
#[derive(Component)]
pub struct MathObject;

#[derive(Component)]
pub struct Selectable {
    pub hover_color: Color,
    pub selected_color: Color,
}
```

### Universal Traits

Mathematical identity and verification rely on the `MathHash` and `EpsilonMatch` traits.

```rust
pub trait MathHash {
    fn math_hash(&self) -> u64;
}

pub trait EpsilonMatch {
    fn epsilon_match(&self, other: &Self, epsilon: f32) -> bool;
}
```

### The Geometry Trait

```rust
/// All renderable geometry implements the `Geometry` trait, which bridges the mathematical definition to the Bevy rendering pipeline.
pub trait Geometry: MathHash + EpsilonMatch + Send + Sync {
    fn generate_mesh(&self, meshes: &mut Assets<Mesh>) -> Handle<Mesh>;
}

/// Specialized trait for 1D manifolds (curves) in 3D space.
/// Extends `Geometry` to provide frame orientation logic.
pub trait CurveGeometry: Geometry {
    /// Computes the Frenet-Serret frame at progress t [0.0, 1.0].
    /// Returns (Tangent, Normal, Binormal).
    fn frenet_frame(&self, t: f32) -> (Vec3, Vec3, Vec3);
    
    /// Samples the curve at the given resolution to produce a sequence of points.
    fn sample(&self, resolution: usize) -> Vec<Vec3>;
}

/// Trait for defining a continuous transformation between two mathematical objects.
/// The `To` parameter allows for cross-type transformations (e.g., ImplicitSurface -> ParametricCurve).
pub trait Homotopy<To = Self>: Send + Sync {
    /// Produces a new object at the given progress t [0.0, 1.0].
    /// Returning `Self` ensuring the animated entity maintains its primary component type.
    fn homotope(&self, other: &To, t: f32) -> Self;
}
```

---
