//! ECS (Entity Component System) Module for RTGC-0.8
//! Provides entity management, component storage, and system execution

pub mod ecs_module;
pub mod job_system;

pub use ecs_module::EcsManager;
pub use job_system::{JobSystem, Job, JobPriority, JobResult, JobSystemConfig, JobSystemStats, ParallelJob, RangeJob, EcsParallelUpdateSystem};
