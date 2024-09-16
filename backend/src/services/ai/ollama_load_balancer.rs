use ollama_rs::Ollama;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct OllamaLoadBalancer {
    servers: Vec<Arc<Mutex<Ollama>>>,
    thread_pool: ThreadPool,
    next_server: AtomicUsize,
}

impl OllamaLoadBalancer {
    pub fn new(server_ports: &[u16], threads_per_server: usize) -> Self {
        let servers: Vec<Arc<Mutex<Ollama>>> = server_ports
            .iter()
            .map(|&port| {
                Arc::new(Mutex::new(Ollama::new(
                    format!("http://localhost:{}", port),
                    port,
                )))
            })
            .collect();

        let total_threads = servers.len() * threads_per_server;
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(total_threads)
            .build()
            .expect("Failed to create thread pool");

        Self {
            servers,
            thread_pool,
            next_server: AtomicUsize::new(0),
        }
    }

    pub async fn get_server(&self) -> Arc<Mutex<Ollama>> {
        let index = self.next_server.fetch_add(1, Ordering::Relaxed) % self.servers.len();
        self.servers[index].clone()
    }

    pub fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        self.thread_pool.install(f)
    }
}
