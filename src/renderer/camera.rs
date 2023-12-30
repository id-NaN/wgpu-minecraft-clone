pub trait Camera {
    fn get_view_projection(&self) -> glm::Mat4;
    fn set_position(&mut self, position: glm::Vec3);
    fn set_rotation(&mut self, rotation: na::UnitQuaternion<f32>);
}

pub struct PerspectiveCamera {
    projection: glm::Mat4,
    position: glm::Vec3,
    rotation: na::UnitQuaternion<f32>,
    aspect_ratio: f32,
    fov_y: f32,
    near_plane: f32,
    far_plane: f32,
}

impl PerspectiveCamera {
    pub fn new(
        width: f32,
        height: f32,
        near_plane: f32,
        far_plane: f32,
    ) -> Self {
        let mut camera = Self {
            projection: glm::identity(),
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: na::UnitQuaternion::identity(),
            aspect_ratio: width / height,
            fov_y: crate::SETTINGS.graphics.fov,
            near_plane,
            far_plane,
        };
        camera.recalculate_projection();
        camera
    }

    pub fn calculate_view(&self) -> glm::Mat4 {
        glm::Mat4::from(self.rotation.to_rotation_matrix())
            .append_translation(&self.position)
            .try_inverse()
            .unwrap()
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.aspect_ratio = width / height;
        self.recalculate_projection();
    }

    fn recalculate_projection(&mut self) {
        self.projection = glm::perspective_lh_zo(
            self.aspect_ratio,
            self.fov_y,
            self.near_plane,
            self.far_plane,
        );
    }
}

impl Camera for PerspectiveCamera {
    fn get_view_projection(&self) -> glm::Mat4 {
        self.projection * self.calculate_view()
    }

    fn set_position(&mut self, position: glm::Vec3) {
        self.position = position;
    }

    fn set_rotation(&mut self, rotation: na::UnitQuaternion<f32>) {
        self.rotation = rotation;
    }
}

pub struct OrthographicCamera {
    projection: glm::Mat4,
    position: glm::Vec3,
    rotation: na::UnitQuaternion<f32>,
    depth: f32,
    width: f32,
    height: f32,
}

impl OrthographicCamera {
    pub fn new(depth: f32, width: f32, height: f32) -> Self {
        let mut camera = Self {
            projection: glm::identity(),
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: na::UnitQuaternion::identity(),
            depth,
            width,
            height,
        };
        camera.recalculate_projection();
        camera
    }

    pub fn calculate_view(&self) -> glm::Mat4 {
        glm::Mat4::from(self.rotation.to_rotation_matrix())
            .append_translation(&self.position)
            .try_inverse()
            .unwrap()
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.recalculate_projection();
    }

    fn recalculate_projection(&mut self) {
        self.projection = glm::ortho_lh_zo(
            -self.width / 2.0,
            self.width / 2.0,
            -self.height / 2.0,
            self.height / 2.0,
            0.0,
            self.depth,
        )
    }
}

impl Camera for OrthographicCamera {
    fn get_view_projection(&self) -> glm::Mat4 {
        self.projection * self.calculate_view()
    }

    fn set_position(&mut self, position: glm::Vec3) {
        self.position = position;
    }

    fn set_rotation(&mut self, rotation: na::UnitQuaternion<f32>) {
        self.rotation = rotation;
    }
}
