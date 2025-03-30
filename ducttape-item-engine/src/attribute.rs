use std::collections::HashMap;

use valence_text::{color::NamedColor, Color, IntoText, Text};

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum AttributeReason {
    Hidden,
    Display(String),
}

pub const ALL_ATTRIBUTE_TYPES: [AttributeType; 6] = [
    AttributeType::Sharpness,
    AttributeType::Durability,
    AttributeType::Weight,
    AttributeType::Strength,
    AttributeType::Agility,
    AttributeType::Reach,
];

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, serde::Deserialize)]
pub enum AttributeType {
    /// How much damage the item can deal
    Sharpness,
    /// How much damage the item can take
    Durability,
    /// How much the item weighs
    Weight,
    /// How much weight you can be supported
    Strength,
    /// How fast you can attack
    Agility,
    /// How far you can reach
    Reach,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub uuid: uuid::Uuid,
    pub reason: AttributeReason,
    pub priority: u8,
    pub modifier: AttributeModifier,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum AttributeModifier {
    Multiply(f64),
    Add(f64),
    Set(f64),
}

impl AttributeModifier {
    pub fn is_buff(&self) -> bool {
        match self {
            AttributeModifier::Multiply(m) => *m > 1.0,
            AttributeModifier::Add(a) => *a > 0.0,
            AttributeModifier::Set(_) => false,
        }
    }

    pub fn is_neutral(&self) -> bool {
        match self {
            AttributeModifier::Multiply(m) => *m == 1.0,
            AttributeModifier::Add(a) => *a == 0.0,
            AttributeModifier::Set(_) => true,
        }
    }

    pub fn is_debuff(&self) -> bool {
        match self {
            AttributeModifier::Multiply(m) => *m < 1.0,
            AttributeModifier::Add(a) => *a < 0.0,
            AttributeModifier::Set(_) => false,
        }
    }

    pub fn stat_color(&self) -> NamedColor {
        if self.is_buff() {
            NamedColor::Green
        } else if self.is_neutral() {
            NamedColor::Yellow
        } else if self.is_debuff() {
            NamedColor::Red
        } else {
            NamedColor::White
        }
    }
}

impl std::fmt::Display for AttributeModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeModifier::Multiply(m) => {
                write!(f, " x {}", m)
            }
            AttributeModifier::Add(a) => {
                if *a > 0.0 {
                    write!(f, " + {}", a)
                } else {
                    write!(f, " - {}", a.abs())
                }
            }
            AttributeModifier::Set(s) => {
                write!(f, " = {}", s)
            }
        }
    }
}

impl<'a> IntoText<'a> for AttributeModifier {
    fn into_cow_text(self) -> std::borrow::Cow<'a, Text> {
        format!("{}", self)
            .into_text()
            .color(Color::Named(self.stat_color()))
            .into_cow_text()
    }
}

impl<'a> IntoText<'a> for AttributeType {
    fn into_cow_text(self) -> std::borrow::Cow<'a, Text> {
        (match self {
            AttributeType::Sharpness => "🗡️",
            AttributeType::Durability => "⚡",
            AttributeType::Weight => "🏋️",
            AttributeType::Strength => "💪",
            AttributeType::Agility => "🏃",
            AttributeType::Reach => "🏹",
        })
        .into_cow_text()
    }
}

impl<'a> IntoText<'a> for Attribute {
    fn into_cow_text(self) -> std::borrow::Cow<'a, Text> {
        let mut txt = self.modifier.into_text();
        if let AttributeReason::Display(name) = self.reason {
            txt = txt + " (" + name + ")";
        }

        txt.into_cow_text()
    }
}

/// Helper struct for aggregatijng attributes
#[derive(Clone)]
pub struct AttributeParser {
    attributes: HashMap<AttributeType, Vec<Attribute>>,
}

impl From<HashMap<AttributeType, Vec<Attribute>>> for AttributeParser {
    fn from(attributes: HashMap<AttributeType, Vec<Attribute>>) -> Self {
        AttributeParser { attributes }
    }
}

impl Default for AttributeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeParser {
    pub fn new() -> Self {
        AttributeParser {
            attributes: HashMap::new(),
        }
    }

    pub fn push(&mut self, at: AttributeType, attribute: Attribute) {
        let vec = self.attributes.entry(at).or_default();
        vec.push(attribute);
        self.sort();
    }

    fn sort(&mut self) {
        for (_, vec) in self.attributes.iter_mut() {
            vec.sort_by(|a, b| a.priority.cmp(&b.priority));
        }
    }

    pub fn aggregate_to_value(&self, at: AttributeType) -> f64 {
        let mut value = 0.0;
        if let Some(vec) = self.attributes.get(&at) {
            for attribute in vec.iter() {
                match attribute.modifier {
                    AttributeModifier::Multiply(m) => {
                        value *= m;
                    }
                    AttributeModifier::Add(a) => {
                        value += a;
                    }
                    AttributeModifier::Set(s) => {
                        value = s;
                    }
                }
            }
        }
        value
    }

    pub fn aggregate_to_values(&self) -> HashMap<AttributeType, f64> {
        let mut result = HashMap::new();
        for (at, _) in self.attributes.iter() {
            result.insert(*at, self.aggregate_to_value(*at));
        }
        result
    }

    pub fn aggregate_to_fixed_attribute(&self, at: AttributeType) -> Attribute {
        Attribute {
            uuid: uuid::Uuid::new_v4(),
            reason: AttributeReason::Hidden,
            priority: 0,
            modifier: AttributeModifier::Set(self.aggregate_to_value(at)),
        }
    }

    pub fn aggregate_to_fixed_attributes(&self) -> HashMap<AttributeType, Attribute> {
        self.attributes
            .keys()
            .map(|at| (*at, self.aggregate_to_fixed_attribute(*at)))
            .collect()
    }

    pub fn aggregate_to_component(&self, at: AttributeType) -> Text {
        let mut txt = "".into_text();
        if let Some(vec) = self.attributes.get(&at) {
            let mut vec = vec.clone();
            vec.sort_by_key(|a| a.priority);
            for attribute in vec.iter() {
                txt = txt + attribute.clone() + "\n";
            }
        }
        txt + self.aggregate_to_fixed_attribute(at)
    }

    pub fn aggregate_to_components(&self) -> HashMap<AttributeType, Text> {
        self.attributes
            .keys()
            .map(|at| (*at, self.aggregate_to_component(*at)))
            .collect()
    }
}

impl<'a> IntoText<'a> for AttributeParser {
    fn into_cow_text(self) -> std::borrow::Cow<'a, Text> {
        let mut txt = "".into_text();

        let mut components: Vec<_> = self.aggregate_to_components().into_iter().collect();
        components.sort_by_key(|(at, _)| *at);
        for (at, c) in components {
            txt = txt + at + ":\n" + c + "\n";
        }

        txt.into_cow_text()
    }
}
