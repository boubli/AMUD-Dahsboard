#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let workers = std::env::var("AMUD_TOKIO_WORKERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()
        .expect("tokio runtime");
    rt.block_on(amud_server::run());
}
