use std::rc::Rc;

use ducttape_item_engine::{item::{Item, ItemCollection}, prelude_items::rock::Rock, text_renderer::{ansi_renderer::AnsiRenderer, bbcode_renderer::BBCodeRenderer}};
use godot::{classes::{Control, IControl, Label, VBoxContainer}, obj::NewAlloc, prelude::*};

#[derive(GodotClass)]
#[class(base = Control)]
pub struct Console {
    base: Base<Control>,
    box_container: Gd<VBoxContainer>
}

#[godot_api]
impl IControl for Console {
    fn init(base: Base<Control>) -> Self {
        Self { base, box_container: VBoxContainer::new_alloc() }
    }

    fn ready(&mut self) {
        let mut base = self.base_mut();
        base.set_visible(true);
        base.set_size(Vector2::new(200.0, 200.0));
        drop(base);

        let mut vbox = self.box_container.clone();
        vbox.set_size(Vector2::new(200.0, 200.0));
        self.base_mut().add_child(&vbox);

        let mut label = Label::new_alloc();
        label.set_text("Hello, world!");
        vbox.add_child(&label);

        let mut label = Label::new_alloc();
        label.set_text("Hello, world!");
        vbox.add_child(&label);
    }
}

impl Console {
    fn add_line(&mut self, text: &str) {
        let mut label = Label::new_alloc();
        label.set_text(text);
        self.box_container.add_child(&label);
    }
}