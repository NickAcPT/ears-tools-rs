use nmsr_rendering::{
    high_level::{
        camera::CameraRotation,
        pipeline::{
            scene::Scene,
            GraphicsContext,
        },
    },
    low_level::{EulerRot, Quat, Vec3},
};

// Rotate our orbital camera based on the mouse
static mut MOUSE_DOWN: bool = false;
static mut LAST_X: Option<f32> = None;
static mut LAST_Y: Option<f32> = None;

pub fn rotate_camera(scene: &mut Scene, ctx: &GraphicsContext, yaw: f32, pitch: f32, roll: f32) {
    let camera = scene.camera_mut();
    let rotation = camera.get_rotation_as_mut();

    rotation.yaw = yaw;
    rotation.pitch = pitch;
    rotation.roll = roll;

    let one_eighty_diff = (rotation.yaw.abs() - 180.0).abs();
    let yaw = if one_eighty_diff < 0.01 {
        camera.get_yaw().abs() + 90.0
    } else if camera.get_yaw().is_sign_positive() || camera.get_yaw() <= -90.0 {
        camera.get_yaw()
    } else {
        camera.get_yaw() + 90.0
    };

    let aligned_yaw = ((yaw + 180.0) / 90.0).floor() * 90.0;

    let rot_quat: Quat = Quat::from_euler(
        EulerRot::ZXY,
        camera.get_roll().to_radians(),
        0.0,
        aligned_yaw.to_radians(),
    );

    let light = Vec3::new(0.0, -6.21, 6.21);
    let front_lighting = rot_quat.mul_vec3(light) * Vec3::new(1.0, 1.0, -1.0);

    scene.sun_information_mut().direction = front_lighting;

    scene.update(ctx);
}

pub fn handle_mouse_move(scene: &mut Scene, ctx: &GraphicsContext, x: f32, y: f32) {
    if unsafe { MOUSE_DOWN } {
        if let (Some(last_x), Some(last_y)) = unsafe { (LAST_X, LAST_Y) } {
            let x = x - last_x;
            let y = y - last_y;

            let camera = scene.camera_mut();
            let CameraRotation {
                mut yaw,
                mut pitch,
                roll,
            } = &camera.get_rotation();

            yaw += x;
            pitch += y;
            
            pitch = pitch.clamp(-89.9, 89.9);

            rotate_camera(scene, ctx, yaw, pitch, *roll);
        }
    }

    unsafe {
        LAST_X.replace(x);
        LAST_Y.replace(y);
    }
}

pub fn handle_mouse_down() {
    unsafe { MOUSE_DOWN = true };
}

pub fn handle_mouse_up() {
    unsafe {
        MOUSE_DOWN = false;
        LAST_X = None;
        LAST_Y = None;
    }
}

pub fn handle_mouse_scroll(scene: &mut Scene, ctx: &GraphicsContext, delta: f32) {
    let camera = scene.camera_mut();

    let dist = camera.get_distance_as_mut();

    if let Some(dist) = dist {
        *dist -= delta;
        
        *dist = dist.clamp(5.0, 90.0);

        scene.update(ctx);
    }
}
