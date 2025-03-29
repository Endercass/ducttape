pub mod rope;

lazy_static::lazy_static! {
    pub static ref ROPE_TEXTURE: ducttape_item_engine::item::ItemTexture = image::open(asset_to_absolute("item/rope/rope.png")).ok().into();
}

fn asset_to_absolute(asset_relative_path: &str) -> String {
    let res_path = format!("res://assets/{}", asset_relative_path);
    let project_settings = godot::classes::ProjectSettings::singleton();
    project_settings.globalize_path(&res_path).into()
}