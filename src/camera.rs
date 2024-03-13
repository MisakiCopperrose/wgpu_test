pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    pub fn build_view_projection(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(
            f32::to_radians(self.fov_y),
            self.aspect,
            self.z_near,
            self.z_far,
        );

        proj * view
    }

    pub fn input_move_camera(&mut self, event: &winit::event::WindowEvent, speed: f32) -> bool {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: winit::event::ElementState::Pressed,
                        physical_key: keycode,
                        ..
                    },
                ..
            } => match keycode {
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyW) => {
                    self.eye += self.forward().normalize_or_zero() * speed;

                    std::println!("W: {}", self.eye);

                    true
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyS) => {
                    self.eye -= self.forward().normalize_or_zero() * speed;

                    std::println!("S: {}", self.eye);

                    true
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyA) => {
                    self.eye = self.target
                        - (self.forward() + self.right() * speed).normalize_or_zero()
                            * self.forward().length();

                    std::println!("A: {}", self.eye);

                    true
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyD) => {
                    self.eye = self.target
                        - (self.forward() - self.right() * speed).normalize_or_zero()
                            * self.forward().length();

                    std::println!("D: {}", self.eye);

                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn forward(&self) -> glam::Vec3 {
        self.target - self.eye
    }

    pub fn right(&self) -> glam::Vec3 {
        self.forward().normalize_or_zero().cross(self.up)
    }
}
