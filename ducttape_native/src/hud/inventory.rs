use std::rc::Rc;

use ducttape_item_engine::{item::{Item, ItemCollection}, prelude_items::rock::Rock, text_renderer::{ansi_renderer::AnsiRenderer, bbcode_renderer::BBCodeRenderer}};
use godot::{classes::{Control, IControl, Label, VBoxContainer}, obj::NewAlloc, prelude::*};

#[derive(GodotClass)]
#[class(base = Control)]
pub struct Inventory {
    base: Base<Control>,
    items: ItemCollection,
    selected_item: i32,
}

#[godot_api]
impl IControl for Inventory {
    fn init(base: Base<Control>) -> Self {
        Self { base, items: ItemCollection::new(), selected_item: 0 }
    }

    fn ready(&mut self) {
        let rock = Rock::new();

        self.items.add_item(Box::new(rock));
        let mut base = self.base_mut();
        base.set_visible(true);
        base.set_size(Vector2::new(200.0, 200.0));
        // base.(Color::from_rgb(0.5, 0.5, 0.5));
        

        let mut vbox = VBoxContainer::new_alloc();
        vbox.set_size(Vector2::new(200.0, 200.0));
        vbox.set_modulate(Color::from_rgb(0.5, 0.5, 0.5));
        base.add_child(&vbox);

        drop(base);

        self.items.iter().for_each(|item| {
            let mut label = Label::new_alloc();
            label.set_text(&item.get_name().to_bbcode_string());
            vbox.add_child(&label);
           
        });


    }

    // fn input(&mut self, evt: Gd<InputEvent>) {
    //     if evt.is_action_pressed("ui_inventory") {
    //         self.base().set_visible(!self.base().is_visible());
    //     }
    // }
}