use crate::glam::*;

/// Camera3d bundle that holds both the `Projection` and `Camera`
#[derive(Default, Debug, Clone, Copy)]
pub struct Camera3dBundle {
    /// The `Camera3d` part of this bundle
    pub camera: Camera3d,
    /// The `Projection` part of this bundle
    pub projection: Projection,
}

/// A 3d Camera
#[derive(Debug, Default, Clone, Copy)]
pub struct Camera3d {
    /// The position of this `Camera3d`
    pub position: Vec3,
    /// The yaw or y axis rotation of this `Camera3d`
    pub yaw: f32,
    /// The pitch or x axis rotation of this `Camera3d`
    pub pitch: f32,
}

impl Camera3d {
    /// Create a new camera from the given position and rotation data
    pub fn new<V: Into<Vec3>>(position: V, yaw: f32, pitch: f32) -> Self {
        Self {
            position: position.into(),
            yaw,
            pitch,
        }
    }

    pub(crate) fn calc_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        glam::Mat4::look_to_rh(
            self.position,
            Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vec3::Y,
        )
    }
}

/// Projection for a camera
#[derive(Clone, Copy, Debug)]
pub struct Projection {
    /// The aspect ratio of the projection
    pub aspect: f32,
    /// The field of view for the projection
    pub fovy: f32,
    /// The near clipping plane for the projection
    pub znear: f32,
    /// The far clipping plane for the projection
    pub zfar: f32,
}

impl Default for Projection {
    fn default() -> Self {
        Self::new(1920, 1080, 70.0_f32.to_radians(), 0.1, 1000.0)
    }
}

impl Projection {
    /// Create a new `Projection` from the given parameters
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    /// Force a resize for this `Projection`
    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub(crate) fn calc_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraUniform {
    pub(crate) fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub(crate) fn update_view_proj(&mut self, camera_bundle: &Camera3dBundle) {
        let view = camera_bundle.projection.calc_matrix() * camera_bundle.camera.calc_matrix();
        self.view_proj = view.to_cols_array_2d();
    }
}
