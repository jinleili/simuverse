#[derive(Copy, Clone, Debug)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point3D { x, y, z }
    }

    pub fn zero() -> Self {
        Point3D::new(0.0, 0.0, 0.0)
    }

    pub fn is_equal_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }

    // 加减乘除运算
    pub fn add(&self, other: &Point3D) -> Self {
        Point3D::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn minus(&self, other: &Point3D) -> Self {
        Point3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn multiply_f(&self, param: f32) -> Self {
        Point3D::new(self.x * param, self.y * param, self.z * param)
    }

    pub fn divide_f(&self, param: f32) -> Self {
        Point3D::new(self.x / param, self.y / param, self.z / param)
    }

    pub fn offset(&self, dx: f32, dy: f32, dz: f32) -> Self {
        Point3D::new(self.x + dx, self.y + dy, self.z + dz)
    }

    // 模长
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

impl From<Point3D> for [f32; 3] {
    fn from(vs: Point3D) -> Self {
        [vs.x, vs.y, vs.z]
    }
}

impl From<[f32; 3]> for Point3D {
    fn from(vs: [f32; 3]) -> Self {
        Point3D::new(vs[0], vs[1], vs[2])
    }
}

impl From<&[f32; 3]> for Point3D {
    fn from(vs: &[f32; 3]) -> Self {
        Point3D::new(vs[0], vs[1], vs[2])
    }
}

impl From<glam::Vec3> for Point3D {
    fn from(vec3: glam::Vec3) -> Self {
        let vs: [f32; 3] = vec3.into();
        Point3D::new(vs[0], vs[1], vs[2])
    }
}
