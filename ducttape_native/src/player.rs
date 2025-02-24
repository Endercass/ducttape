use godot::{
    classes::{CharacterBody2D, ICharacterBody2D, InputEvent},
    prelude::*,
};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
struct Player {
    base: Base<CharacterBody2D>,
    #[export]
    speed: f32,
    #[export]
    gravity: f32,
    #[export]
    jump_height: f32,
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            speed: 100.0,
            gravity: 200.0,
            jump_height: -150.0,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let gravity = self.gravity;
        let speed = self.speed;
        let mut base = self.base_mut();

        let input = Input::singleton();
        let mut velocity = base.get_velocity();

        velocity.y += gravity * delta as f32;
        velocity.x = (input.get_action_strength("move_right")
            - input.get_action_strength("move_left"))
            * speed
            * if input.is_action_pressed("move_sprint") {
                2.0
            } else {
                1.0
            };

        base.set_velocity(velocity);

        base.move_and_slide();
    }

    fn input(&mut self, evt: Gd<InputEvent>) {
        let jump_height = self.jump_height;
        let mut base = self.base_mut();
        if evt.is_action_pressed("move_jump") && base.is_on_floor() {
            let velocity = base.get_velocity();

            base.set_velocity(Vector2::new(velocity.x, jump_height));
        }
    }
}

// impl Component for Player {
//    const STORAGE_TYPE: bevy::ecs::component::StorageType = bevy::ecs::component::StorageType::Table;
// }

// pub fn spawn(commands: &mut Commands) -> Gd<Player> {
//     let player = Player::new_alloc();
//     commands.spawn(()).insert(player);
//     player
// }
