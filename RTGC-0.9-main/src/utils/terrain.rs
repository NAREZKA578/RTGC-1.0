/// Утилиты для работы с ландшафтом
use nalgebra::Vector3;

/// Вычисляет нормаль поверхности terrain методом центральных разностей
/// 
/// # Аргументы
/// * `height_getter` - функция, возвращающая высоту в точке (x, z)
/// * `x` - координата X
/// * `z` - координата Z
/// * `sample_dist` - расстояние между сэмплами (обычно 1.0)
/// 
/// # Возвращает
/// Нормализованный вектор нормали (направлен вверх)
#[inline]
pub fn compute_terrain_normal<F>(height_getter: F, x: f32, z: f32, sample_dist: f32) -> Vector3<f32>
where
    F: Fn(f32, f32) -> f32,
{
    let h_left = height_getter(x - sample_dist, z);
    let h_right = height_getter(x + sample_dist, z);
    let h_back = height_getter(x, z - sample_dist);
    let h_front = height_getter(x, z + sample_dist);

    // Central differences для касательных
    // tangent_x = (2*sample_dist, h_right - h_left, 0)
    // tangent_z = (0, h_front - h_back, 2*sample_dist)
    // normal = tangent_x cross tangent_z
    let normal = Vector3::new(
        h_left - h_right,
        2.0 * sample_dist,
        h_back - h_front,
    );

    // Нормализация
    let len = normal.magnitude();
    if len > 1e-4 {
        normal / len
    } else {
        Vector3::y() // По умолчанию - вверх
    }
}

/// Вычисляет нормаль terrain из heightmap (Vec<Vec<f32>>)
/// 
/// # Аргументы
/// * `heightmap` - 2D массив высот [row][col] = height
/// * `scale` - масштаб terrain (x, y, z)
/// * `origin` - мировая позиция начала terrain
/// * `world_x`, `world_z` - мировые координаты точки
/// * `sample_dist` - расстояние между сэмплами в мировых координатах
pub fn compute_terrain_normal_from_heightmap(
    heightmap: &[Vec<f32>],
    scale: Vector3<f32>,
    origin: Vector3<f32>,
    world_x: f32,
    world_z: f32,
    sample_dist: f32,
) -> Vector3<f32> {
    let height_getter = |wx: f32, wz: f32| -> f32 {
        // Преобразуем мировые координаты в индексы heightmap
        let local_x = (wx - origin.x) / scale.x;
        let local_z = (wz - origin.z) / scale.z;
        get_height_from_heightmap(heightmap, local_x, local_z, scale.y, 0.0)
    };
    
    compute_terrain_normal(height_getter, world_x, world_z, sample_dist)
}

/// Получает высоту из heightmap с билинейной интерполяцией
#[inline]
pub fn get_height_from_heightmap(
    heightmap: &[Vec<f32>],
    local_x: f32,
    local_z: f32,
    height_scale: f32,
    default_height: f32,
) -> f32 {
    let rows = heightmap.len();
    if rows == 0 {
        return default_height;
    }
    let cols = heightmap[0].len();
    if cols == 0 {
        return default_height;
    }
    
    let x0 = local_x.floor() as usize;
    let z0 = local_z.floor() as usize;
    let x1 = (x0 + 1).min(cols - 1);
    let z1 = (z0 + 1).min(rows - 1);
    let x0 = x0.min(cols - 1);
    let z0 = z0.min(rows - 1);
    
    let fx = local_x - x0 as f32;
    let fz = local_z - z0 as f32;
    
    // Билинейная интерполяция
    let h00 = heightmap[z0][x0] * height_scale;
    let h10 = heightmap[z0][x1] * height_scale;
    let h01 = heightmap[z1][x0] * height_scale;
    let h11 = heightmap[z1][x1] * height_scale;
    
    let h0 = h00 * (1.0 - fx) + h10 * fx;
    let h1 = h01 * (1.0 - fx) + h11 * fx;
    
    h0 * (1.0 - fz) + h1 * fz
}
