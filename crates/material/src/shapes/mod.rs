//! M3 Expressive shapes: the 35-shape library and general morphing.
//!
//! - [`ShapeKind`]: the library, in the order of Compose's `MaterialShapes`.
//! - [`RoundedPolygon`]: any outline of vertices with optional rounding.
//! - [`Morph`]: an animation between any two polygons, matched corner to
//!   corner, so the outline neither twists nor needs to be star-shaped.
//! - [`MaterialShape`]: the element that paints a shape or a morph.
//!
//! # Sources
//!
//! The Figma kit's pages and md3.io define no shape library, so by the kit's
//! reference priority the source is the next that does: m3e, which ports
//! Compose Material3's `MaterialShapes.kt` at `19991f2c` and
//! `androidx.graphics:graphics-shapes:1.0.1`. This module ports the same files
//! from Google's source (Apache-2.0: see `LICENSE-androidx` in this directory
//! and each file's header). `tests/compose_shapes.rs` holds it to a dump those
//! unmodified sources produced on the JVM.
//!
//! Morphing defaults to [`MorphMatching::Current`], current `graphics-shapes`
//! rather than the pinned 1.0.1 ([`MorphMatching::Compose1_5`]): at the
//! progress values surveyed, 1.0.1's matching makes 13 of the 1190 ordered
//! pairs self-intersect, and current matching none. No reference dump exists
//! for it, so its outlines are held to geometric tests instead.
//!
//! # Example
//!
//! ```
//! use gpui_kit_material::shapes::{MaterialShape, ShapeKind};
//!
//! // The outline halfway from a circle to a heart. Morphs are built once per
//! // pair, so asking again costs nothing.
//! let morph = ShapeKind::Circle.morph_to(ShapeKind::Heart);
//! assert_eq!(morph.as_cubics(0.5).len(), morph.len());
//!
//! // An element that springs to whichever shape it is next given.
//! let _favorite = MaterialShape::new("favorite", ShapeKind::Heart)
//!     .label("Favorite")
//!     .size(96.)
//!     .animated(true);
//! ```
mod catalog;
mod element;
mod morph;
mod polygon;

pub(crate) use element::PathCache;

pub use catalog::ShapeKind;
pub use element::{MaterialShape, ShapeFrame, morph_path, polygon_path};
pub use morph::{Morph, MorphMatching};
pub use polygon::{CornerRounding, Cubic, RoundedPolygon};
