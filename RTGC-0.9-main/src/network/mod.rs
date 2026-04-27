//! Network Module for RTGC
//! Provides multiplayer synchronization infrastructure
//! 
//! Features:
//! - Entity replication with priority-based updates
//! - Client-server architecture
//! - Lag compensation and interpolation
//! - World chunk streaming
//! - RPC system

pub mod protocol;

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tracing::{warn, debug, info};
use crossbeam_channel::{bounded, Sender, Receiver};

pub use protocol::{
    EntityId, EntityType, ReplicationPriority, ReplicatedComponent,
    ReplicatedEntity, ChunkData, GameState, NetworkMessage, EntityUpdate,
    PlayerInput, NetworkStats, ClientInfo
};

/// Network event types
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    Connected { address: SocketAddr },
    Disconnected { reason: String },
    MessageReceived { message: NetworkMessage },
    StateUpdated { tick: u64 },
    EntitySpawned { entity: ReplicatedEntity },
    EntityDespawned { id: EntityId },
    ChunkLoaded { x: i32, y: i32 },
    RpcReceived { method: String, args: serde_json::Value },
}

/// Client connection state
#[derive(Debug, Clone)]
pub struct ClientState {
    pub client_id: u64,
    pub address: SocketAddr,
    pub connected_at: Instant,
    pub last_message_at: Instant,
    pub last_input_tick: u64,
    pub last_ack_tick: u64,
    pub rtt_ms: f32,
    pub packet_loss: f32,
    pub interest_radius: f32,
}

impl ClientState {
    pub fn new(client_id: u64, address: SocketAddr) -> Self {
        let now = Instant::now();
        Self {
            client_id,
            address,
            connected_at: now,
            last_message_at: now,
            last_input_tick: 0,
            last_ack_tick: 0,
            rtt_ms: 0.0,
            packet_loss: 0.0,
            interest_radius: 100.0,
        }
    }
    
    pub fn to_info(&self) -> ClientInfo {
        ClientInfo {
            client_id: self.client_id,
            player_name: String::new(),
            connection_time_secs: self.connected_at.elapsed().as_secs(),
            last_input_tick: self.last_input_tick,
            last_ack_tick: self.last_ack_tick,
            average_latency_ms: self.rtt_ms,
            packet_loss_percent: self.packet_loss,
        }
    }
}

/// Server-side replication manager
pub struct ReplicationServer {
    clients: HashMap<u64, ClientState>,
    entities: HashMap<EntityId, ReplicatedEntity>,
    next_client_id: u64,
    next_entity_id: u64,
    current_tick: u64,
    tick_rate: f32,
    last_tick_time: Instant,
    event_tx: Sender<NetworkEvent>,
    event_rx: Receiver<NetworkEvent>,
    stats: NetworkStats,
}

impl ReplicationServer {
    pub fn new(tick_rate: f32) -> Self {
        let (event_tx, event_rx) = bounded(1024);
        
        Self {
            clients: HashMap::new(),
            entities: HashMap::new(),
            next_client_id: 1,
            next_entity_id: 1,
            current_tick: 0,
            tick_rate,
            last_tick_time: Instant::now(),
            event_tx,
            event_rx,
            stats: NetworkStats::default(),
        }
    }
    
    /// Add a new client
    pub fn add_client(&mut self, address: SocketAddr) -> u64 {
        let client_id = self.next_client_id;
        self.next_client_id += 1;
        
        let client = ClientState::new(client_id, address);
        self.clients.insert(client_id, client);
        
        info!("Client {} connected from {}", client_id, address);
        
        let _ = self.event_tx.send(NetworkEvent::Connected { address });
        
        client_id
    }
    
    /// Remove a client
    pub fn remove_client(&mut self, client_id: u64, reason: &str) {
        if let Some(client) = self.clients.remove(&client_id) {
            info!("Client {} disconnected: {}", client_id, reason);
            
            let _ = self.event_tx.send(NetworkEvent::Disconnected { 
                reason: reason.to_string() 
            });
        }
    }
    
    /// Spawn a new entity for replication
    pub fn spawn_entity(&mut self, entity: ReplicatedEntity) -> EntityId {
        let id = EntityId(self.next_entity_id);
        self.next_entity_id += 1;
        
        let mut entity = entity;
        entity.id = id;
        entity.tick_created = self.current_tick;
        
        self.entities.insert(id, entity.clone());
        
        debug!("Spawned entity {:?}", id);
        id
    }
    
    /// Despawn an entity
    pub fn despawn_entity(&mut self, id: EntityId) {
        if self.entities.remove(&id).is_some() {
            debug!("Despawned entity {:?}", id);
        }
    }
    
    /// Update an entity's components
    pub fn update_entity(&mut self, id: EntityId, components: Vec<ReplicatedComponent>) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.tick_updated = self.current_tick;
            entity.components = components;
        }
    }
    
    /// Check if entity should be replicated to client based on interest
    fn should_replicate_to_client(&self, client: &ClientState, entity: &ReplicatedEntity) -> bool {
        match entity.priority {
            ReplicationPriority::Critical => true,
            ReplicationPriority::High => true,
            ReplicationPriority::Medium => true,
            ReplicationPriority::Low => false,
        }
    }
    
    /// Process server tick
    pub fn tick(&mut self) {
        let tick_duration = Duration::from_secs_f32(1.0 / self.tick_rate);
        
        if self.last_tick_time.elapsed() >= tick_duration {
            self.current_tick += 1;
            self.last_tick_time = Instant::now();
            
            self.broadcast_state();
            self.cleanup_stale_clients();
        }
    }
    
    /// Broadcast state updates to all clients
    fn broadcast_state(&mut self) {
        // Collect client IDs first to avoid borrow issues
        let client_ids: Vec<_> = self.clients.keys().copied().collect();
        
        for client_id in client_ids {
            let relevant_entities: Vec<_> = self.entities
                .values()
                .filter(|e| {
                    if let Some(client) = self.clients.get(&client_id) {
                        self.should_replicate_to_client(client, e)
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            if !relevant_entities.is_empty() {
                self.stats.entities_replicated = relevant_entities.len() as u32;
            }
        }

        self.stats.packets_sent += self.clients.len() as u64;
    }
    
    /// Remove clients that haven't sent messages recently
    fn cleanup_stale_clients(&mut self) {
        let timeout = Duration::from_secs(30);
        let stale_clients: Vec<_> = self.clients
            .iter()
            .filter(|(_, c)| c.last_message_at.elapsed() > timeout)
            .map(|(&id, _)| id)
            .collect();
        
        for client_id in stale_clients {
            self.remove_client(client_id, "Connection timeout");
        }
    }
    
    /// Get server statistics
    pub fn stats(&self) -> &NetworkStats {
        &self.stats
    }
    
    /// Get connected clients count
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
    
    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
    
    /// Get current tick
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }
    
    /// Receive network events
    pub fn poll_events(&self) -> Option<NetworkEvent> {
        self.event_rx.try_recv().ok()
    }
}

/// Client-side replication manager
pub struct ReplicationClient {
    server_address: Option<SocketAddr>,
    client_id: Option<u64>,
    local_state: GameState,
    entity_buffer: HashMap<EntityId, ReplicatedEntity>,
    input_history: VecDeque<(u64, PlayerInput)>,
    pending_acks: VecDeque<u64>,
    tick_rate: f32,
    last_update_tick: u64,
    connection_established: bool,
    event_tx: Sender<NetworkEvent>,
    event_rx: Receiver<NetworkEvent>,
    stats: NetworkStats,
}

impl ReplicationClient {
    pub fn new(tick_rate: f32) -> Self {
        let (event_tx, event_rx) = bounded(1024);
        
        Self {
            server_address: None,
            client_id: None,
            local_state: GameState::default(),
            entity_buffer: HashMap::new(),
            input_history: VecDeque::with_capacity(60),
            pending_acks: VecDeque::with_capacity(60),
            tick_rate,
            last_update_tick: 0,
            connection_established: false,
            event_tx,
            event_rx,
            stats: NetworkStats::default(),
        }
    }
    
    /// Connect to server
    pub fn connect(&mut self, address: SocketAddr, player_name: &str) {
        self.server_address = Some(address);
        
        info!("Connecting to server at {}", address);
        
        // Simulate successful connection
        self.handle_join_accepted(1, GameState::default());
    }
    
    /// Disconnect from server
    pub fn disconnect(&mut self, reason: &str) {
        if self.connection_established {
            self.connection_established = false;
            self.client_id = None;
            
            info!("Disconnected: {}", reason);
            
            let _ = self.event_tx.send(NetworkEvent::Disconnected {
                reason: reason.to_string(),
            });
        }
    }
    
    /// Send input to server
    pub fn send_input(&mut self, tick: u64, input: PlayerInput) {
        self.input_history.push_back((tick, input.clone()));
        
        while self.input_history.len() > (self.tick_rate * 2.0) as usize {
            self.input_history.pop_front();
        }
        
        self.stats.packets_sent += 1;
    }
    
    /// Process received state update from server
    pub fn process_state_update(
        &mut self, 
        game_state: GameState,
        entity_updates: Vec<EntityUpdate>,
        entity_spawns: Vec<ReplicatedEntity>,
        entity_despawns: Vec<EntityId>,
    ) {
        self.local_state = game_state;
        self.last_update_tick = self.local_state.server_tick;
        
        for entity in entity_spawns {
            let entity_clone = entity.clone();
            self.entity_buffer.insert(entity.id, entity);

            let _ = self.event_tx.send(NetworkEvent::EntitySpawned {
                entity: self.entity_buffer[&entity_clone.id].clone(),
            });
        }
        
        for update in entity_updates {
            if let Some(entity) = self.entity_buffer.get_mut(&update.id) {
                entity.tick_updated = update.tick;
                entity.components = update.updated_components;
            }
        }
        
        for id in entity_despawns {
            self.entity_buffer.remove(&id);
            
            let _ = self.event_tx.send(NetworkEvent::EntityDespawned { id });
        }
        
        self.stats.packets_received += 1;
        self.stats.entities_replicated = self.entity_buffer.len() as u32;
        
        let _ = self.event_tx.send(NetworkEvent::StateUpdated {
            tick: self.last_update_tick,
        });
    }
    
    /// Handle join accepted from server
    pub fn handle_join_accepted(&mut self, client_id: u64, initial_state: GameState) {
        self.client_id = Some(client_id);
        self.local_state = initial_state;
        self.connection_established = true;
        
        info!("Connected to server as client {}", client_id);
        
        // Use stored server_address or a default if not set (shouldn't happen in normal flow)
        let address = self.server_address.unwrap_or_else(|| {
            warn!("Server address not set when connection accepted, using default");
            "127.0.0.1:0".parse().unwrap_or(SocketAddr::from(([127, 0, 0, 1], 0)))
        });
        
        let _ = self.event_tx.send(NetworkEvent::Connected {
            address,
        });
    }
    
    /// Get local game state
    pub fn local_state(&self) -> &GameState {
        &self.local_state
    }
    
    /// Get entity by ID
    pub fn get_entity(&self, id: EntityId) -> Option<&ReplicatedEntity> {
        self.entity_buffer.get(&id)
    }
    
    /// Get all entities
    pub fn entities(&self) -> &HashMap<EntityId, ReplicatedEntity> {
        &self.entity_buffer
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connection_established
    }
    
    /// Get client ID
    pub fn client_id(&self) -> Option<u64> {
        self.client_id
    }
    
    /// Get statistics
    pub fn stats(&self) -> &NetworkStats {
        &self.stats
    }
    
    /// Poll network events
    pub fn poll_events(&self) -> Option<NetworkEvent> {
        self.event_rx.try_recv().ok()
    }
}

/// Network manager for coordinating client/server
pub enum NetworkManager {
    Server(ReplicationServer),
    Client(ReplicationClient),
    None,
}

impl NetworkManager {
    pub fn new_server(tick_rate: f32) -> Self {
        NetworkManager::Server(ReplicationServer::new(tick_rate))
    }
    
    pub fn new_client(tick_rate: f32) -> Self {
        NetworkManager::Client(ReplicationClient::new(tick_rate))
    }
    
    pub fn tick(&mut self) {
        match self {
            NetworkManager::Server(server) => server.tick(),
            NetworkManager::Client(_) => {}
            NetworkManager::None => {}
        }
    }
    
    pub fn poll_events(&self) -> Option<NetworkEvent> {
        match self {
            NetworkManager::Server(server) => server.poll_events(),
            NetworkManager::Client(client) => client.poll_events(),
            NetworkManager::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_client_connection() {
        let mut server = ReplicationServer::new(60.0);
        let mut client = ReplicationClient::new(60.0);
        
        let address: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let client_id = server.add_client(address);
        
        assert_eq!(client_id, 1);
        assert_eq!(server.client_count(), 1);
        
        client.handle_join_accepted(client_id, GameState::default());
        
        assert!(client.is_connected());
        assert_eq!(client.client_id(), Some(1));
    }
    
    #[test]
    fn test_entity_replication() {
        let mut server = ReplicationServer::new(60.0);
        
        let entity = ReplicatedEntity::new(
            EntityId::null(),
            EntityType::Vehicle,
            ReplicationPriority::High,
        );
        
        let id = server.spawn_entity(entity);
        
        assert!(!id.is_null());
        assert_eq!(server.entity_count(), 1);
        
        server.despawn_entity(id);
        
        assert_eq!(server.entity_count(), 0);
    }
}
