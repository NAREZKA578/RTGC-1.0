use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use tracing;

#[derive(Clone)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
    active_jobs: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, &'static str> {
        if size == 0 {
            return Err("Thread pool size must be greater than zero");
        }

        let (sender, receiver) = crossbeam_channel::unbounded();
        let receiver = Arc::new(Mutex::new(receiver));
        let active_jobs = Arc::new(AtomicUsize::new(0));
        let shutdown = Arc::new(AtomicBool::new(false));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                Arc::clone(&active_jobs),
                Arc::clone(&shutdown),
            ));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
            active_jobs,
            shutdown,
        })
    }

    pub fn execute<F>(&self, f: F) -> JoinHandle
    where
        F: FnOnce() + Send + 'static,
    {
        self.active_jobs.fetch_add(1, Ordering::SeqCst);
        let (job_sender, job_receiver) = crossbeam_channel::bounded(1);
        let job = Box::new(move || {
            f();
            let _ = job_sender.send(());
        });

        if let Some(sender) = self.sender.as_ref() {
            if let Err(e) = sender.send(job) {
                tracing::error!("Failed to send job to thread pool: {}", e);
                self.active_jobs.fetch_sub(1, Ordering::SeqCst);
                return JoinHandle {
                    receiver: job_receiver,
                    active_jobs: None,
                };
            }
        } else {
            tracing::warn!("Thread pool is shut down, cannot execute job");
            self.active_jobs.fetch_sub(1, Ordering::SeqCst);
            return JoinHandle {
                receiver: job_receiver,
                active_jobs: None,
            };
        }

        JoinHandle {
            receiver: job_receiver,
            active_jobs: Some(Arc::downgrade(&self.active_jobs)),
        }
    }

    pub fn wait_all(&self) {
        while self.active_jobs.load(Ordering::SeqCst) > 0 {
            thread::yield_now();
        }
    }
}

pub struct JoinHandle {
    receiver: Receiver<()>,
    active_jobs: Option<std::sync::Weak<AtomicUsize>>,
}

impl JoinHandle {
    pub fn join(self) -> Result<(), &'static str> {
        self.receiver.recv().map_err(|_| "Failed to join task")
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Clone for Worker {
    fn clone(&self) -> Self {
        Self { thread: None }
    }
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<Receiver<Job>>>,
        active_jobs: Arc<AtomicUsize>,
        shutdown: Arc<AtomicBool>,
    ) -> Worker {
        let thread_result = thread::Builder::new()
            .name(format!("physics-worker-{}", id))
            .spawn(move || loop {
                // Сначала проверяем флаг shutdown
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                // Пробуем получить задачу с таймаутом, чтобы периодически проверять shutdown
                let job = receiver
                    .lock()
                    .recv_timeout(std::time::Duration::from_millis(100));

                match job {
                    Ok(job) => {
                        job();
                        active_jobs.fetch_sub(1, Ordering::SeqCst);
                    }
                    Err(_) => {
                        // Таймаут или ошибка - продолжаем цикл и проверяем shutdown
                        continue;
                    }
                }
            });

        match thread_result {
            Ok(thread) => Worker { thread: Some(thread) },
            Err(_) => Worker { thread: None },
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Устанавливаем флаг shutdown
        self.shutdown.store(true, Ordering::SeqCst);

        // Закрываем sender, чтобы все recv() вернули ошибку
        self.sender.take();

        // Ждём завершения всех активных задач с таймаутом
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();
        while self.active_jobs.load(Ordering::SeqCst) > 0 && start.elapsed() < timeout {
            thread::yield_now();
        }

        // Если задачи всё ещё есть, логируем предупреждение
        let remaining = self.active_jobs.load(Ordering::SeqCst);
        if remaining > 0 {
            tracing::warn!(
                "ThreadPool dropping with {} active jobs remaining (timeout)",
                remaining
            );
        }

        // Соединяем все worker потоки
        for worker in &mut self.workers {
            if let Some(handle) = worker.thread.take() {
                let _ = handle.join();
            }
        }

        tracing::info!("ThreadPool shut down gracefully");
    }
}
