use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

/// Compose multiple layers into a `tracing's` subscriber.
// If not to mark returned type as `Send + Sync`, I am not going to have this information
// when I'll be working with a return type of [`get_subscriber`]
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> Box<dyn Subscriber + Send + Sync>
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    // TODO: Can I play around with types here and write it a
    // bit nicer?

    // Bunyan log includes more information(for example host, pid, request-id from `tracing-actix-web`)
    // compared to `fmt`
    if std::env::var("BUNYAN_LOG").is_ok() {
        Box::new(
            Registry::default()
                .with(env_filter)
                .with(JsonStorageLayer)
                .with(BunyanFormattingLayer::new(name, sink)),
        )
    } else {
        Box::new(
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_writer(sink)
                .finish(),
        )
    }
}

/// Register a subscriber as global default to process span data.
///
/// Must be only called once
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
