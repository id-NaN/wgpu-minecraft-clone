pub trait Camera {
    fn get_view_projection(&self) -> glm::Mat4;
    fn set_position(&mut self, position: glm::Vec3);
    fn set_rotation(&mut self, rotation: na::UnitQuaternion<f32>);
    fn set_aspect_ratio(&mut self, aspect_ratio: f32);
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

fn calculate_view(
    rotation: na::UnitQuaternion<f32>,
    position: glm::Vec3,
) -> glm::Mat4 {
    glm::Mat4::from(rotation.to_rotation_matrix())
        .prepend_translation(&-position)
}

impl PerspectiveCamera {
    pub fn new(aspect_ratio: f32, near_plane: f32, far_plane: f32) -> Self {
        let mut camera = Self {
            projection: glm::identity(),
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: na::UnitQuaternion::identity(),
            aspect_ratio,
            fov_y: crate::SETTINGS.graphics.fov,
            near_plane,
            far_plane,
        };
        camera.recalculate_projection();
        camera
    }

    pub fn calculate_view(&self) -> glm::Mat4 {
        calculate_view(self.rotation, self.position)
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

    fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.recalculate_projection();
    }
}

pub struct OrthographicCamera {
    projection: glm::Mat4,
    position: glm::Vec3,
    rotation: na::UnitQuaternion<f32>,
    depth: f32,
    width: f32,
    aspect_ratio: f32,
}

impl OrthographicCamera {
    pub fn new(depth: f32, width: f32, aspect_ratio: f32) -> Self {
        let mut camera = Self {
            projection: glm::identity(),
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: na::UnitQuaternion::identity(),
            depth,
            width,
            aspect_ratio,
        };
        camera.recalculate_projection();
        camera
    }

    pub fn calculate_view(&self) -> glm::Mat4 {
        calculate_view(self.rotation, self.position)
    }

    fn recalculate_projection(&mut self) {
        let height = self.width / self.aspect_ratio;
        let projection = glm::ortho_lh_zo(
            -self.width / 2.0,
            self.width / 2.0,
            -height / 2.0,
            height / 2.0,
            0.0,
            self.depth,
        );
        self.projection = projection;
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

    fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }
}

pub struct ObliqueOrthographicCamera {
    position: glm::Vec3,
    rotation: na::UnitQuaternion<f32>,
    clipping_plane_normal: glm::Vec3,
    depth: f32,
    width: f32,
    aspect_ratio: f32,
}

impl ObliqueOrthographicCamera {
    pub fn new(
        depth: f32,
        width: f32,
        aspect_ratio: f32,
        clipping_plane_normal: glm::Vec3,
    ) -> Self {
        Self {
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: na::UnitQuaternion::identity(),
            clipping_plane_normal,
            depth,
            width,
            aspect_ratio,
        }
    }

    pub fn calculate_view(&self) -> glm::Mat4 {
        calculate_view(self.rotation, self.position)
    }

    fn calculate_projection(&self) -> glm::Mat4 {
        let height = self.width / self.aspect_ratio;
        let mut projection = glm::ortho_lh_zo(
            -self.width / 2.0,
            self.width / 2.0,
            -height / 2.0,
            height / 2.0,
            0.0,
            self.depth,
        );

        let clip_plane = glm::vec4(
            self.clipping_plane_normal.x,
            self.clipping_plane_normal.y,
            self.clipping_plane_normal.z,
            1.0,
        );
        let q = projection.try_inverse().unwrap()
            * glm::vec4(
                clip_plane.x.signum(),
                clip_plane.y.signum(),
                1.0,
                1.0,
            );
        let c = clip_plane * (2.0 * clip_plane.dot(&q));
        projection[2] = c.x - projection[3];
        projection[6] = c.y - projection[7];
        projection[10] = c.z - projection[11];
        projection[14] = c.w - projection[15];
        projection
    }
}

impl Camera for ObliqueOrthographicCamera {
    fn get_view_projection(&self) -> glm::Mat4 {
        self.calculate_projection() * self.calculate_view()
    }

    fn set_position(&mut self, position: glm::Vec3) {
        self.position = position;
    }

    fn set_rotation(&mut self, rotation: na::UnitQuaternion<f32>) {
        self.rotation = rotation;
    }

    fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }
}
