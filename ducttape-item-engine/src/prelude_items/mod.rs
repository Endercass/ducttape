pub mod air;
pub mod rock;
pub mod stats;
pub mod dev_tablet;

lazy_static::lazy_static! {
    pub static ref MISSING_TEXTURE: image::DynamicImage = image::open(asset_to_absolute("missing.png")).unwrap();
    pub static ref ROCK_TEXTURE: crate::item::ItemTexture = image::open(asset_to_absolute("rock.png")).ok().into();
    pub static ref AIR_TEXTURE: crate::item::ItemTexture = image::open(asset_to_absolute("air.png")).ok().into();
    pub static ref DEV_TABLET_TEXTURE: crate::item::ItemTexture = image::open(asset_to_absolute("tablet.png")).map_or(crate::item::ItemTexture::None, |atlas| (atlas, crate::item::FrameProperties {width: 32, height: 32, duration: 0.5}, crate::item::AnimationType::Loop).into());
}

fn asset_to_absolute(asset_relative_path: &str) -> String {
    format!("{}/assets/{}", env!("CARGO_MANIFEST_DIR"), asset_relative_path)
}

#[cfg(test)]
mod tests {
    use valence_text::IntoText;

    use crate::{
        attribute::{
            Attribute, AttributeModifier, AttributeParser, AttributeReason, AttributeType,
        },
        item::{DummyHook, EngineHook, Item, ItemMut, ItemTexture, SpecialAbility, Stats},
        text_renderer::ansi_renderer::AnsiRenderer,
    };
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    pub struct Ball<THook: EngineHook = DummyHook> {
        stats: BallStats,
        phantom: std::marker::PhantomData<THook>,
    }

    impl<THook: EngineHook> Ball<THook> {
        pub fn new() -> Self {
            Ball {
                stats: BallStats::default(),
                phantom: std::marker::PhantomData,
            }
        }
    }

    impl<THook: EngineHook> Item<THook> for Ball<THook> {
        fn get_name(&self) -> String {
            "⚽".into()
        }

        fn get_ident(&self) -> String {
            "ball".into()
        }

        fn get_stats(&self) -> Box<dyn Stats> {
            Box::new(self.stats.clone())
        }

        fn special_abilities(&self) -> Vec<Box<dyn SpecialAbility<THook>>> {
            Vec::new()
        }

        fn get_texture(&self) -> ItemTexture {
            None.into()
        }
    }

    impl<THook: EngineHook> ItemMut for Ball<THook> {
        fn get_stats_mut(&mut self) -> &mut dyn Stats {
            &mut self.stats
        }
    }

    #[derive(Debug, Clone)]
    pub struct BallStats {
        attributes: HashMap<AttributeType, Vec<Attribute>>,
    }

    impl Default for BallStats {
        fn default() -> Self {
            let mut attributes = HashMap::new();
            attributes.insert(
                AttributeType::Durability,
                vec![
                    Attribute {
                        uuid: uuid::Uuid::new_v4(),
                        reason: AttributeReason::Display("⚽".to_string()),
                        priority: 0,
                        modifier: AttributeModifier::Set(100.0),
                    },
                    Attribute {
                        uuid: uuid::Uuid::new_v4(),
                        reason: AttributeReason::Display("✖️".to_string()),
                        priority: 4,
                        modifier: AttributeModifier::Multiply(1.5),
                    },
                    Attribute {
                        uuid: uuid::Uuid::new_v4(),
                        reason: AttributeReason::Display("➕".to_string()),
                        priority: 2,
                        modifier: AttributeModifier::Add(10.0),
                    },
                    Attribute {
                        uuid: uuid::Uuid::new_v4(),
                        reason: AttributeReason::Display("➗".to_string()),
                        priority: 6,
                        modifier: AttributeModifier::Multiply(0.75),
                    },
                    Attribute {
                        uuid: uuid::Uuid::new_v4(),
                        reason: AttributeReason::Display("➖".to_string()),
                        priority: 8,
                        modifier: AttributeModifier::Add(-10.0),
                    },
                ],
            );

            BallStats { attributes }
        }
    }

    impl Stats for BallStats {
        fn get_attribute(&self, at: AttributeType, id: uuid::Uuid) -> Attribute {
            self.attributes
                .get(&at)
                .unwrap()
                .iter()
                .find(|a| a.uuid == id)
                .unwrap()
                .clone()
        }

        fn get_attributes(&self, at: AttributeType) -> Vec<Attribute> {
            self.attributes.get(&at).unwrap().clone()
        }

        fn get_all_attributes(&self) -> HashMap<AttributeType, Vec<Attribute>> {
            self.attributes.clone()
        }

        fn push_attribute(&mut self, at: AttributeType, attribute: Attribute) {
            self.attributes.entry(at).or_default().push(attribute);
        }

        fn push_attributes(&mut self, attributes: HashMap<AttributeType, Attribute>) {
            for (at, attribute) in attributes {
                self.push_attribute(at, attribute);
            }
        }

        fn set_attribute(&mut self, at: AttributeType, id: uuid::Uuid, attribute: Attribute) {
            let vec = self.attributes.entry(at).or_default();

            if let Some(index) = vec.iter().position(|a| a.uuid == id) {
                vec[index] = attribute;
            }
        }

        fn set_attributes(&mut self, at: AttributeType, attributes: Vec<Attribute>) {
            self.attributes.insert(at, attributes);
        }

        fn remove_attribute(&mut self, at: AttributeType, id: uuid::Uuid) {
            if let Some(vec) = self.attributes.get_mut(&at) {
                vec.retain(|a| a.uuid != id);
            }
        }

        fn remove_attributes(&mut self, at: AttributeType) {
            self.attributes.remove(&at);
        }
    }

    #[test]
    fn ball_item() {
        let ball: Ball = Ball::new();
        let stats = ball.get_stats();
        let parser = AttributeParser::from(stats.get_all_attributes());

        let debug_txt = ball.get_name().into_text() + "\n---\n" + parser.clone();

        println!("{}", debug_txt.to_ansi_string());

        let total_durability = parser.aggregate_to_value(AttributeType::Durability);

        assert_eq!(total_durability, 110.0);
    }
}
