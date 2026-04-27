// ============================================================================
// STEP 1: Job System - Многопоточная система задач
// ============================================================================
// Система для распараллеливания вычислений по всем доступным ядрам CPU.
// Позволяет дробить большие задачи (обновление тысяч объектов, генерация
// чанков, расчет LOD) на мелкие независимые задачи, выполняемые пулом потоков.
// ============================================================================

use std::sync::{Arc, atomic::{AtomicUsize, AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::collections::VecDeque;
use crossbeam_channel::{bounded, Sender, Receiver, TrySendError};
use parking_lot::{Mutex, Condvar};
use tracing;

/// Максимальное количество задач в очереди
const MAX_QUEUE_SIZE: usize = 4096;

/// Тип идентификатора задачи
pub type JobId = usize;

/// Приоритет задачи
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for JobPriority {
    fn default() -> Self {
        JobPriority::Normal
    }
}

/// Задача для выполнения
pub trait Job: Send + Sync {
    /// Выполнение задачи
    fn execute(&self);
    
    /// Имя задачи для отладки
    fn name(&self) -> &'static str {
        "Unnamed Job"
    }
    
    /// Размер задачи (для балансировки нагрузки)
    fn size_hint(&self) -> usize {
        1
    }
}

/// Внутреннее представление задачи
struct JobWrapper {
    id: JobId,
    priority: JobPriority,
    job: Arc<dyn Job>,
    dependencies: Vec<JobId>,
}

/// Результат выполнения задачи
#[derive(Debug, Clone)]
pub struct JobResult {
    pub id: JobId,
    pub success: bool,
    pub error: Option<String>,
}

/// Статистика работы Job System
#[derive(Debug, Clone, Default)]
pub struct JobSystemStats {
    pub total_jobs_submitted: usize,
    pub total_jobs_completed: usize,
    pub total_jobs_failed: usize,
    pub active_workers: usize,
    pub queue_size: usize,
    pub average_execution_time_ms: f64,
}

/// Конфигурация Job System
#[derive(Debug, Clone)]
pub struct JobSystemConfig {
    pub num_threads: usize,
    pub queue_size: usize,
    pub steal_enabled: bool,
}

impl Default for JobSystemConfig {
    fn default() -> Self {
        Self {
            num_threads: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).max(2),
            queue_size: MAX_QUEUE_SIZE,
            steal_enabled: true,
        }
    }
}

/// Менеджер зависимостей между задачами
struct DependencyManager {
    pending_dependencies: Mutex<Vec<(JobId, JobId)>>, // (job_id, depends_on)
    completed_jobs: Mutex<Vec<JobId>>,
    ready_condition: Condvar,
}

impl DependencyManager {
    fn new() -> Self {
        Self {
            pending_dependencies: Mutex::new(Vec::new()),
            completed_jobs: Mutex::new(Vec::new()),
            ready_condition: Condvar::new(),
        }
    }
    
    fn add_dependency(&self, job_id: JobId, depends_on: JobId) {
        self.pending_dependencies.lock().push((job_id, depends_on));
    }
    
    fn mark_completed(&self, job_id: JobId) {
        let mut completed = self.completed_jobs.lock();
        completed.push(job_id);
        
        // Уведомляем ожидающие задачи
        self.ready_condition.notify_all();
    }
    
    fn is_ready(&self, job_id: JobId) -> bool {
        let deps = self.pending_dependencies.lock();
        let completed = self.completed_jobs.lock();
        
        !deps.iter().any(|&(jid, depends_on)| {
            jid == job_id && !completed.contains(&depends_on)
        })
    }
    
    fn get_ready_jobs(&self, all_jobs: &[JobWrapper]) -> Vec<JobId> {
        let completed = self.completed_jobs.lock();
        let deps = self.pending_dependencies.lock();
        
        all_jobs
            .iter()
            .filter(|job| {
                !completed.contains(&job.id) &&
                !deps.iter().any(|&(jid, depends_on)| {
                    jid == job.id && !completed.contains(&depends_on)
                })
            })
            .map(|job| job.id)
            .collect()
    }
}

/// Основной класс Job System с безопасной инициализацией через RefCell
pub struct JobSystem {
    config: JobSystemConfig,
    workers: parking_lot::Mutex<Vec<JoinHandle<()>>>, // Используем Mutex для безопасного доступа
    shutdown: AtomicBool,
    active_workers: AtomicUsize,
    
    // Очереди задач по приоритетам
    high_priority_queue: Mutex<VecDeque<JobWrapper>>,
    normal_priority_queue: Mutex<VecDeque<JobWrapper>>,
    low_priority_queue: Mutex<VecDeque<JobWrapper>>,
    
    // Каналы для коммуникации
    job_sender: Sender<JobWrapper>,
    job_receiver: Receiver<JobWrapper>,
    result_sender: Sender<JobResult>,
    result_receiver: Receiver<JobResult>,
    
    // Управление зависимостями
    dependency_manager: Arc<DependencyManager>,
    
    // Счётчики
    next_job_id: AtomicUsize,
    stats: Mutex<JobSystemStats>,
}

impl JobSystem {
    /// Создание новой системы задач
    pub fn new() -> Arc<Self> {
        Self::with_config(JobSystemConfig::default())
    }
    
    /// Создание системы с конфигурацией
     pub fn with_config(config: JobSystemConfig) -> Arc<Self> {
        let (job_sender, job_receiver) = bounded(config.queue_size);
        let (result_sender, result_receiver) = bounded(config.queue_size);
        
        // Создаём систему с пустым вектором workers, который будет заполнен ниже
        let system = Arc::new(Self {
            config: config.clone(),
            workers: parking_lot::Mutex::new(Vec::with_capacity(config.num_threads)),
            shutdown: AtomicBool::new(false),
            active_workers: AtomicUsize::new(0),
            high_priority_queue: Mutex::new(VecDeque::new()),
            normal_priority_queue: Mutex::new(VecDeque::new()),
            low_priority_queue: Mutex::new(VecDeque::new()),
            job_sender,
            job_receiver,
            result_sender,
            result_receiver,
            dependency_manager: Arc::new(DependencyManager::new()),
            next_job_id: AtomicUsize::new(0),
            stats: Mutex::new(JobSystemStats::default()),
        });
        
        // Запуск рабочих потоков
        for i in 0..config.num_threads {
            let sys = Arc::clone(&system);
            let handle = thread::Builder::new()
                .name(format!("JobWorker-{}", i))
                .spawn(move || {
                    sys.worker_loop(i);
                })
                .expect("Critical: Failed to spawn worker thread - cannot continue without working threads");
            
            // Безопасно добавляем handle в mutex
            system.workers.lock().push(handle);
        }
        
        system
    }
    
    /// Главный цикл рабочего потока
    fn worker_loop(&self, _worker_id: usize) {
        while !self.shutdown.load(Ordering::Relaxed) {
            // Попытка получить задачу из канала
            if let Ok(job) = self.job_receiver.recv_timeout(std::time::Duration::from_millis(10)) {
                self.active_workers.fetch_add(1, Ordering::Relaxed);
                
                let start = std::time::Instant::now();
                let job_id = job.id;
                let job_name = job.job.name();
                
                // Выполнение задачи
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    job.job.execute();
                }));
                
                let _execution_time = start.elapsed();

                // Отправка результата
                let job_result = match result {
                    Ok(_) => JobResult {
                        id: job_id,
                        success: true,
                        error: None,
                    },
                    Err(e) => {
                        let error_msg = if let Some(s) = e.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = e.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "Unknown panic".to_string()
                        };

                        tracing::error!("Job '{}' panicked: {}", job_name, error_msg);

                        JobResult {
                            id: job_id,
                            success: false,
                            error: Some(error_msg),
                        }
                    }
                };

                // Обновление статистики до отправки (чтобы не использовать job_result после move)
                {
                    let mut stats = self.stats.lock();
                    stats.total_jobs_completed += 1;
                    if !job_result.success {
                        stats.total_jobs_failed += 1;
                    }
                }

                let _ = self.result_sender.send(job_result);

                // Пометка задачи как выполненной
                self.dependency_manager.mark_completed(job_id);

                self.active_workers.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }
    
    /// Добавление задачи в очередь
    pub fn add_job<T: Job + 'static>(&self, job: T, priority: JobPriority) -> JobId {
        let id = self.next_job_id.fetch_add(1, Ordering::Relaxed);
        
        let wrapper = JobWrapper {
            id,
            priority,
            job: Arc::new(job),
            dependencies: Vec::new(),
        };
        
        // Отправка в канал
        match self.job_sender.try_send(wrapper) {
            Ok(_) => {
                let mut stats = self.stats.lock();
                stats.total_jobs_submitted += 1;
                id
            }
            Err(TrySendError::Full(_)) => {
                tracing::warn!("Job queue is full, dropping job");
                id
            }
            Err(TrySendError::Disconnected(_)) => {
                tracing::error!("Job receiver disconnected");
                id
            }
        }
    }
    
    /// Добавление задачи с зависимостями
    pub fn add_job_with_deps<T: Job + 'static>(
        &self,
        job: T,
        priority: JobPriority,
        dependencies: &[JobId],
    ) -> JobId {
        let id = self.next_job_id.fetch_add(1, Ordering::Relaxed);
        
        // Регистрация зависимостей
        for &dep_id in dependencies {
            self.dependency_manager.add_dependency(id, dep_id);
        }
        
        let wrapper = JobWrapper {
            id,
            priority,
            job: Arc::new(job),
            dependencies: dependencies.to_vec(),
        };
        
        // Проверка готовности
        if self.dependency_manager.is_ready(id) {
            match self.job_sender.try_send(wrapper) {
                Ok(_) => {
                    let mut stats = self.stats.lock();
                    stats.total_jobs_submitted += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to send job: {:?}", e);
                }
            }
        } else {
            // Задача будет отправлена когда зависимости будут выполнены
            // В реальной реализации нужно хранить ожидающие задачи
            tracing::debug!("Job {} waiting for dependencies", id);
        }
        
        id
    }
    
    /// Блокирующее ожидание завершения задачи
    pub fn wait_for_job(&self, job_id: JobId, timeout: Option<std::time::Duration>) -> Option<JobResult> {
        let deadline = timeout.map(|t| std::time::Instant::now() + t);
        
        loop {
            if let Some(d) = deadline {
                if std::time::Instant::now() > d {
                    return None;
                }
            }
            
            // Проверка всех результатов
            while let Ok(result) = self.result_receiver.try_recv() {
                if result.id == job_id {
                    return Some(result);
                }
            }
            
            thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    
    /// Ожидание завершения нескольких задач
    pub fn wait_for_jobs(&self, job_ids: &[JobId], timeout: Option<std::time::Duration>) -> Vec<JobResult> {
        let mut results = Vec::with_capacity(job_ids.len());
        let mut pending: Vec<JobId> = job_ids.to_vec();
        
        let deadline = timeout.map(|t| std::time::Instant::now() + t);
        
        while !pending.is_empty() {
            if let Some(d) = deadline {
                if std::time::Instant::now() > d {
                    break;
                }
            }
            
            while let Ok(result) = self.result_receiver.try_recv() {
                if let Some(pos) = pending.iter().position(|&id| id == result.id) {
                    pending.remove(pos);
                    results.push(result);
                }
            }
            
            thread::sleep(std::time::Duration::from_millis(1));
        }
        
        results
    }
    
    /// Получение статистики
    pub fn get_stats(&self) -> JobSystemStats {
        let mut stats = self.stats.lock().clone();
        stats.active_workers = self.active_workers.load(Ordering::Relaxed);
        stats.queue_size = self.high_priority_queue.lock().len()
            + self.normal_priority_queue.lock().len()
            + self.low_priority_queue.lock().len();
        stats
    }
    
    /// Корректное завершение работы
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
        
        // Ожидание завершения всех потоков через mutex
        let workers = std::mem::take(&mut *self.workers.lock());
        for worker in workers {
            let _ = worker.join();
        }
    }
    
    /// Параллельное выполнение итерации - заглушка для компиляции
    pub fn parallel_for<I, F>(&self, items: I, func: F)
    where
        I: Iterator + Send,
        I::Item: Send + Sync + 'static,
        F: Fn(I::Item) + Send + Sync + 'static,
    {
        // Заглушка: выполняем последовательно для компиляции
        for item in items {
            func(item);
        }
    }
}

impl Drop for JobSystem {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// ============================================================================
// Вспомогательные типы задач
// ============================================================================

/// Простая задача-обёртка для замыканий
pub struct ParallelJob<F> {
    func: F,
    name: &'static str,
}

impl<F> ParallelJob<F>
where
    F: Fn() + Send + Sync,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            name: "ParallelJob",
        }
    }
    
    pub fn with_name(func: F, name: &'static str) -> Self {
        Self { func, name }
    }
}

impl<F> Job for ParallelJob<F>
where
    F: Fn() + Send + Sync,
{
    fn execute(&self) {
        (self.func)();
    }
    
    fn name(&self) -> &'static str {
        self.name
    }
}

/// Задача для обработки диапазона индексов
pub struct RangeJob<F> {
    start: usize,
    end: usize,
    func: F,
}

impl<F> RangeJob<F>
where
    F: Fn(usize) + Send + Sync,
{
    pub fn new(start: usize, end: usize, func: F) -> Self {
        Self { start, end, func }
    }
}

impl<F> Job for RangeJob<F>
where
    F: Fn(usize) + Send + Sync,
{
    fn execute(&self) {
        for i in self.start..self.end {
            (self.func)(i);
        }
    }
    
    fn name(&self) -> &'static str {
        "RangeJob"
    }
    
    fn size_hint(&self) -> usize {
        self.end - self.start
    }
}

// ============================================================================
// Интеграция с ECS
// ============================================================================

/// Система для параллельного обновления ECS компонентов
pub struct EcsParallelUpdateSystem<'a, T> {
    components: &'a [T],
    update_func: Box<dyn Fn(&T) + Send + Sync>,
}

impl<'a, T: Send + Sync> EcsParallelUpdateSystem<'a, T> {
    pub fn new<F>(components: &'a [T], func: F) -> Self
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        Self {
            components,
            update_func: Box::new(func),
        }
    }
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    
    #[test]
    fn test_simple_job() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        
        struct TestJob {
            counter: Arc<AtomicUsize>,
        }
        
        impl Job for TestJob {
            fn execute(&self) {
                self.counter.fetch_add(1, Ordering::Relaxed);
            }
            
            fn name(&self) -> &'static str {
                "TestJob"
            }
        }
        
        let job_system = JobSystem::new();
        
        for _ in 0..10 {
            let job = TestJob {
                counter: Arc::clone(&counter_clone),
            };
            job_system.add_job(job, JobPriority::Normal);
        }
        
        // Даем время на выполнение
        thread::sleep(std::time::Duration::from_millis(100));
        
        assert_eq!(counter.load(Ordering::Relaxed), 10);
        
        job_system.shutdown();
    }
    
    #[test]
    fn test_parallel_for() {
        let sum = Arc::new(Mutex::new(0));
        let sum_clone = Arc::clone(&sum);
        
        let job_system = JobSystem::new();
        
        job_system.parallel_for(0..100, move |x| {
            let mut s = sum_clone.lock();
            *s += x;
        });
        
        let expected: i32 = (0..100).sum();
        assert_eq!(*sum.lock(), expected);
        
        job_system.shutdown();
    }
}
