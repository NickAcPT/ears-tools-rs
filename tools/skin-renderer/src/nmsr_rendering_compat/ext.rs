use nmsr_rasterizer_test::camera::CameraRotation;

pub trait NmsrRasterizationTestCameraExt {
    fn set_rotation(&mut self, rotation: CameraRotation);
}

impl NmsrRasterizationTestCameraExt for nmsr_rasterizer_test::camera::Camera {
    fn set_rotation(&mut self, rotation: CameraRotation) {
        *self.get_rotation_mut() = rotation;
    }
}