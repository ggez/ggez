//! The module provides an easy to use interface for implementing physics for your ggez game
//! and is only available with the `physics` feature flag for ggez.
//! ```toml
//! [dependencies]
//! ggez = { version = "0.6", features = ["physics", ...] }
//! ```
//!
//! This module uses `rapier2d` as its backend.

#![allow(missing_debug_implementations, missing_copy_implementations)]

use glam::Vec2;
use rapier2d::{
    dynamics::{RigidBody, RigidBodyHandle},
    geometry::Collider,
    na as rapier_na,
};

use rapier2d::dynamics::{IntegrationParameters, JointSet, RigidBodySet};
use rapier2d::geometry::{BroadPhase, ColliderSet, NarrowPhase};
use rapier2d::pipeline::PhysicsPipeline;

pub use rapier2d::*;

/// A body handle is a unique identity for the physics object
/// in the physics world.
pub type BodyHandle = RigidBodyHandle;

///
pub struct Physics {
    gravity: rapier_na::Vector2<f32>,
    integration_parameters: IntegrationParameters,
    pipeline: PhysicsPipeline,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    joints: JointSet,
}

impl Physics {
    /// Create a new physics world.
    pub fn new(gravity: Option<Vec2>) -> Self {
        let gravity = convert_vec_ggez_to_vec_rp(gravity.unwrap_or(Vec2::new(0.0, 300.0)));

        let integration_parameters = IntegrationParameters::default();
        let pipeline = PhysicsPipeline::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let bodies = RigidBodySet::new();
        let colliders = ColliderSet::new();
        let joints = JointSet::new();

        Self {
            gravity,
            integration_parameters,
            pipeline,
            broad_phase,
            narrow_phase,
            bodies,
            colliders,
            joints,
        }
    }

    /// Take a time step in our world!.
    /// **Note**: This function should be called in every update for the physics world to update.
    pub fn step(&mut self) {
        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            None,
            None,
            &(),
        );
    }

    /// Insert a new rigid body in the physics world.
    pub fn insert_body(&mut self, body: RigidBody, collider: Collider) -> BodyHandle {
        let body_handle = self.bodies.insert(body);

        let _ = self
            .colliders
            .insert(collider, body_handle, &mut self.bodies);

        body_handle
    }

    /// Get a immutable reference to a body in the world
    pub fn get_body(&mut self, handle: BodyHandle) -> Option<&RigidBody> {
        self.bodies.get(handle)
    }
}

/// Convert the ggez vector type to the rapier vector type.
fn convert_vec_ggez_to_vec_rp(ggez_vec: Vec2) -> rapier_na::Vector2<f32> {
    rapier_na::Vector2::new(ggez_vec.x, ggez_vec.y)
}
