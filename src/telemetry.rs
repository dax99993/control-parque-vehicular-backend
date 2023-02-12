use tokio::task::JoinHandle;

use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;

/// Compose multiple layers into `tracing`'s subscriber
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as return type to avoid having
/// to spell out the actual type of the return subscriber, which is 
/// indeed quite complex
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// lateron
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Sync + Send
    where 
        Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // Failling back to printing all logs at info-level or above
    // if the RUST_LOG environment variable has not been set
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(
        name,
        //Output the formatted layer to stoud
        sink
    );
    // The 'with' method is provided by 'SubscriberExt', an extension
    // trait for 'Subscriber' exposed by 'tracing_subscriber'
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Register all  `log`'s events to out subscriber
    LogTracer::init().expect("Failed to set logger");
    // 'set_global_default' can be used by applications to specify
    // what subscriber should be used by process spans.
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where 
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
