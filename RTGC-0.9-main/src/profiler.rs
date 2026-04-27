use std::collections::HashMap;
use std::time::Instant;
use std::sync::atomic::AtomicUsize;

// DEBUG: Отладка экспортов profiler - добавлено в Profiler::new()

/// GPU timing query result
#[derive(Debug, Clone)]
pub struct GpuTiming {
    pub name: String,
    pub time_ms: f64,
    pub timestamp: u64,
}

/// Memory allocation tracking
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub size_bytes: usize,
    pub alignment: usize,
    pub tag: String,
    pub timestamp: u64,
}

/// Memory statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_freed: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
    pub allocations_by_tag: HashMap<String, usize>,
}

pub struct Profiler {
    timers: HashMap<String, Instant>,
    measurements: HashMap<String, Vec<f64>>,
    /// GPU timings (if supported by backend)
    gpu_timings: Vec<GpuTiming>,
    /// Memory tracking
    memory_stats: MemoryStats,
    /// Frame counter
    frame_count: u64,
    /// Total CPU time this frame
    frame_cpu_time_ms: f64,
    /// Total GPU time this frame (if available)
    frame_gpu_time_ms: Option<f64>,
    /// Maximum number of measurements to keep per timer (circular buffer)
    max_measurements: usize,
}

impl Profiler {
    /// Maximum number of measurements to store per timer to prevent memory leaks
    pub const MAX_MEASUREMENTS: usize = 1000;

    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
            measurements: HashMap::new(),
            gpu_timings: Vec::new(),
            memory_stats: MemoryStats::default(),
            frame_count: 0,
            frame_cpu_time_ms: 0.0,
            frame_gpu_time_ms: None,
            max_measurements: Self::MAX_MEASUREMENTS,
        }
    }

    pub fn start_timer(&mut self, name: &str) {
        self.timers.insert(name.to_string(), Instant::now());
    }

    pub fn stop_timer(&mut self, name: &str) -> Option<f64> {
        if let Some(start_time) = self.timers.remove(name) {
            let elapsed = start_time.elapsed().as_secs_f64() * 1000.0; // Convert to milliseconds
            
            let measurements = self.measurements.entry(name.to_string())
                .or_insert_with(Vec::new);
            
            // Circular buffer: remove oldest measurement if we're at capacity
            if measurements.len() >= self.max_measurements {
                measurements.remove(0);
                tracing::warn!(target: "profiler", "Measurement buffer for '{}' full, dropping oldest", name);
            }
            
            measurements.push(elapsed);
            self.frame_cpu_time_ms += elapsed;
            Some(elapsed)
        } else {
            tracing::warn!(target: "profiler", "Timer '{}' not found", name);
            None
        }
    }

    /// Record GPU timing (called by renderer when GPU queries are resolved)
    pub fn record_gpu_timing(&mut self, name: &str, time_ms: f64) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|e| {
                tracing::warn!("SystemTime before UNIX_EPOCH: {}", e);
                std::time::Duration::ZERO
            })
            .as_millis() as u64;
        
        self.gpu_timings.push(GpuTiming {
            name: name.to_string(),
            time_ms,
            timestamp,
        });
        
        self.frame_gpu_time_ms = Some(self.frame_gpu_time_ms.unwrap_or(0.0) + time_ms);
    }

    /// Get latest GPU timing for a named section
    pub fn get_gpu_timing(&self, name: &str) -> Option<f64> {
        self.gpu_timings.iter()
            .filter(|t| t.name == name)
            .last()
            .map(|t| t.time_ms)
    }

    /// Track a memory allocation
    pub fn track_allocation(&mut self, size: usize, alignment: usize, tag: &str) {
        self.memory_stats.total_allocated += size;
        self.memory_stats.current_usage += size;
        self.memory_stats.allocation_count += 1;
        
        if self.memory_stats.current_usage > self.memory_stats.peak_usage {
            self.memory_stats.peak_usage = self.memory_stats.current_usage;
        }
        
        *self.memory_stats.allocations_by_tag.entry(tag.to_string()).or_insert(0) += size;
    }

    /// Track a memory deallocation
    pub fn track_deallocation(&mut self, size: usize, tag: &str) {
        self.memory_stats.total_freed += size;
        if self.memory_stats.current_usage >= size {
            self.memory_stats.current_usage -= size;
        } else {
            self.memory_stats.current_usage = 0;
        }
        self.memory_stats.deallocation_count += 1;
        
        if let Some(current) = self.memory_stats.allocations_by_tag.get_mut(tag) {
            if *current >= size {
                *current -= size;
            } else {
                *current = 0;
            }
        }
    }

    /// Get memory statistics
    pub fn get_memory_stats(&self) -> &MemoryStats {
        &self.memory_stats
    }

    /// Begin a new frame (reset per-frame counters)
    pub fn begin_frame(&mut self) {
        self.frame_count += 1;
        self.frame_cpu_time_ms = 0.0;
        self.frame_gpu_time_ms = None;
        self.gpu_timings.clear();
    }

    /// Get current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get CPU time for current frame
    pub fn frame_cpu_time(&self) -> f64 {
        self.frame_cpu_time_ms
    }

    /// Get GPU time for current frame (if available)
    pub fn frame_gpu_time(&self) -> Option<f64> {
        self.frame_gpu_time_ms
    }

    pub fn get_average_time(&self, name: &str) -> Option<f64> {
        if let Some(times) = self.measurements.get(name) {
            if !times.is_empty() {
                Some(times.iter().sum::<f64>() / times.len() as f64)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_last_time(&self, name: &str) -> Option<f64> {
        if let Some(times) = self.measurements.get(name) {
            times.last().copied()
        } else {
            None
        }
    }

    pub fn print_profile_report(&self) {
        tracing::info!(target: "profiler", "=== Performance Profile Report ===");
        tracing::info!(target: "profiler", "Frame: {}", self.frame_count);
        tracing::info!(target: "profiler", "CPU Time: {:.3}ms", self.frame_cpu_time_ms);
        if let Some(gpu_time) = self.frame_gpu_time_ms {
            tracing::info!(target: "profiler", "GPU Time: {:.3}ms", gpu_time);
        }
        tracing::info!(target: "profiler", "--- CPU Timings ---");
        for (name, times) in &self.measurements {
            if !times.is_empty() {
                let avg_time = times.iter().sum::<f64>() / times.len() as f64;
                let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max_time = times.iter().fold(0.0_f64, |a, &b| a.max(b));

                tracing::info!(target: "profiler", 
                    "{}: avg={:.3}ms, min={:.3}ms, max={:.3}ms, calls={}",
                    name,
                    avg_time,
                    min_time,
                    max_time,
                    times.len()
                );
            }
        }
        tracing::info!(target: "profiler", "--- Memory Statistics ---");
        tracing::info!(target: "profiler", "Current Usage: {} bytes ({:.2} MB)", 
            self.memory_stats.current_usage,
            self.memory_stats.current_usage as f64 / 1024.0 / 1024.0);
        tracing::info!(target: "profiler", "Peak Usage: {} bytes ({:.2} MB)",
            self.memory_stats.peak_usage,
            self.memory_stats.peak_usage as f64 / 1024.0 / 1024.0);
        tracing::info!(target: "profiler", "Total Allocated: {} bytes", self.memory_stats.total_allocated);
        tracing::info!(target: "profiler", "Total Freed: {} bytes", self.memory_stats.total_freed);
        tracing::info!(target: "profiler", "Allocation Count: {}", self.memory_stats.allocation_count);
        tracing::info!(target: "profiler", "Deallocation Count: {}", self.memory_stats.deallocation_count);
        tracing::info!(target: "profiler", "--- Allocations by Tag ---");
        for (tag, size) in &self.memory_stats.allocations_by_tag {
            tracing::info!(target: "profiler", "{}: {} bytes ({:.2} MB)", tag, size, *size as f64 / 1024.0 / 1024.0);
        }
        tracing::info!(target: "profiler", "===================================");
    }

    pub fn reset(&mut self) {
        self.timers.clear();
        self.measurements.clear();
        self.gpu_timings.clear();
        self.memory_stats = MemoryStats::default();
        self.frame_count = 0;
        self.frame_cpu_time_ms = 0.0;
        self.frame_gpu_time_ms = None;
    }
}

use std::sync::LazyLock;

// Lazy initialization - используем parking_lot::Mutex который игнорирует отравление
use parking_lot::Mutex;

static PROFILER: LazyLock<Mutex<Profiler>> = LazyLock::new(|| {
    Mutex::new(Profiler::new())
});

fn get_profiler() -> &'static LazyLock<Mutex<Profiler>> {
    &PROFILER
}

#[macro_export]
macro_rules! profile_scope {
    ($name:expr_2021, $block:block) => {{
        let _guard = $crate::profiler::ProfileGuard::new($name);
        $block
    }};
}

pub struct ProfileGuard<'a> {
    name: &'a str,
}

impl<'a> ProfileGuard<'a> {
    pub fn new(name: &'static str) -> Self {
        get_profiler().lock().start_timer(name);
        Self { name }
    }
}

impl<'a> Drop for ProfileGuard<'a> {
    fn drop(&mut self) {
        get_profiler().lock().stop_timer(self.name);
    }
}

pub fn start_timer(name: &str) {
    get_profiler().lock().start_timer(name);
}

pub fn stop_timer(name: &str) -> Option<f64> {
    get_profiler().lock().stop_timer(name)
}

pub fn get_average_time(name: &str) -> Option<f64> {
    get_profiler().lock().get_average_time(name)
}

pub fn get_last_time(name: &str) -> Option<f64> {
    get_profiler().lock().get_last_time(name)
}

pub fn print_profile_report() {
    get_profiler().lock().print_profile_report();
}

pub fn reset_profiler() {
    get_profiler().lock().reset();
}

/// RAII-style profile scope for automatic timing
pub struct ProfileScope {
    name: String,
}

impl ProfileScope {
    pub fn new(name: &str) -> Self {
        let mut profiler = get_profiler().lock();
        profiler.start_timer(name);
        Self { name: name.to_string() }
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        let mut profiler = get_profiler().lock();
        profiler.stop_timer(&self.name);
    }
}

/// Begin a new profiling frame
pub fn begin_frame() {
    let mut profiler = get_profiler().lock();
    profiler.begin_frame();
}

/// End a profiling frame
pub fn end_frame() {
    // Currently no special end-frame logic needed
    // This function is provided for API completeness
}