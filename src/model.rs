use glam::Vec3;

use crate::camera::Camera;

pub trait Model {
    fn intersection(&self, origin: Vec3, ray: Vec3, camera: &Camera) -> Option<f32>;
    fn reflection(&self, ray: Vec3, point: Vec3, camera: &Camera) -> Option<Vec3>;
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub color: Vec3,
    pub emission_color: Vec3,
    pub emission: f32,
}

impl Sphere {
    pub fn new(
        center: Vec3,
        radius: f32,
        color: Vec3,
        emission_color: Vec3,
        emission: f32,
    ) -> Self {
        Self {
            center,
            radius,
            color,
            emission_color,
            emission,
        }
    }
}

impl Model for Sphere {
    fn intersection(&self, origin: Vec3, ray: Vec3, camera: &Camera) -> Option<f32> {
        let norm = ray.normalize();
        let q = origin - camera.convert(self.center);
        let a = norm.dot(norm);
        let b = 2.0 * q.dot(norm);
        let c = q.dot(q) - self.radius * self.radius;
        let d = b * b - 4.0 * a * c;

        if d < 0.0 {
            return None;
        }

        let distance = (-b - d.sqrt()) / (2.0 * a);
        if distance <= 0.0 {
            return None;
        }
        Some(distance)
    }

    fn reflection(&self, ray: Vec3, point: Vec3, camera: &Camera) -> Option<Vec3> {
        let c = camera.convert(self.center);
        let normal = (point - c).normalize();
        Some(ray - 2.0 * ray.dot(normal) * normal)
    }
}
