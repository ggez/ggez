use crate::glam::*;

/// Camera3d bundle that holds both the `Projection` and `Camera`
#[derive(Default, Debug, Clone, Copy)]
pub struct Camera3d {
    /// The `Camera3d` part of this bundle
    pub transform: Camera3dTransform,
    /// The `Projection` part of this bundle
    pub projection: Projection,
}

impl Camera3d {
    /// Calculate the matrix for your camera
    pub fn calc_matrix(&self) -> mint::ColumnMatrix4<f32> {
        (self.projection.calc_matrix() * self.transform.calc_matrix()).into()
    }
}

/// A 3d Camera
#[derive(Debug, Clone, Copy)]
pub struct Camera3dTransform {
    /// The position of this `Camera3d`
    pub position: mint::Point3<f32>,
    /// The yaw or y axis rotation of this `Camera3d`
    pub yaw: f32,
    /// The pitch or x axis rotation of this `Camera3d`
    pub pitch: f32,
}

impl Default for Camera3dTransform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO.into(),
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl Camera3dTransform {
    /// Create a new camera from the given position and rotation data
    pub fn new<V: Into<mint::Point3<f32>>>(position: V, yaw: f32, pitch: f32) -> Self {
        Self {
            position: position.into(),
            yaw,
            pitch,
        }
    }

    /// Change the position of the `Camera3d`
    pub fn position<P>(&mut self, position_: P) -> &mut Self
    where
        P: Into<mint::Point3<f32>>,
    {
        let p: mint::Point3<f32> = position_.into();
        self.position = p;
        self
    }

    /// Move the position by given amount
    pub fn translate<T>(&mut self, translate_: T) -> &mut Self
    where
        T: Into<mint::Vector3<f32>>,
    {
        let t: mint::Vector3<f32> = translate_.into();
        self.position(glam::Vec3::from(self.position) + glam::Vec3::from(t))
    }

    pub(crate) fn calc_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        glam::Mat4::look_to_rh(
            self.position.into(),
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
