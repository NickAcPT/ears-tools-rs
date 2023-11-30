mod model;
pub mod camera;

pub mod high_level {
    pub mod camera {
        pub use nmsr_rasterizer_test::camera::*;
    }
    pub mod pipeline {
        pub mod scene {
            pub use crate::nmsr_rendering::model::Scene;
            pub use nmsr_rasterizer_test::{model::Size, shader::SunInformation};
        }
    }
    pub mod parts {
        pub use nmsr_player_parts::parts::*;
    }
}

pub mod low_level {
    pub use glam::{Mat4, Vec3, Vec3A, EulerRot, Quat};
}
