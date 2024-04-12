//! # Asset Loader
//! Very simple slot based loader that loads your assets **between frames**.
//! 
//! Since macroquad is single threaded, there's no way to load a texture 
//! 
//! **Note: It doesn't cache the results.**

use std::time::Instant;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use macroquad::file::load_file;
use macroquad::texture::load_texture;
use macroquad::{file::set_pc_assets_folder, texture::Texture2D};
use macroquad::text::{load_ttf_font, Font as MQFOnt};
use macroquad::audio::{load_sound, Sound as MQSound};
use slotmap::{new_key_type, Key, SlotMap};

// mod server;

pub struct Handle<T>
where T: Key {
    pub(crate) key:  T
}


impl<T> Handle<T>
where T: Key {
    pub fn new(key: T) -> Self {
        Self { key }
    }

    pub fn null() -> Self {
        Self {
            key: T::null()
        }
    }
}

new_key_type! { 
    pub struct TextureKey;
    pub struct FontKey;
    pub struct SoundKey;
    pub struct BytesKey;
}

// new_key_type! { struct FontKey; }

// new_key_type! { struct SoundKey; }
// new_key_type! { struct BytesKey; }

/// An enum representing the load status of an asset. 
/// 
/// Useful to throw errors
pub(crate) enum AssetStatus<T> {
    /// The loading has finished successfully
    Done(T),
    /// Something wrong went with the loading
    Failed,
    /// Empty value. Just here for default handles
    Null,
    /// Currently loads
    Loading
}

/// All command types right now only accept paths. 
/// 
/// In the future loading from bytes or `Image` is a must.
/// 
enum LoadCommandType {
    Texture(TextureKey, &'static str),
    Font(FontKey, &'static str),
    Sound(SoundKey, &'static str),
    Bytes(BytesKey, &'static str)
}

#[derive(Resource)]
pub(super) struct AssetStorage {
    pub textures: SlotMap<TextureKey, AssetStatus<Texture2D>>, // Just a reference, can modify if needed
    pub fonts: SlotMap<FontKey, AssetStatus<MQFOnt>>,
    pub sounds: SlotMap<SoundKey, AssetStatus<MQSound>>,
    pub bytes: SlotMap<BytesKey, AssetStatus<Vec<u8>>>,

    to_process: Vec<LoadCommandType>
}

impl Default for AssetStorage {
    fn default() -> Self {
        Self {
            textures: SlotMap::with_key(),
            fonts: SlotMap::with_key(),
            sounds: SlotMap::with_key(),
            bytes: SlotMap::with_key(),
            to_process: Vec::new()
        }
    }
}

#[derive(SystemParam)]
pub struct Assets<'w> {
    storage: ResMut<'w, AssetStorage>,
}

impl<'w> Assets<'w>{
    /// Load a texture from the file path (on wasm uses requests).
    /// 
    /// Attention: it will use the relative path, affected by the `asset_path` from [`QuadAssetPlugin`].
    pub fn load_texture_fs(&mut self, path: &'static str) -> Handle<TextureKey> {
        let key = self.storage.textures.insert(AssetStatus::Loading);
        self.storage.to_process.push(LoadCommandType::Texture(key.clone(), path));
        Handle::new(key)
    }

    pub fn load_sound_fs(&mut self, path: &'static str) -> Handle<SoundKey> {
        let key = self.storage.sounds.insert(AssetStatus::Loading);
        self.storage.to_process.push(LoadCommandType::Sound(key.clone(), path));
        Handle::new(key)
    }

    pub fn load_font_fs(&mut self, path: &'static str) -> Handle<FontKey> {
        let key = self.storage.fonts.insert(AssetStatus::Loading);
        self.storage.to_process.push(LoadCommandType::Font(key.clone(), path));
        Handle::new(key)
    }

    pub fn load_bytes_fs(&mut self, path: &'static str) -> Handle<BytesKey> {
        let key = self.storage.bytes.insert(AssetStatus::Loading);
        self.storage.to_process.push(LoadCommandType::Bytes(key.clone(), path));
        Handle::new(key)
    }
}

#[derive(Resource)]
struct LoaderTimeLimit(u32);

pub struct QuadAssetPlugin {
    pub asset_path: String,
    pub loader_time_limit: u32,
}

impl Default for QuadAssetPlugin {
    fn default() -> Self {
        Self {
            asset_path: "".to_owned(),
            loader_time_limit: 4,
        }
    }
}

impl Plugin for QuadAssetPlugin {
    fn build(&self, app: &mut App) {
        set_pc_assets_folder(&self.asset_path);

        app
            .init_resource::<AssetStorage>()
            .insert_resource(LoaderTimeLimit(self.loader_time_limit))
            .add_systems(PreUpdate, load_assets)
        ;
    }
}

/// This system will try to load as many assets as possible in a span of **n** seconds. 
/// 
/// This number can be customized in the [`QuadAssetPlugin`].  
fn load_assets(w: &mut World) {
    let millis_limit = w.get_resource::<LoaderTimeLimit>().unwrap().0;
    let mut store = w.get_resource_mut::<AssetStorage>().unwrap();
    if store.to_process.len() == 0 {
        return;
    }

    let mut once = true; // Just to escape some bugs 
    let now = Instant::now();
    pollster::block_on(async {
        while once || now.elapsed().as_millis() < millis_limit as u128 {
            once = false;
            match store.to_process.pop().unwrap() {
                LoadCommandType::Texture(k, p) => {
                    let asset = match load_texture(p).await {
                        Ok(t) => AssetStatus::Done(t),
                        Err(err) => {
                            error!("failed to load texture: {:?}", err);
                            AssetStatus::Failed
                        },
                    };
                    store.textures.insert_with_key(|k| asset);
                },
                LoadCommandType::Sound(k, p) => {
                    let asset = match load_sound(p).await {
                        Ok(t) => AssetStatus::Done(t),
                        Err(err) => {
                            error!("failed to load sound file: {:?}", err);
                            AssetStatus::Failed
                        },
                    };
                    store.sounds.insert_with_key(|k| asset);
                },
                LoadCommandType::Font(k, p) => {
                    let asset = match load_ttf_font(p).await {
                        Ok(t) => AssetStatus::Done(t),
                        Err(err) => {
                            error!("failed to load ttf: {:?}", err);
                            AssetStatus::Failed
                        },
                    };
                    store.fonts.insert_with_key(|k| asset);
                },
                LoadCommandType::Bytes(k, p) => {
                    let asset = match load_file(p).await {
                        Ok(t) => AssetStatus::Done(t),
                        Err(err) => {
                            error!("failed to load bytes: {:?}", err);
                            AssetStatus::Failed
                        },
                    };
                    store.bytes.insert_with_key(|k| asset);
                }
            }
        }
    })
}