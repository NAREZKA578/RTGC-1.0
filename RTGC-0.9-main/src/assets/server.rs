//! Asset Server - централизованная загрузка и управление ресурсами
//! 
//! Поддерживает текстуры, шейдеры, меши, шрифты с кэшированием

use crate::graphics::rhi::{IDevice, ResourceHandle, TextureDescription, TextureFormat, TextureUsage, TextureType, TextureDimension, ResourceState};
use crate::graphics::font::FontAtlas;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Типы ассетов
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AssetType {
    Texture,
    Shader,
    Mesh,
    Font,
}

/// Ключ для кэша ассетов
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AssetKey {
    pub asset_type: AssetType,
    pub path: String,
}

impl AssetKey {
    pub fn texture(path: &str) -> Self {
        Self {
            asset_type: AssetType::Texture,
            path: path.to_string(),
        }
    }
    
    pub fn shader(path: &str) -> Self {
        Self {
            asset_type: AssetType::Shader,
            path: path.to_string(),
        }
    }
    
    pub fn font(path: &str) -> Self {
        Self {
            asset_type: AssetType::Font,
            path: path.to_string(),
        }
    }
}

/// Загруженная текстура
pub struct LoadedTexture {
    pub handle: ResourceHandle,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
}

/// Загруженный шрифт
pub struct LoadedFont {
    pub atlas: Arc<FontAtlas>,
    pub size: f32,
}

/// Asset Server
pub struct AssetServer {
    device: Arc<dyn IDevice>,
    /// Кэш текстур
    textures: HashMap<AssetKey, LoadedTexture>,
    /// Кэш шрифтов (путь -> (размер -> LoadedFont))
    fonts: HashMap<String, HashMap<u32, LoadedFont>>,
    /// Базовый путь для ассетов
    base_path: String,
}

impl AssetServer {
    pub fn new(device: Arc<dyn IDevice>, base_path: &str) -> Self {
        Self {
            device,
            textures: HashMap::new(),
            fonts: HashMap::new(),
            base_path: base_path.to_string(),
        }
    }
    
    /// Получить полный путь к ассету
    fn get_full_path(&self, path: &str) -> String {
        if Path::new(path).is_absolute() {
            path.to_string()
        } else {
            format!("{}/{}", self.base_path, path)
        }
    }
    
    /// Загрузить текстуру из файла
    pub fn load_texture(&mut self, path: &str) -> Result<ResourceHandle, String> {
        let key = AssetKey::texture(path);
        
        // Проверяем кэш
        if let Some(loaded) = self.textures.get(&key) {
            return Ok(loaded.handle);
        }
        
        let full_path = self.get_full_path(path);
        
        // Загружаем изображение через image крейт
        let img = image::open(&full_path)
            .map_err(|e| format!("Failed to load texture {}: {}", full_path, e))?;
        
        // Конвертируем в RGBA8
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        
        // Создаём описание текстуры
        let desc = TextureDescription {
            texture_type: TextureType::Texture2D,
            format: TextureFormat::Rgba8Unorm,
            width: width as u32,
            height: height as u32,
            depth: 1,
            mip_levels: 1,
            array_size: 1,
            usage: TextureUsage::SHADER_READ | TextureUsage::TRANSFER_DST,
            initial_state: ResourceState::ShaderResource,
            dimension: TextureDimension::D2,
            depth_or_array_layers: 1,
        };
        
        // Создаём текстуру в RHI
        let handle = self.device.create_texture(&desc)
            .map_err(|e| format!("Failed to create texture: {:?}", e))?;
        
        // Загружаем данные в текстуру
        self.device.update_texture(handle, 0, 0, 0, width as u32, height as u32, 1, rgba.as_ref())
            .map_err(|e| format!("Failed to update texture: {:?}", e))?;
        
        let loaded = LoadedTexture {
            handle,
            width: width as u32,
            height: height as u32,
            format: TextureFormat::Rgba8Unorm,
        };
        
        self.textures.insert(key, loaded);
        Ok(handle)
    }
    
    /// Загрузить шрифт
    pub fn load_font(&mut self, path: &str, size: u32) -> Result<Arc<FontAtlas>, String> {
        // Проверяем кэш
        if let Some(fonts_by_size) = self.fonts.get(path) {
            if let Some(loaded) = fonts_by_size.get(&size) {
                return Ok(loaded.atlas.clone());
            }
        }
        
        let full_path = self.get_full_path(path);
        
        // Загружаем шрифт
        let mut atlas = FontAtlas::load_from_file(&full_path, size as f32)?;
        
        // Создаём текстуру для атласа глифов
        let (width, height) = atlas.get_atlas_dimensions();
        let atlas_data = atlas.get_atlas_data().to_vec();
        
        let desc = TextureDescription {
            texture_type: TextureType::Texture2D,
            format: TextureFormat::Rgba8Unorm,
            width,
            height,
            depth: 1,
            mip_levels: 1,
            array_size: 1,
            usage: TextureUsage::SHADER_READ | TextureUsage::TRANSFER_DST,
            initial_state: ResourceState::ShaderResource,
            dimension: TextureDimension::D2,
            depth_or_array_layers: 1,
        };
        
        let texture_handle = self.device.create_texture(&desc)
            .map_err(|e| format!("Failed to create font texture: {:?}", e))?;
        
        // Загружаем данные в текстуру
        self.device.update_texture(texture_handle, 0, 0, 0, width, height, 1, &atlas_data)
            .map_err(|e| format!("Failed to update font texture: {:?}", e))?;
        
        // Сохраняем handle в атласе
        atlas.texture = Some(texture_handle);
        
        let arc_atlas = Arc::new(atlas);
        
        // Кэшируем
        let size_key = size as u32;
        self.fonts
            .entry(path.to_string())
            .or_insert_with(HashMap::new)
            .insert(size_key, LoadedFont {
                atlas: arc_atlas.clone(),
                size: size as f32,
            });
        
        Ok(arc_atlas)
    }
    
    /// Получить текстуру из кэша
    pub fn get_texture(&self, path: &str) -> Option<ResourceHandle> {
        let key = AssetKey::texture(path);
        self.textures.get(&key).map(|t| t.handle)
    }
    
    /// Получить шрифт из кэша
    pub fn get_font(&self, path: &str, size: f32) -> Option<Arc<FontAtlas>> {
        let size_key = size as u32;
        self.fonts
            .get(path)
            .and_then(|fonts| fonts.get(&size_key))
            .map(|f| f.atlas.clone())
    }
    
    /// Выгрузить ассет из кэша
    pub fn unload(&mut self, path: &str, asset_type: AssetType) {
        let key = match asset_type {
            AssetType::Texture => AssetKey::texture(path),
            AssetType::Shader => AssetKey::shader(path),
            AssetType::Font => AssetKey::font(path),
            AssetType::Mesh => return, // Меши пока не кэшируются
        };
        
        match asset_type {
            AssetType::Texture => {
                self.textures.remove(&key);
            }
            AssetType::Font => {
                if let Some(fonts) = self.fonts.get_mut(path) {
                    fonts.clear();
                }
            }
            _ => {}
        }
    }
    
    /// Очистить весь кэш
    pub fn clear_cache(&mut self) {
        self.textures.clear();
        self.fonts.clear();
    }
}
