use std::sync::mpsc;

use ducttape_item_engine::{
    item::{Item as _, ItemCollection, ItemCollectionEvent},
    text_renderer::bbcode_renderer::BBCodeRenderer,
};
use godot::{
    classes::{
        control::SizeFlags, Control, GridContainer, IPanel, Image, ImageTexture, InputEvent, Label,
        Panel, ResourceLoader, Texture2D, TextureRect,
    },
    obj::NewAlloc,
    prelude::*,
};

use crate::{singletons::inventory, template::loader::image_to_texture};

#[derive(GodotClass)]
#[class(base = Panel)]
pub struct Inventory {
    base: Base<Panel>,
    grid: Option<Gd<GridContainer>>,
    receiver: Option<mpsc::Receiver<ItemCollectionEvent>>,
}

trait CenterAnchor {
    fn center_anchor(&mut self, size: Vector2);
}

impl CenterAnchor for Control {
    fn center_anchor(&mut self, size: Vector2) {
        self.set_anchor(Side::TOP, 0.5);
        self.set_anchor(Side::LEFT, 0.5);
        self.set_anchor(Side::RIGHT, 0.5);
        self.set_anchor(Side::BOTTOM, 0.5);
        self.set_offset(Side::TOP, -size.y / 2.0);
        self.set_offset(Side::LEFT, -size.x / 2.0);
        self.set_offset(Side::RIGHT, size.x / 2.0);
        self.set_offset(Side::BOTTOM, size.y / 2.0);
    }
}

impl Inventory {
    fn render(&mut self) {
        if self.grid.is_none() {
            return;
        }

        let inventory = inventory::INVENTORY.lock().unwrap();
        let grid = self.grid.as_mut().unwrap();

        grid.get_children().iter_shared().for_each(|child| {
            grid.remove_child(&child);
        });

        let slot_texture = ResourceLoader::singleton()
            .load_ex("res://inventory slot.png")
            .type_hint("Texture")
            .done()
            .unwrap()
            .cast::<Texture2D>();

        inventory.iter().for_each(|item| {
            let mut slot_bg = TextureRect::new_alloc();
            slot_bg.set_custom_minimum_size(Vector2::new(48.0, 48.0));
            slot_bg.set_texture(&slot_texture);

            match item.get_texture().and_then(|img| image_to_texture(img)) {
                Some(texture) => {
                    println!("Item texture: {:?}", texture);
                    let mut icon = TextureRect::new_alloc();
                    icon.set_texture(&texture);
                    icon.center_anchor(Vector2::new(32.0, 32.0));
                    slot_bg.add_child(&icon);
                }
                None => {
                    println!("Item texture: {}", item.get_name());

                    let mut label = Label::new_alloc();
                    label.set_text(&item.get_name());
                    label.add_theme_font_size_override("font_size", 24);
                    label.center_anchor(Vector2::new(24.0, 24.0));
                    slot_bg.add_child(&label);
                }
            }
            grid.add_child(&slot_bg);
        });
    }
}

#[godot_api]
impl IPanel for Inventory {
    fn init(base: Base<Panel>) -> Self {
        Self {
            base,
            grid: None,
            receiver: None,
        }
    }

    fn ready(&mut self) {
        let mut base = self.base_mut();
        base.set_visible(false);
        base.center_anchor(Vector2::new(240.0, 240.0));

        let mut grid = GridContainer::new_alloc();
        grid.set_h_size_flags(SizeFlags::EXPAND_FILL);
        grid.set_v_size_flags(SizeFlags::EXPAND_FILL);
        grid.set_columns(4);
        grid.set_size(Vector2::new(200.0, 200.0));
        grid.center_anchor(Vector2::new(200.0, 200.0));
        // grid.set_custom_minimum_size(Vector2::new(480.0, 270.0));

        // grid.set_modulate(Color::from_rgb(0.5, 0.5, 0.5));
        base.add_child(&grid);

        drop(base);

        self.grid = Some(grid);
        self.render();

        let (sender, receiver) = std::sync::mpsc::channel();

        let mut inventory = inventory::INVENTORY.lock().unwrap();

        inventory.listen(Box::new(move |event| {
            godot_print!("listener closure {:?}", event);
            sender.send(event).unwrap();
        }));

        drop(inventory);

        self.receiver = Some(receiver);
    }

    fn process(&mut self, _delta: f64) {
        if let Some(receiver) = self.receiver.take() {
            // Temporarily take ownership of the receiver
            for event in receiver.try_iter() {
                self.render(); // Now we can mutably borrow `self`
                godot_print!("process method {:?}", event);
            }
            self.receiver = Some(receiver); // Put the receiver back
        }
    }

    fn input(&mut self, evt: Gd<InputEvent>) {
        if evt.is_action_pressed("ui_inventory") {
            let mut base = self.base_mut();

            let visible = base.is_visible();
            base.set_visible(!visible);
        }
    }
}
