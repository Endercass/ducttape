use std::sync::{mpsc, Arc, Mutex};

use ducttape_item_engine::{item::{Item, ItemCollection, ItemCollectionEvent, ItemStack, ItemTexture}, prelude_items::{air::Air, MISSING_TEXTURE}};
use godot::{
    classes::{
        control::{MouseFilter, SizeFlags}, Control, GridContainer, HBoxContainer, IControl, IGridContainer, IPanel, InputEvent, InputEventMouseButton, InputEventMouseMotion, MarginContainer, Panel, ResourceLoader, RichTextLabel, ScrollContainer, StyleBoxFlat, Texture2D, TextureRect, VBoxContainer
    }, obj::NewAlloc, prelude::*
};

use crate::{singletons::inventory::INVENTORY, template::loader::{anim_to_texture, image_to_texture}};

#[derive(GodotClass)]
#[class(base = GridContainer)]
pub struct InventoryContainer {
    base: Base<GridContainer>,
    width: i32,
    active_item: i32,
    slots: Vec<Gd<InventoryItemSlot>>,
}

#[godot_api]
impl InventoryContainer {
    #[func]
    fn _on_change_active_state(&mut self, item: Gd<InventoryItem>, active: bool) {
        if let Some(slot) = item.get_parent().and_then(|parent| parent.try_cast::<InventoryItemSlot>().ok()) {
            if active {
                self.active_item = slot.bind().get_slot_index();
            } else {
                self.active_item = -1;
            }

            godot_print!("Active item: {:?}", self.active_item);
        }
    }

    #[signal]
    fn grid_resized(&self, width: i32);

    #[func]
    fn _on_grid_resized(&mut self, width: i32) {
        godot_print!("Grid resized to {} elements wide", width);
    }

    #[func]
    fn _on_item_drag_end(&mut self, item: Gd<InventoryItem>, pos: Vector2) {
        for slot in self.slots.iter_mut() {
            slot.emit_signal("item_slot_drag_end", &[item.to_variant(), pos.to_variant()]);
        }
    }

    #[func]
    pub fn set_width(&mut self, width: i32) {
        self.width = width;
        self.base_mut().set_columns(width);
        self.base_mut().emit_signal("grid_resized", &[Variant::from(width as i64)]);
    }

    #[func]
    pub fn get_width(&self) -> i32 {
        self.width
    }

    #[func]
    fn _on_request_rerender(&mut self) {
        let self_gd = self.to_gd();
        
        self.slots.iter_mut().enumerate().for_each(|(index, slot)| {
            let inventory = INVENTORY.lock().unwrap();
            // slot.set_item(InventoryItem::new(inventory.get_item(slot.get_slot_index() as usize).unwrap().clone()));
            // let item = inventory.get_item(index as usize).unwrap().clone();
            let item = inventory.get_item(index).cloned().unwrap_or_else(|_| Air::new_itemstack());
            let item_node = InventoryItem::new(item);
            
            slot.bind_mut().set_item(item_node.clone());

            item_node.clone().connect("item_drag_end", &Callable::from_object_method(
                &self_gd,
                "_on_item_drag_end"));

            item_node.clone().connect("change_active_state", &Callable::from_object_method(
                &self_gd,
                "_on_change_active_state"));

        });
    }

    #[signal]
    fn request_rerender(&self);
}

#[godot_api]
impl IGridContainer for InventoryContainer {
    fn init(base: Base<GridContainer>) -> Self {
        Self {
            base,
            width: 4,
            active_item: -1,
            slots: vec![],
        }
    }

    fn ready(&mut self) {
        let width = self.width;
        let inventory = INVENTORY.lock().unwrap();

        let self_gd = self.to_gd();
        let mut base = self.base_mut();

        base.connect("request_rerender", &Callable::from_object_method(&self_gd, "_on_request_rerender"));
        base.connect("grid_resized", &Callable::from_object_method(&self_gd, "_on_grid_resized"));

        let slots: Vec<Gd<InventoryItemSlot>> = inventory.iter().enumerate().map(|(index, item)| {
            let mut slot = InventoryItemSlot::new_alloc();
            slot.bind_mut().set_slot_index(index as i32);

            godot_print!("Setting item {:?}", item);

            base.add_child(&slot);

            let item_node = InventoryItem::new(item.clone());
            
            slot.bind_mut().set_item(
                item_node.clone()
            );

            item_node.clone().connect("item_drag_end", &Callable::from_object_method(
                &self_gd,
                "_on_item_drag_end"));
            slot
        }).collect();

        drop(base);

        let mut base = self.base_mut();
        base.set_columns(width);
        base.set_h_size_flags(SizeFlags::EXPAND_FILL);
        base.set_v_size_flags(SizeFlags::EXPAND_FILL);

        drop(base);

        self.slots = slots;
    }
}

#[derive(GodotClass)]
#[class(base = Control)]
pub struct InventoryItemSlot {
    base: Base<Control>,
    slot_index: i32,
    item: Option<Gd<InventoryItem>>,
}

#[godot_api]
impl InventoryItemSlot {
    #[func]
    pub fn set_item(&mut self, item: Gd<InventoryItem>) {
        self.item = Some(item.clone());
        godot_print!("IIS recv (func)");
        self.base_mut().emit_signal("item_changed", &[item.to_variant()]);
    }

    #[func]
    pub fn get_item(&self) -> Option<Gd<InventoryItem>> {
        self.item.clone()
    }

    #[func]
    pub fn set_slot_index(&mut self, index: i32) {
        self.slot_index = index;
    }

    #[func]
    pub fn get_slot_index(&self) -> i32 {
        self.slot_index
    }

    #[signal]
    fn item_changed(&self, item: Gd<InventoryItem>);

    #[func]
    fn _on_item_changed(&mut self, item: Gd<InventoryItem>) {
        godot_print!("IIS recv (signal)");

        godot_print!("Item changed: {:?}", item);
        let mut base = self.base_mut();
        base.get_children().iter_shared().for_each(|child| {
            base.remove_child(&child);
        });
        base.add_child(&item);
    }

    #[signal]
    fn item_slot_drag_end(&self, item: Gd<InventoryItem>, pos: Vector2);

    #[func]
    fn _on_item_slot_drag_end(&mut self, item: Gd<InventoryItem>, pos: Vector2) {
        if self.base().get_global_rect().contains_point(pos) {
            // We are the destination
            if item != self.item.clone().unwrap() {
                if let Ok(item) = item.get_parent().unwrap().try_cast::<InventoryItemSlot>() {
                    let old_slot_number = item.bind().get_slot_index();
                    godot_print!("Old item slot number: {:?}", old_slot_number);
                    let new_slot_number = self.slot_index;
                    godot_print!("New item slot number: {:?}", new_slot_number);

                    let mut inventory = INVENTORY.lock().unwrap();
                    let items = inventory.get_items_mut();

                    let (left, right) = if old_slot_number < new_slot_number {
                        items.split_at_mut(new_slot_number as usize)
                    } else {
                        items.split_at_mut(old_slot_number as usize)
                    };
                    std::mem::swap(&mut left[old_slot_number.min(new_slot_number) as usize], &mut right[0]);
                    
                    inventory.notify(ItemCollectionEvent::ManualRefresh);
                    drop(inventory);
                }
            }
        }
    }
}

#[godot_api]
impl IControl for InventoryItemSlot {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            slot_index: 0,
            item: None,
        }
    }

    fn ready(&mut self) {
        let self_gd = self.to_gd();
        let mut base = self.base_mut();
        base.set_custom_minimum_size(Vector2::new(32.0, 32.0));

        base.connect("item_slot_drag_end", &Callable::from_object_method(&self_gd, "_on_item_slot_drag_end"));
        base.connect("item_changed", &Callable::from_object_method(&self_gd, "_on_item_changed"));
    }
}

#[derive(Default)]
pub struct GuiItemState {
    pub dragging: bool,
    pub mouse_down: bool,
}


#[derive(GodotClass)]
#[class(base = Control, no_init)]
pub struct InventoryItem {
    base: Base<Control>,
    item: ItemStack,
    texture_rect: Option<Gd<TextureRect>>,
    state: Arc<Mutex<GuiItemState>>,
}

#[godot_api]
impl InventoryItem {
    pub fn new(item: ItemStack) -> Gd<Self> {
        Gd::from_init_fn(|base| {
            Self {
                base,
                item,
                texture_rect: None,
                state: Default::default(),
            }
        })
    }

    #[signal]
    fn change_active_state(&self, active: bool);

    #[func]
    fn _on_change_active_state(&mut self, _: Gd<Object>, active: bool) {
        godot_print!("Changing active state to {:?}", active);
        self.set_item_highlight(active);
    }

    #[signal]
    fn item_drag_end(&self, pos: Vector2);

    #[signal]
    fn request_rerender(&self);

    #[func]
    fn set_item_highlight(&mut self, highlight: bool) {
        let color = if highlight {
            Color::from_rgba(1.0, 1.0, 1.0, 0.5)
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 1.0)
        };

        self.base_mut().set_modulate(color);
    }

    #[func]
    fn _on_mouse_entered(&mut self) {
        self.set_item_highlight(true);
    }

    #[func]
    fn _on_mouse_exited(&mut self) {
        self.set_item_highlight(false);
    }

    fn handle_drop(&mut self, pos: Vector2) {
        godot_print!("dropping @ {:?}", pos);

        let self_gd = self.to_gd();

        self.base_mut().emit_signal("item_drag_end", &[self_gd.to_variant(), pos.to_variant()]);

        self.base_mut().set_position(Vector2::new(0.0, 0.0));

        // Rerender the inventory manually if we arent updating the inventory so that the item isnt floating around
        // self.request_rerender();

        // let item = self.item.clone();


    }

    fn handle_drag(&mut self, evt: Gd<InputEventMouseMotion>) {
        let pos = self.base().get_position() + evt.get_relative();

        self.base_mut().set_position(pos);
    }

    fn handle_click(&mut self, evt: Gd<InputEventMouseButton>) {
        let self_gd = self.to_gd();

        self.base_mut().emit_signal("change_active_state", &[self_gd.to_variant(), evt.is_pressed().to_variant()]);

        let mut state = self.state.lock().unwrap();
        
        state.mouse_down = evt.is_pressed();
        
        if state.mouse_down {
            state.dragging = true;
            drop(state);
        } else if state.dragging {
            state.dragging = false;
            drop(state);
            self.handle_drop(self.base().get_global_position() + evt.get_position());
        }
    }

    #[func]
    fn _on_gui_input(&mut self, evt: Gd<InputEvent>) {
        if let Ok(evt) = evt.clone().try_cast::<InputEventMouseButton>() {
            self.handle_click(evt);
        }

        if let Ok(evt) = evt.try_cast::<InputEventMouseMotion>() {
            if self.state.lock().unwrap().dragging {
                self.handle_drag(evt);
            }
        }

    }

    fn is_textured(&self) -> bool {
        self.texture_rect.is_some()
    }
}

#[godot_api]
impl IControl for InventoryItem {
    fn ready(&mut self) {
        let item = self.item.clone();
        let self_gd = self.to_gd();
        let mut base = self.base_mut();
        base.set_custom_minimum_size(Vector2::new(32.0, 32.0));
        base.set_h_size_flags(SizeFlags::EXPAND_FILL);
        base.set_v_size_flags(SizeFlags::EXPAND_FILL);
        base.set_mouse_filter(MouseFilter::STOP);

        if item.get_ident() == "air" {
            base.set_mouse_filter(MouseFilter::PASS);
            return;
        }

        base.connect("mouse_entered", &Callable::from_object_method(&self_gd, "_on_mouse_entered"));
        base.connect("mouse_exited", &Callable::from_object_method(&self_gd, "_on_mouse_exited"));
        base.connect("gui_input", &Callable::from_object_method(&self_gd, "_on_gui_input"));
        base.connect("change_active_state", &Callable::from_object_method(&self_gd, "_on_change_active_state"));
    }

    fn process(&mut self, _delta: f64) {
        let item = self.item.clone();

        if item.get_ident() == "air" {
            return;
        }

        if !self.is_textured() {
            let mut texture_rect = TextureRect::new_alloc();
            texture_rect.set_custom_minimum_size(Vector2::new(32.0, 32.0));
            texture_rect.set_h_size_flags(SizeFlags::EXPAND_FILL);
            texture_rect.set_v_size_flags(SizeFlags::EXPAND_FILL);

            match item.get_texture() {
                ItemTexture::None => {
                    texture_rect.set_texture(&image_to_texture(MISSING_TEXTURE.clone()).unwrap());
                },
                ItemTexture::Image(img) => {
                    texture_rect.set_texture(&image_to_texture(img).unwrap());
                }
                ItemTexture::Animated(atlas, frame_properties, animation_type) => {
                    let (texture, timer) = anim_to_texture(atlas, frame_properties, animation_type).unwrap();
                    
                    texture_rect.set_texture(&texture);
                    self.base_mut().add_child(&timer);
                }
            };
    
            self.base_mut().add_child(&texture_rect.clone());
            self.texture_rect = Some(texture_rect.clone());
        }
        
    }
}



#[derive(GodotClass)]
#[class(base = Panel)]
pub struct Inventory {
    base: Base<Panel>,
    grid: Option<Gd<InventoryContainer>>,
    attrs: Option<Gd<VBoxContainer>>,
    stats: Option<Gd<RichTextLabel>>,
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
        // if self.grid.is_none() {
        //     return;
        // }

        if let Some(grid) = self.grid.as_mut() {
            grid.emit_signal("request_rerender", &[]);
        }




        
    //     let inventory = INVENTORY.lock().unwrap();
    //     let grid = self.grid.as_mut().unwrap();
    //     let attrs = self.attrs.as_mut().unwrap();
    //     let stats = self.stats.as_mut().unwrap().clone();
        
    //     grid.get_children().iter_shared().for_each(|child| {
    //         grid.remove_child(&child);
    //     });
        
    //     let pos = grid.get_global_position();

    //     let mut slot_position_field: Vec<Vector2> = vec![];

    //     inventory.iter().for_each(|item| {
    //         // More performant to store the stats in a cache when the inventory is updated rather than
    //         // re-parsing the attributes every time the mouse enters the item slot
    //         let mut statbox_cache: Vec<(AttributeType, Text)> = AttributeParser::from(item.get_stats().get_all_attributes()).aggregate_to_components().iter().map(|(attr, text)| (attr.clone(), text.clone())).collect();
    //         statbox_cache.sort_by(|(a, _), (b, _)| a.cmp(b));

    //         let mouse_down_on_item = Arc::new(Mutex::new(false));
    //         let dragging = Arc::new(Mutex::new(false));
    //         let start_pos = Arc::new(Mutex::new(Vector2::new(0.0, 0.0)));
    //         let last_pos = Arc::new(Mutex::new(Vector2::new(0.0, 0.0)));

    //         // // let mut attrs_cache: Vec<AttributeType> = item.get_stats().get_all_attributes().keys().cloned().collect();
    //         // // attrs_cache.sort();
    //         // let mut el = TextureRect::new_alloc();
    //         // el.set_size(Vector2::new(32.0, 32.0));

    //         // match item.get_texture() {
    //         //     ItemTexture::None => {
    //         //         el.set_texture(&image_to_texture(MISSING_TEXTURE.clone()).unwrap());
    //         //     },
    //         //     ItemTexture::Image(img) => {
    //         //         el.set_texture(&image_to_texture(img).unwrap());
    //         //     }
    //         //     ItemTexture::Animated(atlas, frame_properties, animation_type) => {
    //         //         let (texture, timer) = anim_to_texture(atlas, frame_properties, animation_type).unwrap();
                    
    //         //         el.set_texture(&texture);
    //         //         el.add_child(&timer);
    //         //     }

    //         // }

    //         // el.clone().connect("mouse_entered", &Callable::from_local_fn("_on_mouse_entered", {
    //         //     let mut stats = stats.clone();  
    //         //     let mut el = el.clone();
    //         //     let mut attrs = attrs.clone();
    //         //     move |_| {
    //         //         el.set_modulate(Color::from_rgb(0.75, 0.75, 0.75));
    //         //         attrs.get_children().iter_shared().for_each(|child| {
    //         //             attrs.remove_child(&child);
    //         //         });
                    
    //         //         stats.set_text("");

    //         //         statbox_cache.iter().for_each(|(attr, value)| {
    //         //             let mut label = Label::new_alloc();

    //         //             label.set_custom_minimum_size(Vector2::new(32.0, 32.0));
    //         //             label.set_text(&attr.into_text().to_bbcode_string());
    //         //             label.set_mouse_filter(MouseFilter::STOP);
    //         //             label.set_h_size_flags(SizeFlags::EXPAND_FILL);
    //         //             label.set_v_size_flags(SizeFlags::SHRINK_CENTER);
    //         //             label.set_vertical_alignment(VerticalAlignment::CENTER);
    //         //             label.set_horizontal_alignment(HorizontalAlignment::CENTER);


    //         //             label.clone().connect("mouse_entered", &Callable::from_local_fn("_on_mouse_entered", {
    //         //                 let mut label = label.clone();
    //         //                 let mut stats = stats.clone();
    //         //                 let value = value.clone();
    //         //                 move |_| {
    //         //                     label.set_modulate(Color::from_rgb(0.75, 0.75, 0.75));
    //         //                     stats.set_text(&value.to_bbcode_string());
    //         //                     godot_print!("Stats go here: {:?}", value.to_bbcode_string());
    //         //                     Ok(Variant::nil())
    //         //                 }
    //         //             }));

    //         //             label.clone().connect("mouse_exited", &Callable::from_local_fn("_on_mouse_exited", {
    //         //                 let mut label = label.clone();
    //         //                 let mut stats = stats.clone();
    //         //                 move |_| {
    //         //                     label.set_modulate(Color::from_rgb(1.0, 1.0, 1.0));
    //         //                     stats.set_text("");
    //         //                     Ok(Variant::nil())
    //         //                 }
    //         //             }));

    //         //             attrs.add_child(&label);
    //         //         });
    //         //         Ok(Variant::nil())
    //         //     }
    //         // }));

    //         // el.clone().connect("mouse_exited", &Callable::from_local_fn("_on_mouse_exited", {
    //         //     let mut el = el.clone();
    //         //     let mouse_down_on_item = mouse_down_on_item.clone();
    //         //     move |_| {

    //         //         if *mouse_down_on_item.lock().unwrap() {
    //         //             // mouse moving faster than the item, don't trigger the mouse exit event
    //         //             return Ok(Variant::nil());
    //         //         }   
    //         //         el.set_modulate(Color::from_rgb(1.0, 1.0, 1.0));
    //         //         Ok(Variant::nil())
    //         //     }
    //         // }));

    //         // el.clone().connect("gui_input", &Callable::from_local_fn("_on_mouse_button", {
    //         //     let item = item.clone();
    //         //     let mut el = el.clone();
    //         //     let mouse_down_on_item = mouse_down_on_item.clone();
    //         //     let dragging = dragging.clone();
    //         //     let slot_position_field = slot_position_field.clone();
    //         //     move |v| {
    //         //         if let Ok(evt) = v[0].try_to::<Gd<InputEventMouseButton>>() {
    //         //             *mouse_down_on_item.lock().unwrap() = evt.is_pressed();
    //         //             *start_pos.lock().unwrap() = evt.get_position();

    //         //             if (*dragging.lock().unwrap() && !*mouse_down_on_item.lock().unwrap()) {
    //         //                 // stop dragging
    //         //                 *dragging.lock().unwrap() = false;
    //         //                 let closest_slot = slot_position_field.iter().enumerate().min_by(|(_, a), (_, b)| {
    //         //                     let a = a.distance_to(evt.get_position());
    //         //                     let b = b.distance_to(evt.get_position());
    //         //                     a.partial_cmp(&b).unwrap()
    //         //                 }).unwrap().0;

    //         //                 godot_print!("Closest slot: {:?}", closest_slot);
    //         //             }
    //         //         }

    //         //         if *mouse_down_on_item.lock().unwrap() || *dragging.lock().unwrap() {
    //         //             if let Ok(evt) = v[0].try_to::<Gd<InputEventMouseMotion>>() {
    //         //                 if !evt.get_relative().is_zero_approx() {
    //         //                     *dragging.lock().unwrap() = true;
    //         //                     let viewport = el.get_viewport().unwrap();
    //         //                     let mouse_pos = viewport.get_mouse_position() - pos - el.get_rect().size / 2.0;

    //         //                     godot_print!("Mouse pos {:?}", mouse_pos);
    //         //                     godot_print!("Start pos {:?}", start_pos.lock().unwrap());
    //         //                     godot_print!("Relative pos {:?}", evt.get_relative());

    //         //                     *last_pos.lock().unwrap() = mouse_pos;
    //         //                     el.set_position(mouse_pos);
    //         //                 }
    //         //             }
    //         //         }

    //         //         //                            let drop_pos = evt.get_position();

    //         //         // let closest_slot = slot_position_field.iter().enumerate().min_by(|(_, a), (_, b)| {
    //         //         //     // let a = a.distance_to(drop_pos);
    //         //         //     // let b = b.distance_to(drop_pos);
    //         //         //     let a = a.distance_squared_to(drop_pos); // slightly faster
    //         //         //     let b = b.distance_squared_to(drop_pos);
    //         //         //     a.partial_cmp(&b).unwrap()
    //         //         // }).unwrap().0;

    //         //         Ok(Variant::nil())
    //         //     }
    //         // }));

    //         let mut el = InventoryItem::new(item.clone());

    //         el.connect("request_rerender", &Callable::from_local_fn("_on_request_rerender", {
    //             move |_| {
    //                 let mut inventory = INVENTORY.lock().unwrap();
    //                 inventory.refresh();
    //                 Ok(Variant::nil())
    //             }
    //         }));
    //         // el.set_item(item.clone());

    //         grid.add_child(&el);

    //         slot_position_field.push(el.get_canvas_transform().affine_inverse() * el.get_position());
    // });
    }
}

#[godot_api]
impl IPanel for Inventory {
    fn init(base: Base<Panel>) -> Self {
        Self {
            base,
            grid: None,
            attrs: None,
            stats: None,
            receiver: None,
        }
    }

    fn ready(&mut self) {
        let mut base = self.base_mut();
        base.set_visible(false);
        base.center_anchor(Vector2::new(320.0, 176.0));
        // make the background color transparent
        let mut style_box = StyleBoxFlat::new_gd();
        style_box.set_bg_color(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
        base.add_theme_stylebox_override("panel", &style_box);

        let gui_texture = ResourceLoader::singleton()
            .load("res://assets/gui/inventory.png")
            .unwrap()
            .cast::<Texture2D>();

        let mut gui = TextureRect::new_alloc();

        gui.set_texture(&gui_texture);
        gui.center_anchor(Vector2::new(320.0, 176.0));
        base.add_child(&gui);

        let mut content_box = HBoxContainer::new_alloc();
        content_box.set_anchor_and_offset(Side::TOP, 0.0, 16.0);
        content_box.set_anchor_and_offset(Side::LEFT, 0.0, 16.0);
        content_box.set_anchor_and_offset(Side::RIGHT, 1.0, -16.0);
        content_box.set_anchor_and_offset(Side::BOTTOM, 1.0, -16.0);
        content_box.add_theme_constant_override("separation", 0);

        base.add_child(&content_box);

        let mut grid_margin = MarginContainer::new_alloc();
        grid_margin.add_theme_constant_override("margin_top", 2);
        grid_margin.add_theme_constant_override("margin_left", 2);
        grid_margin.add_theme_constant_override("margin_right", 2);
        grid_margin.add_theme_constant_override("margin_bottom", 2);

        content_box.add_child(&grid_margin);

        let grid = InventoryContainer::new_alloc();

        // let mut grid = GridContainer::new_alloc();

        // grid.set_columns(4);
        // grid.add_theme_constant_override("v_separation", 4);
        // grid.add_theme_constant_override("h_separation", 4);

        grid_margin.add_child(&grid);

        let mut attr_margin = MarginContainer::new_alloc();
        attr_margin.add_theme_constant_override("margin_top", 2);
        attr_margin.add_theme_constant_override("margin_left", 2);
        attr_margin.add_theme_constant_override("margin_right", 2);
        attr_margin.add_theme_constant_override("margin_bottom", 2);
        attr_margin.set_custom_minimum_size(Vector2::new(36.0, 144.0));

        content_box.add_child(&attr_margin);

        let mut attr_scroll = ScrollContainer::new_alloc();
        attr_scroll.set_h_size_flags(SizeFlags::EXPAND_FILL);
        attr_scroll.set_v_size_flags(SizeFlags::EXPAND_FILL);

        // attr_scroll.get_v_scroll_bar().unwrap().set_modulate(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
        let mut scrollbar = attr_scroll.get_v_scroll_bar().unwrap();
        scrollbar.set_modulate(Color::from_rgba(0.0, 0.0, 0.0, 0.5));
        scrollbar.set_custom_minimum_size(Vector2::new(2.0, 0.0));
        scrollbar.set_anchor_and_offset(Side::TOP, 0.0, 0.0);
        scrollbar.set_anchor_and_offset(Side::BOTTOM, 1.0, 0.0);
        scrollbar.set_anchor_and_offset(Side::RIGHT, 1.0, 0.0);
        scrollbar.set_anchor_and_offset(Side::LEFT, 1.0, -2.0);
        
        attr_margin.add_child(&attr_scroll);

        let mut attr_box = VBoxContainer::new_alloc();
        attr_box.set_h_size_flags(SizeFlags::EXPAND_FILL);
        attr_box.set_v_size_flags(SizeFlags::EXPAND_FILL);
        attr_box.add_theme_constant_override("separation", 4);

        attr_scroll.add_child(&attr_box);

        let mut stats_margin = MarginContainer::new_alloc();
        stats_margin.add_theme_constant_override("margin_top", 2);
        stats_margin.add_theme_constant_override("margin_left", 2);
        stats_margin.add_theme_constant_override("margin_right", 2);
        stats_margin.add_theme_constant_override("margin_bottom", 2);
        stats_margin.set_custom_minimum_size(Vector2::new(108.0, 144.0));

        content_box.add_child(&stats_margin);
        
        let mut stats = RichTextLabel::new_alloc(); 

        stats.set_h_size_flags(SizeFlags::EXPAND_FILL);
        stats.set_v_size_flags(SizeFlags::EXPAND_FILL);

        stats.set_use_bbcode(true);
        // stats.set_text("St/ats go here");
        stats.set_text("[color=#ff0000]Stats go here[/color]");
        godot_print_rich!("[color=#ff0000]Stats go here[/color]");    

        // gui.add_child(&stats);
        stats_margin.add_child(&stats);

        drop(base);

        self.grid = Some(grid);
        self.stats = Some(stats);
        self.attrs = Some(attr_box);
        self.render();

        let (sender, receiver) = std::sync::mpsc::channel();

        let mut inventory = INVENTORY.lock().unwrap();

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
