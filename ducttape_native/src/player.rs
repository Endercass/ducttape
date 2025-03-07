use godot::{
    classes::{CharacterBody2D, ICharacterBody2D},
    global::clampf,
    prelude::*,
};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
    base: Base<CharacterBody2D>,
    #[export]
    jump_height: f32,
    #[export]
    acceleration: f32,
    #[export]
    max_speed: f32,
    #[export]
    sprint_multiplier: f32,
    #[export]
    friction: f32,
    #[export]
    air_resistance: f32,
    #[export]
    gravity: f32,
    #[export]
    jump_force: f32,
    #[export]
    jump_cancel_force: f32,
    #[export]
    wall_slide_speed: f32,
    #[export]
    jump_buffer_timer: f64,
    #[export]
    wall_jump_timer: f32,

    state: PlayerState,
}

#[derive(Debug)]
pub struct PlayerState {
    pub sprinting: bool,
    pub can_jump: bool,
    pub should_jump: bool,
    pub wall_jump: bool,
    pub jumping: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            sprinting: false,
            can_jump: true,
            should_jump: false,
            wall_jump: false,
            jumping: false,
        }
    }
}

impl Player {
    fn get_input_direction(&self) -> Vector2 {
        let input = Input::singleton();
        let x_dir =
            input.get_action_strength("move_right") - input.get_action_strength("move_left");
        let y_dir = input.get_action_strength("move_down") - input.get_action_strength("move_up");

        Vector2::new(x_dir, y_dir)
    }

    fn jump(&mut self, direction: Vector2) {
        let mut velocity = self.base().get_velocity();
        let state = &mut self.state;

        state.can_jump = false;
        state.should_jump = false;
        state.jumping = true;

        if state.wall_jump {
            velocity.x += self.jump_force * -direction.x;
            state.wall_jump = false;
            velocity.y = 0.0;
        }

        velocity.y = -self.jump_force;

        self.base_mut().set_velocity(velocity);
    }

    fn buffer_jump(&mut self) {
        self.state.should_jump = true;
    }

    fn cancel_jump(&mut self, delta: f64) {
        self.state.jumping = false;
        let mut velocity = self.base().get_velocity();
        velocity.y -= self.jump_cancel_force * velocity.y.sign() * delta as f32;
        self.base_mut().set_velocity(velocity);
    }

    fn handle_jump(&mut self, delta: f64) {
        let velocity = self.base().get_velocity();

        let inputs = Input::singleton();

        if self.state.should_jump {
            self.state.should_jump = false;
        }

        let jump_strength = inputs.get_action_strength("move_jump");
        let jump_pressed = inputs.is_action_just_pressed("move_jump");

        if self.base().is_on_floor() {
            self.state.can_jump = true;
            self.state.wall_jump = false;
            self.state.jumping = false;
        }

        if (jump_pressed || self.state.should_jump) && self.state.can_jump {
            // self.get_input_direction();
            self.jump(self.get_input_direction());
        } else if jump_pressed {
            self.buffer_jump();
        } else if jump_strength == 0.0 && velocity.y < 0.0 {
            self.cancel_jump(delta);
        } else if !self.base().is_on_floor() && self.base().is_on_wall_only() {
            self.state.can_jump = true;
            self.state.wall_jump = true;
            self.state.jumping = false;
        }
    }

    fn handle_sprint(&mut self) {
        self.state.sprinting = Input::singleton().get_action_strength("move_sprint") > 0.0;
    }

    fn handle_velocity(&mut self, delta: f64) {
        let direction = self.get_input_direction();

        if direction.x != 0.0 {
            self.apply_velocity(delta, direction);
        } else {
            self.apply_friction(delta);
        }
    }

    fn apply_velocity(&mut self, delta: f64, direction: Vector2) {
        let sprint_multiplier = if self.state.sprinting {
            self.sprint_multiplier
        } else {
            1.0
        };

        let mut velocity = self.base().get_velocity();

        velocity.x += direction.x
            * self.acceleration
            * delta as f32
            * (if self.base().is_on_floor() {
                sprint_multiplier
            } else {
                1.0
            });
        velocity.x = velocity
            .x
            .max(-self.max_speed * direction.x.abs() * sprint_multiplier)
            .min(self.max_speed * direction.x.abs() * sprint_multiplier);

        self.base_mut().set_velocity(velocity);
    }

    fn apply_friction(&mut self, delta: f64) {
        let mut velocity = self.base().get_velocity();
        let friction = if self.base().is_on_floor() {
            self.friction
        } else {
            self.air_resistance
        } * delta as f32
            * -velocity.x.signum();

        if velocity.x.abs() <= friction.abs() {
            velocity.x = 0.0;
        } else {
            velocity.x += friction;
        }

        self.base_mut().set_velocity(velocity);
    }

    fn handle_gravity(&mut self, delta: f64) {
        let mut velocity = self.base().get_velocity();

        velocity.y += self.gravity * delta as f32;

        if !self.state.jumping
            && self.base().is_on_wall_only()
            && self.get_input_direction().x != 0.0
        {
            velocity.y = clampf(velocity.y as f64, 0.0, self.wall_slide_speed as f64) as f32;
        }

        self.base_mut().set_velocity(velocity);
    }
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            acceleration: 500.0,
            max_speed: 100.0,
            sprint_multiplier: 1.5,
            friction: 500.0,
            air_resistance: 200.0,
            gravity: 500.0,
            jump_force: 200.0,
            jump_cancel_force: 800.0,
            wall_slide_speed: 50.0,
            jump_buffer_timer: 0.1,
            wall_jump_timer: 0.1,
            jump_height: 100.0,
            state: PlayerState::default(),
        }
    }

    fn ready(&mut self) {
        // Add a camera to the player
        let cam = Camera2D::new_alloc();
        self.base_mut().add_child(&cam);
    }

    fn physics_process(&mut self, delta: f64) {
        self.handle_jump(delta);
        self.handle_sprint();
        self.handle_velocity(delta);

        self.handle_gravity(delta);

        self.base_mut().move_and_slide();
    }
}
