
use time::{UtcOffset, macros::format_description};

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt::{time::OffsetTime, MakeWriter}};

pub(crate) fn init_log() {
    init_log2(env!("CARGO_PKG_NAME"), ||std::io::stdout())
}

pub(crate) fn init_log2<W2>(name: &str, w: W2) 
where
    W2: for<'writer> MakeWriter<'writer> + 'static + Send + Sync,
{

    // https://time-rs.github.io/book/api/format-description.html
    let fmts = format_description!("[hour]:[minute]:[second].[subsecond digits:3]");

    let offset = UtcOffset::current_local_offset().expect("should get local offset!");
    let timer = OffsetTime::new(offset, fmts);
    
    let filter = if cfg!(debug_assertions) {
        if let Ok(v) = std::env::var(EnvFilter::DEFAULT_ENV) {
            v.into()
        } else {
            format!("{name}=debug").into()
            // "rcn=debug".into()
            // "debug".into()
        }
    } else {
        EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
    };
        
    tracing_subscriber::fmt()
    .with_max_level(tracing::metadata::LevelFilter::DEBUG)
    .with_env_filter(filter)
    // .with_env_filter("rtun=debug,rserver=debug")
    .with_writer(w)
    .with_timer(timer)
    .with_target(false)
    .init();
}
