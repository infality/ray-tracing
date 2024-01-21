use glam::{Mat4, Vec3, Vec4};

pub struct Camera {
    pub position: Vec3,
    pub direction: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub reset: bool,
}

impl Camera {
    pub fn new(position: Vec3, look_at: Vec3) -> Self {
        let direction = (position - look_at).normalize();
        let right = Vec3::new(0.0, 1.0, 0.0).cross(direction).normalize();
        let up = direction.cross(right);
        Camera {
            position,
            direction,
            right,
            up,
            yaw: -90.0,
            pitch: 0.0,
            reset: true,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from_cols(
            Vec4::new(self.right.x, self.right.y, self.right.z, -self.position.x),
            Vec4::new(self.up.x, self.up.y, self.up.z, -self.position.y),
            Vec4::new(
                self.direction.x,
                self.direction.y,
                self.direction.z,
                -self.position.z,
            ),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn convert(&self, vec: Vec3) -> Vec3 {
        let result = self.view_matrix()
            * Vec4::new(
                self.position.x - vec.x,
                self.position.y - vec.y,
                self.position.z - vec.z,
                1.0,
            );
        Vec3::new(result.x, result.y, result.z)
    }
}
