
use anyhow::Result;
use time::{UtcOffset, macros::format_description};

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt::{time::OffsetTime, MakeWriter}};



#[tokio::main]
async fn main() -> Result<()> {
    init_log("rcn");
    tracing::debug!("hello");
    rcn::run().await?;
    Ok(())
}

mod rcn {
    use std::{path::Path, fmt::Write};

    use anyhow::{Result, Context};
    use tokio::net::UnixDatagram;

    pub async fn run() -> Result<()> {
        let cindir = std::env::var(CINDIR).with_context(||"can't get env [{CINDIR}]")?;
        let cindir_path: &Path = cindir.as_ref();

        let cn_id = 1;
        
        let mut cn_socket_path = cindir_path.join("mscn");
        write!(cn_socket_path.as_mut_os_string(), "{cn_id}")?;
        let socket = UnixDatagram::bind(&cn_socket_path)
        .with_context(||format!("can't bind unix socket path [{cn_socket_path:?}]"))?;

        let ms_socket_path = cindir_path.join("msvn");
        socket.send_to(b"123", &ms_socket_path).await?;
        
        Ok(())
    }

    const CINDIR: &str = "CINDIR";
}

pub(crate) fn init_log(name: &str) {
    init_log2(name, ||std::io::stdout())
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
        .with_default_directive(LevelFilter::DEBUG.into())
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

