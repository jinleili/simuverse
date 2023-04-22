use super::*;

impl Light {
    #[inline(always)]
    pub(super) fn light_info(&self) -> LightInfo {
        LightInfo {
            light_position: self.position.to_homogeneous().cast().unwrap().into(),
            light_color: self.color.cast().unwrap().extend(1.0).into(),
            light_type: [self.light_type.into(), 0, 0, 0],
        }
    }
}

impl Default for Light {
    #[inline(always)]
    fn default() -> Light {
        Light {
            position: Point3::origin(),
            color: Vector3::new(1.0, 1.0, 1.0),
            light_type: LightType::Point,
        }
    }
}

impl From<LightType> for usize {
    #[inline(always)]
    fn from(light_type: LightType) -> usize {
        match light_type {
            LightType::Point => 0,
            LightType::Uniform => 1,
        }
    }
}

impl From<LightType> for u32 {
    #[inline(always)]
    fn from(light_type: LightType) -> u32 {
        match light_type {
            LightType::Point => 0,
            LightType::Uniform => 1,
        }
    }
}
