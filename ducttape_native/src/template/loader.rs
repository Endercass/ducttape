use std::{collections::HashMap, io::Cursor, ops::Deref, sync::Arc};

use ducttape_item_engine::{
    attribute::{Attribute, AttributeModifier, AttributeReason, AttributeType},
    item::{EngineHook, Item, SpecialAbility, Stats},
    prelude_items::stats::{BasicStats, BasicStatsBuilder},
};
use dyn_clone::DynClone;
use godot::{
    classes::{
        class_macros::private::callbacks::create, image::Format, IImageFormatLoaderExtension,
        IImageTexture, ITexture2D, Image, ImageTexture, ProjectSettings, Texture2D,
    },
    prelude::*,
};
use hex_color::HexColor;
use image::{DynamicImage, GenericImage, GenericImageView as _};
use ndarray::Array2;
use serde::Deserialize;
use uuid::Uuid;

pub const TEMPLATE_EXAMPLE: &str = r##"
# The folder where the asset is located
# this is relative to the template.toml file, so it is usually "."
# from this folder we will load template.png, and then will load each
# `${component}:${item}.png` file
# where: component is the key in the components table, and item is the identifier of the item 
asset_folder = "."
# The data name of the item, for usage in the internal item registry
# this is distinct from the item identifier, because compound items like this one
# will have multiple instances depending on the components, but they will all share the same data_name
data_name = "spear"

[attribute.Sharpness]
strategy = "Sum"

# 5 is a reasonable sharpness for a spear, as its not the sharpest weapon due to the tip shape
# but it is still mainly used for piercing, so it should be sharp enough
[attribute.Sharpness.attr.'9aedbcff-7ea9-4228-b7c8-df371092d901']
# Priority of the attribute, acts as the order of attributes in the item tooltip
# and as the order of attributes in the calculation of the item's stats
# base attributes of the item, as seen here, will come before the component
# item attributes
priority = 1

# Optionally add a reason (emoji item name) for the attribute. If this is not present,
# the attribute will be interpreted as a
# `Hidden` enum variant where the stat is not displayed in the item tooltip
reason = "🍢"

modifier = { "Set" = 5 }

[attribute.Durability]
strategy = "Sum"

# Spears are fairly durable but not exceptionally so, so we will set the durability of the spear to 100
[attribute.Durability.attr.'bd04a445-be04-416d-a775-ebd61db4587c']
priority = 1
reason = "🍢"
modifier = { "Set" = 100 }

[attribute.Weight]
strategy = "Sum"

# Spears are generally lightweight, so we will set the weight of the spear to 5
[attribute.Weight.attr.'59ebf731-12be-4e93-92a3-f29e38d2a5f7']
priority = 1
reason = "🍢"
modifier = { "Set" = 5 }

[attribute.Strength]
strategy = "Average"

# The strength attribute is what controls how much weight the item can carry,
# which in the context of a spear is how much force it can withstand before breaking.
# When the spear is used for mobility, i.e. bridging across gaps, the strength attribute will 
# judge how much durability the spear will lose when used in this way.
# We will subtract the weight of the player from the strength of the spear and then
# subtract the result from the durability of the spear to get the new durability of the spear
# after the player has used it for bridging.
[attribute.Strength.attr.'ef312d65-31bb-46b3-8fde-5098ac305146']
priority = 1
reason = "🍢"
modifier = { "Set" = 50 }

[attribute.Agility]
strategy = "Average"

# Since spears are used for mobility, agility is an important attribute for them
# We will set the agility of the spear to 10
[attribute.Agility.attr.'e36abc22-bc45-4763-abad-2c712d8e3878']
priority = 1
reason = "🍢"
modifier = { "Set" = 10 }

[attribute.Reach]
strategy = "Sum"

# The reach of the spear is an important attribute, as it determines how far the player can reach
# The spear is a weapon that can either be used for melee or ranged combat, so it should have a decent reach
[attribute.Reach.attr.'17f33751-2499-4928-9fcf-1d0143f443c2']
priority = 1
reason = "🍢"
modifier = { "Set" = 10 }

[components]
# Each component here will specify a color from the template.png mask file that will correspond to the component of the item
# The key is the name of the component, and the value is the color in the mask file
tip = "#ff0000"
shaft = "#00ff00"

[fallback]
# The stick texture is a reasonable fallback for the shaft component
tip = "stick" # tip:stick.png
# Any fallback for the tip component isn't ideal, as it is the most important part of the item visually and functionally
# but stone will do i guess
shaft = "stone" # shaft:stone.png

"##;

pub fn image_to_texture(image: DynamicImage) -> Option<Gd<ImageTexture>> {
    let mut img = Image::new_gd();

    let mut buf = Cursor::new(Vec::new());

    image
        .write_to(&mut buf, image::ImageFormat::Png)
        .expect("Failed to write image to buffer");

    img.load_png_from_buffer(&PackedByteArray::from(buf.into_inner()));

    // let mut data = Vec::new();
    // for pixel in image.pixels() {
    //     data.push(pixel[0]);
    //     data.push(pixel[1]);
    //     data.push(pixel[2]);
    //     data.push(pixel[3]);
    // }
    // let image = Image::create_from_data(
    //     image.width() as i32,
    //     image.height() as i32,
    //     false,
    //     Format::RGBA8,
    //     &PackedByteArray::from(data),
    // )?;

    ImageTexture::create_from_image(&img)
}

// #[derive(GodotClass)]
// #[class(no_init, tool, base=ImageTexture)]
// pub struct DynamicImageTexture2D {
//     base: Base<ImageTexture>,
//     texture: Gd<ImageTexture>,
// }

// impl DynamicImageTexture2D {
//     pub fn from_image(dynamic_image: DynamicImage) -> Option<Gd<Self>> {
//         let dynamic_image = dynamic_image.to_rgba8();
//         let mut data = Vec::new();
//         for pixel in dynamic_image.pixels() {
//             data.push(pixel[0]);
//             data.push(pixel[1]);
//             data.push(pixel[2]);
//             data.push(pixel[3]);
//         }
//         let image = Image::create_from_data(
//             dynamic_image.width() as i32,
//             dynamic_image.height() as i32,
//             false,
//             Format::RGBA8,
//             &PackedByteArray::from(data),
//         )?;

//         let texture = ImageTexture::create_from_image(&image)?;

//         Some(Gd::from_init_fn(|base| Self { base, texture }))
//     }
// }

// #[godot_api]
// impl IImageTexture for DynamicImageTexture2D {
//     fn get_width(&self) -> i32 {
//         self.texture.get_width() as i32
//     }

//     fn get_height(&self) -> i32 {
//         self.texture.get_height() as i32
//     }

//     // fn draw_rect(
//     //     &self,
//     //     to_canvas_item: Rid,
//     //     rect: Rect2,
//     //     tile: bool,
//     //     modulate: Color,
//     //     transpose: bool,
//     // ) {
//     //     self.texture
//     //         .draw_rect_ex(to_canvas_item, rect, tile)
//     //         .modulate(modulate)
//     //         .transpose(transpose)
//     //         .done();
//     // }

//     // fn draw_rect_region(
//     //     &self,
//     //     to_canvas_item: Rid,
//     //     rect: Rect2,
//     //     src_rect: Rect2,
//     //     modulate: Color,
//     //     transpose: bool,
//     //     clip_uv: bool,
//     // ) {
//     //     self.texture
//     //         .draw_rect_region_ex(to_canvas_item, rect, src_rect)
//     //         .modulate(modulate)
//     //         .transpose(transpose)
//     //         .clip_uv(clip_uv)
//     //         .done();
//     // }
// }

#[derive(Debug, Deserialize)]
pub struct SerializableAttribute {
    pub priority: u8,
    pub reason: AttributeReason,
    pub modifier: AttributeModifier,
}

#[derive(Debug, Deserialize)]
pub enum AttributeStrategy {
    // Sum the attributes of each of the components
    Sum,
    // Take the mean of the attributes of each of the components
    Average,
    /// Push all the component attributes to the item handler's attribute handler method
    Manual,
}

#[derive(Debug, Deserialize)]
pub struct AttributeTypeEntry {
    pub strategy: AttributeStrategy,
    pub attr: HashMap<Uuid, SerializableAttribute>,
}

#[derive(Debug, Deserialize)]
pub struct ItemTemplateData {
    pub asset_folder: String,
    pub data_name: String,
    pub attribute: HashMap<AttributeType, AttributeTypeEntry>,
    pub components: HashMap<String, HexColor>,
    pub fallback: HashMap<String, String>,
}

pub struct Mask(Array2<bool>);

// impl FromIterator<(u32, u32, bool)> for Mask {
//     fn from_iter<T: IntoIterator<Item = (u32, u32, bool)>>(iter: T) -> Self {
//         let mut max_x = 0;
//         let mut max_y = 0;
//         let mut mask = Array2::from_elem((1, 1), false);
//         for (x, y, is_masked) in iter {
//             if x >= max_x {
//                 max_x = x + 1;
//             }
//             if y >= max_y {
//                 max_y = y + 1;
//             }
//             mask = Array2::from_shape_vec(
//                 (max_y as usize, max_x as usize),
//                 mask.iter().cloned().collect(),
//             )
//             .unwrap_or_else(|_| Array2::from_elem((1, 1), false));
//             mask[[y as usize, x as usize]] = is_masked;
//         }
//         Self(mask)
//     }
// }

pub struct MaskedImage {
    // The mask is a 2D array of booleans, where true means the pixel is part of the component
    // mask: Vec<Vec<bool>>,
    mask: Mask,

    // The source image of the component
    image: image::DynamicImage,
}

impl MaskedImage {
    pub fn new(mask: Mask, image: image::DynamicImage) -> Self {
        Self { mask, image }
    }

    pub fn new_from_template(
        template: &ItemTemplate,
        component: &str,
        item: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let project_settings = ProjectSettings::singleton();

        let basepath = project_settings.globalize_path("res://assets/item/spear");

        godot_print!("Basepath: {:?}", basepath);

        godot_print!("Loading image: {}/{}:{}.png", basepath, component, item);

        let mask_image = image::open(format!("{}/template.png", basepath))?;

        let image = image::open(format!("{}/{}:{}.png", basepath, component, item))?;

        let width = image.width();
        let height = image.height();

        let mask: Mask = mask_image
            .pixels()
            .map(|(x, y, pixel)| {
                let color = HexColor::rgba(pixel[0], pixel[1], pixel[2], pixel[3]);

                (
                    x,
                    y,
                    template
                        .components
                        .get(component)
                        .map(|c| c == &color)
                        .unwrap_or(false),
                )
            })
            .fold(
                Mask(Array2::default((height as usize, width as usize))),
                |mask, (x, y, is_masked)| {
                    let mut mask = mask;
                    mask.0[[y as usize, x as usize]] = is_masked;
                    mask
                },
            );

        // .collect::<Mask>();
        Ok(Self::new(mask, image))
    }

    pub fn get_mask(&self) -> &Mask {
        &self.mask
    }

    pub fn get_image(&self) -> &image::DynamicImage {
        &self.image
    }

    pub fn get_masked_image(&self) -> image::DynamicImage {
        let mut masked_image = self.image.clone();

        let mask = &self.mask.0;

        for (y, row) in mask.outer_iter().enumerate() {
            for (x, is_masked) in row.iter().enumerate() {
                if !is_masked {
                    masked_image.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 0]));
                }
            }
        }

        masked_image
    }
}

pub struct TemplateComponentRenderer {
    size: (u32, u32),
    component_map: Vec<MaskedImage>,
}

impl TemplateComponentRenderer {
    pub fn new(size: (u32, u32)) -> Self {
        Self {
            component_map: Vec::new(),
            size,
        }
    }

    pub fn render(&self) -> image::DynamicImage {
        let mut image = image::DynamicImage::new_rgba8(self.size.0, self.size.1);

        // Set a background color
        for x in 0..image.width() {
            for y in 0..image.height() {
                image.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
            }
        }

        godot_print!("Image size: {:?}", image.dimensions());

        for masked_image in &self.component_map {
            let mask = masked_image.get_mask();
            let img = masked_image.get_image();

            for (y, row) in mask.0.outer_iter().enumerate() {
                for (x, is_masked) in row.iter().enumerate() {
                    if *is_masked {
                        let pixel = img.get_pixel(x as u32, y as u32);
                        image.put_pixel(x as u32, y as u32, pixel);
                    }
                }
            }

            // let img = masked_image.get_masked_image();
            // godot_print!("Masked image: {:?}", masked_image.dimensions());
            // // image.copy_from(&masked_image, 0, 0).unwrap();
            // // cant use copy_from because the transparent pixels just overwrite the background
            // for x in 0..masked_image.width() {
            //     for y in 0..masked_image.height() {
            //         if (masked_image.m
            //         let pixel = masked_image.get_pixel(x, y);
            //         if pixel[3] != 0 {
            //             image.put_pixel(x, y, pixel);
            //         }
            //     }
            // }
        }

        image
    }
}

pub trait TemplateHandler {
    /// Handle the attributes of each of the component items
    fn attribute_handler(
        &self,
        base_attributes: HashMap<AttributeType, Vec<Attribute>>,
        components: HashMap<String, HashMap<AttributeType, Vec<Attribute>>>,
    ) -> HashMap<AttributeType, Vec<Attribute>> {
        let mut attributes = base_attributes;

        for (_, component_attributes) in components {
            for (at, component_attribute) in component_attributes {
                attributes
                    .entry(at)
                    .or_default()
                    .extend(component_attribute.into_iter());
            }
        }

        attributes
    }
}

#[derive(Debug, Clone)]
pub struct TemplateItem<THook: EngineHook> {
    template: ItemTemplate,
    components: HashMap<String, Arc<dyn Item<THook>>>,
    special_abilities: Vec<Box<dyn SpecialAbility<THook>>>,
}

impl<THook: EngineHook> TemplateItem<THook> {
    pub fn new(template: ItemTemplate) -> Self {
        Self {
            template,
            components: HashMap::new(),
            special_abilities: Vec::new(),
        }
    }

    pub fn add_component(&mut self, part: String, component: Arc<dyn Item<THook>>) {
        for (at, attributes) in component.get_stats().get_all_attributes() {
            for attribute in attributes {
                self.template
                    .attribute
                    .entry(at)
                    .or_default()
                    .push(attribute);
            }
        }
        self.components.insert(part, component);
    }
}

impl<THook: EngineHook> Item<THook> for TemplateItem<THook> {
    fn get_name(&self) -> String {
        self.template.data_name.clone()
    }

    fn get_ident(&self) -> String {
        self.template.data_name.clone()
    }

    fn get_stats(&self) -> Box<dyn Stats> {
        let mut stats = BasicStatsBuilder::new();
        // Implement this later
        Box::new(stats.build())
    }

    fn get_special_abilities(&self) -> Vec<&Box<dyn SpecialAbility<THook>>> {
        self.special_abilities.iter().collect()
    }

    fn special_abilities(&self) -> Vec<Box<dyn SpecialAbility<THook>>> {
        self.special_abilities.clone()
    }

    fn get_texture(&self) -> Option<image::DynamicImage> {
        let mut renderer = TemplateComponentRenderer::new((32, 32));

        for (component, item) in &self.components {
            let masked_image =
                MaskedImage::new_from_template(&self.template, component, &item.get_ident())
                    .unwrap();
            renderer.component_map.push(masked_image);
        }

        Some(renderer.render())
    }
}

#[derive(Debug, Clone)]
pub struct ItemTemplate {
    folder: String,
    data_name: String,
    attribute: HashMap<AttributeType, Vec<Attribute>>,
    components: HashMap<String, HexColor>,
    fallback: HashMap<String, String>,
}

impl ItemTemplate {
    pub fn populate_template<THook: EngineHook>(
        &self,
        components: HashMap<String, Arc<dyn Item<THook>>>,
    ) -> TemplateItem<THook> {
        let mut item: TemplateItem<THook> = TemplateItem::new(self.clone());

        for (component, component_item) in components {
            item.add_component(component, component_item);
        }

        item
    }

    pub fn load_template(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let template: ItemTemplateData = toml::from_str(TEMPLATE_EXAMPLE)?;

        let attribute_map: HashMap<AttributeType, Vec<Attribute>> = template
            .attribute
            .iter()
            .map(|(at, entry)| {
                let attributes = entry
                    .attr
                    .iter()
                    .map(|(id, attr)| Attribute {
                        uuid: *id,
                        reason: attr.reason.clone(),
                        modifier: attr.modifier.clone(),
                        priority: attr.priority,
                    })
                    .collect();
                (*at, attributes)
            })
            .collect();

        // let components = template

        Ok(Self {
            folder: template.asset_folder,
            data_name: template.data_name,
            attribute: attribute_map,
            components: template.components,
            fallback: template.fallback,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_template() {
        // let template: ItemTemplate = toml::from_str(TEMPLATE_EXAMPLE).unwrap();
        let template = ItemTemplate::load_template("template.toml").unwrap();

        println!("{:#?}", template);
    }
}
