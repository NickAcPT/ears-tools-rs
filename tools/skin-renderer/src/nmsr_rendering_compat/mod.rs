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
        pub type GraphicsContext = ();
        pub type SceneContextWrapper = ();
        
        #[derive(Default)]
        pub struct SceneContext();
        
        impl SceneContext {
            pub fn new<T>(_ctx: T) -> Self {
                Self {}
            }
        }
    }
    pub mod parts {
        pub use nmsr_player_parts::parts::*;
    }
}

pub mod low_level {
    pub use glam::{Vec3, EulerRot, Quat};
}
