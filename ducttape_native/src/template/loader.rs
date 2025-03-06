use std::{collections::HashMap, fs, io::Cursor, sync::Arc};

use ducttape_item_engine::{
    attribute::{Attribute, AttributeModifier, AttributeReason, AttributeType},
    item::{EngineHook, Item, SpecialAbility, Stats},
    prelude_items::stats::BasicStatsBuilder,
};
use godot::{
    classes::{Image, ImageTexture, ProjectSettings},
    prelude::*,
};
use hex_color::HexColor;
use image::{DynamicImage, GenericImage, GenericImageView as _};
use ndarray::Array2;
use serde::Deserialize;
use uuid::Uuid;

pub fn image_to_texture(image: DynamicImage) -> Option<Gd<ImageTexture>> {
    let mut img = Image::new_gd();

    let mut buf = Cursor::new(Vec::new());

    image
        .write_to(&mut buf, image::ImageFormat::Png)
        .expect("Failed to write image to buffer");

    img.load_png_from_buffer(&PackedByteArray::from(buf.into_inner()));

    ImageTexture::create_from_image(&img)
}

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
    pub data_name: String,
    pub attribute: HashMap<AttributeType, AttributeTypeEntry>,
    pub components: HashMap<String, HexColor>,
    pub fallback: HashMap<String, String>,
}

pub struct Mask(Array2<bool>);

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
        let settings = ProjectSettings::singleton();

        let base_path: String = settings.globalize_path(&template.folder).into();

        let mask_image = image::open(format!("{}/template.png", base_path))?;

        let image_path = format!("{}/{}:{}.png", base_path, component, item);
        let fallback_path: String = settings
            .globalize_path(template.fallback.get(component).ok_or("Missing Fallback")?)
            .into();

        let image = image::open(&image_path).or_else(|_| image::open(&fallback_path))?;

        let (width, height) = (image.width() as usize, image.height() as usize);

        let mask_data = mask_image.pixels().map(|(x, y, pixel)| {
            let color = HexColor::rgba(pixel[0], pixel[1], pixel[2], pixel[3]);
            let is_masked = template
                .components
                .get(component)
                .map_or(false, |c| c == &color);
            ((y as usize, x as usize), is_masked)
        });

        let mut mask_array = Array2::default((height, width));
        for ((y, x), is_masked) in mask_data {
            mask_array[[y, x]] = is_masked;
        }

        Ok(Self::new(Mask(mask_array), image))
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
        // Intentionally leaving this mut so a warning signifies i need to implement this
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
            let _ = MaskedImage::new_from_template(&self.template, component, &item.get_ident())
                .map(|masked_image| {
                    renderer.component_map.push(masked_image);
                });
        }

        Some(renderer.render())
    }
}

const ASSET_FOLDER: &str = "res://assets/item/"; // {template_name}/{component_name}:{item_name}.png

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

    pub fn load_template(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let folder = format!("{}/{}", ASSET_FOLDER, name);

        let template_path =
            ProjectSettings::singleton().globalize_path(&format!("{}/template.toml", folder));
        let template: ItemTemplateData =
            toml::from_str(&fs::read_to_string(template_path.to_string())?)?;

        let attribute_map = template
            .attribute
            .into_iter()
            .map(|(at, entry)| {
                let attributes = entry
                    .attr
                    .into_iter()
                    .map(|(id, attr)| Attribute {
                        uuid: id,
                        reason: attr.reason,
                        modifier: attr.modifier,
                        priority: attr.priority,
                    })
                    .collect();
                (at, attributes)
            })
            .collect();

        Ok(Self {
            fallback: template
                .fallback
                .into_iter()
                .map(|(k, v)| (k.clone(), format!("{}/{}:{}.png", folder, k, v)))
                .collect(),
            folder,
            data_name: template.data_name,
            attribute: attribute_map,
            components: template.components,
        })
    }
}
