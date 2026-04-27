//! Asynchronous world streaming system
//! 
//! Handles:
//! - Multi-threaded chunk loading
//! - Priority-based loading queue
//! - Background mesh generation
//! - GPU resource upload

use std::collections::VecDeque;

use std::thread::{self, JoinHandle};
use crossbeam_channel::{bounded, Sender, Receiver, TrySendError};
use tracing::{debug, warn, error};

use super::chunk::{Chunk, ChunkId, generate_chunk_mesh, TerrainVertex};

/// Configuration for world streaming
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Radius (in chunks) around player to load
    pub load_radius: u32,
    /// Radius (in chunks) around player to keep loaded
    pub unload_radius: u32,
    /// Maximum concurrent chunk loads
    pub max_concurrent_loads: usize,
    /// Interval between chunk updates in milliseconds
    pub chunk_update_interval_ms: u64,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            load_radius: 5,
            unload_radius: 7,
            max_concurrent_loads: 4,
            chunk_update_interval_ms: 100,
        }
    }
}

/// Message sent from main thread to streaming thread
#[derive(Debug)]
enum StreamMessage {
    /// Queue a chunk for loading
    LoadChunk(Chunk),
    /// Cancel loading of a chunk
    CancelLoad(ChunkId),
    /// Shutdown the streaming system
    Shutdown,
}

/// Result of chunk loading operation
#[derive(Debug)]
pub struct LoadedChunk {
    pub chunk: Chunk,
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
    pub load_time_ms: f32,
}

/// World streaming manager
pub struct WorldStreamer {
    /// Configuration
    config: StreamingConfig,
    /// Channel to send load requests to worker thread
    request_sender: Sender<StreamMessage>,
    /// Channel to receive loaded chunks from worker thread
    result_receiver: Receiver<LoadedChunk>,
    /// Chunks pending upload to GPU
    pending_chunks: Vec<LoadedChunk>,
    /// Handle to the streaming thread
    _thread_handle: Option<JoinHandle<()>>,
    /// Statistics
    stats: StreamingStats,
}

#[derive(Debug, Default)]
pub struct StreamingStats {
    pub chunks_loaded: u32,
    pub chunks_failed: u32,
    pub total_load_time_ms: f32,
    pub current_queue_size: usize,
}

impl WorldStreamer {
    pub fn new(config: StreamingConfig) -> Self {
        // Create channels
        let (request_sender, request_receiver) = bounded::<StreamMessage>(100);
        let (result_sender, result_receiver) = bounded::<LoadedChunk>(100);
        
        // Spawn streaming thread
        let thread_config = config.clone();
        let thread_handle = thread::spawn(move || {
            Self::streaming_thread(thread_config, request_receiver, result_sender);
        });
        
        Self {
            config,
            request_sender,
            result_receiver,
            pending_chunks: Vec::new(),
            _thread_handle: Some(thread_handle),
            stats: StreamingStats::default(),
        }
    }
    
    /// The background streaming thread function
    fn streaming_thread(
        config: StreamingConfig,
        requests: Receiver<StreamMessage>,
        results: Sender<LoadedChunk>,
    ) {
        debug!("Streaming thread started");
        
        let mut load_queue: VecDeque<Chunk> = VecDeque::new();
        let mut active_loads: Vec<(ChunkId, std::time::Instant)> = Vec::new();
        
        loop {
            // Try to receive a message (non-blocking with timeout)
            match requests.recv_timeout(std::time::Duration::from_millis(10)) {
                Ok(msg) => match msg {
                    StreamMessage::LoadChunk(chunk) => {
                        debug!("Queuing chunk {:?} for loading", chunk.id);
                        load_queue.push_back(chunk);
                    }
                    StreamMessage::CancelLoad(chunk_id) => {
                        debug!("Cancelling load for chunk {:?}", chunk_id);
                        load_queue.retain(|c| c.id != chunk_id);
                        active_loads.retain(|(id, _)| *id != chunk_id);
                    }
                    StreamMessage::Shutdown => {
                        debug!("Streaming thread shutting down");
                        break;
                    }
                },
                Err(_) => {
                    // Timeout, continue processing
                }
            }
            
            // Start new loads if we have capacity
            while active_loads.len() < config.max_concurrent_loads {
                if let Some(chunk) = load_queue.pop_front() {
                    let chunk_id = chunk.id;
                    let start_time = std::time::Instant::now();
                    
                    // Process chunk (generate mesh)
                    match Self::process_chunk(&chunk) {
                        Ok((vertices, indices)) => {
                            let load_time = start_time.elapsed().as_secs_f32() * 1000.0;
                            
                            let loaded = LoadedChunk {
                                chunk,
                                vertices,
                                indices,
                                load_time_ms: load_time,
                            };
                            
                            // Send result
                            if let Err(e) = results.send(loaded) {
                                error!("Failed to send loaded chunk: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to process chunk {:?}: {}", chunk_id, e);
                        }
                    }
                    
                    active_loads.push((chunk_id, start_time));
                } else {
                    break; // No more chunks to load
                }
            }
            
            // Clean up completed loads (they're already sent via channel)
            // In a real implementation, we'd track async tasks here
        }
        
        debug!("Streaming thread stopped");
    }
    
    /// Process a chunk - generate mesh data
    fn process_chunk(chunk: &Chunk) -> Result<(Vec<TerrainVertex>, Vec<u32>), String> {
        // Generate mesh at LOD 0 (highest quality)
        let (vertices, indices) = generate_chunk_mesh(&chunk.data, 0);
        Ok((vertices, indices))
    }
    
    /// Queue a chunk for loading
    pub fn queue_chunk_load(&mut self, chunk: Chunk) {
        if let Err(e) = self.request_sender.try_send(StreamMessage::LoadChunk(chunk)) {
            match e {
                TrySendError::Full(_) => {
                    warn!("Stream request queue is full, dropping chunk load request");
                    self.stats.chunks_failed += 1;
                }
                TrySendError::Disconnected(_) => {
                    error!("Stream request channel disconnected");
                }
            }
        }
    }
    
    /// Cancel loading of a chunk
    pub fn cancel_chunk_load(&mut self, chunk_id: ChunkId) {
        let _ = self.request_sender.try_send(StreamMessage::CancelLoad(chunk_id));
    }
    
    /// Poll for completed chunk loads
    pub fn poll_loaded_chunk(&mut self) -> Option<LoadedChunk> {
        match self.result_receiver.try_recv() {
            Ok(loaded) => {
                self.stats.chunks_loaded += 1;
                self.stats.total_load_time_ms += loaded.load_time_ms;
                Some(loaded)
            }
            Err(_) => None,
        }
    }
    
    /// Get streaming statistics
    pub fn get_stats(&self) -> &StreamingStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = StreamingStats::default();
    }
}

impl Drop for WorldStreamer {
    fn drop(&mut self) {
        // Signal shutdown
        let _ = self.request_sender.send(StreamMessage::Shutdown);
        
        // Wait for thread to finish
        if let Some(handle) = self._thread_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Helper for managing chunk load priorities
pub struct ChunkPriorityQueue {
    queue: Vec<(ChunkId, f32)>, // (chunk_id, priority - lower is higher priority)
}

impl ChunkPriorityQueue {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
        }
    }
    
    /// Add or update a chunk's priority
    pub fn insert(&mut self, chunk_id: ChunkId, priority: f32) {
        // Remove existing entry if present
        self.queue.retain(|(id, _)| *id != chunk_id);
        
        // Insert maintaining sorted order (lowest priority first)
        let pos = self.queue.partition_point(|(_, p)| *p < priority);
        self.queue.insert(pos, (chunk_id, priority));
    }
    
    /// Remove a chunk from the queue
    pub fn remove(&mut self, chunk_id: ChunkId) -> bool {
        let pos = self.queue.iter().position(|(id, _)| *id == chunk_id);
        if let Some(pos) = pos {
            self.queue.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Get the highest priority chunk (lowest priority value)
    pub fn pop_highest_priority(&mut self) -> Option<ChunkId> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0).0)
        }
    }
    
    /// Get queue length
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Default for ChunkPriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::ChunkData;
    
    #[test]
    fn test_streamer_creation() {
        let config = StreamingConfig::default();
        let streamer = WorldStreamer::new(config);
        
        assert_eq!(streamer.stats.chunks_loaded, 0);
    }
    
    #[test]
    fn test_priority_queue_ordering() {
        let mut queue = ChunkPriorityQueue::new();
        
        queue.insert(ChunkId::new(0, 0), 5.0);
        queue.insert(ChunkId::new(1, 0), 2.0);
        queue.insert(ChunkId::new(2, 0), 8.0);
        queue.insert(ChunkId::new(3, 0), 1.0);
        
        // Should come out in priority order (lowest first)
        assert_eq!(queue.pop_highest_priority(), Some(ChunkId::new(3, 0)));
        assert_eq!(queue.pop_highest_priority(), Some(ChunkId::new(1, 0)));
        assert_eq!(queue.pop_highest_priority(), Some(ChunkId::new(0, 0)));
        assert_eq!(queue.pop_highest_priority(), Some(ChunkId::new(2, 0)));
    }
    
    #[test]
    fn test_priority_queue_update() {
        let mut queue = ChunkPriorityQueue::new();
        
        queue.insert(ChunkId::new(0, 0), 5.0);
        queue.insert(ChunkId::new(0, 0), 3.0); // Update priority
        
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop_highest_priority(), Some(ChunkId::new(0, 0)));
    }
    
    #[test]
    fn test_priority_queue_remove() {
        let mut queue = ChunkPriorityQueue::new();
        
        queue.insert(ChunkId::new(0, 0), 5.0);
        queue.insert(ChunkId::new(1, 0), 3.0);
        
        assert!(queue.remove(ChunkId::new(0, 0)));
        assert!(!queue.remove(ChunkId::new(0, 0))); // Already removed
        
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop_highest_priority(), Some(ChunkId::new(1, 0)));
    }
}
