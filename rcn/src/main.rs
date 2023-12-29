
use anyhow::Result;
use clap::Parser;


pub mod utils;
pub mod vn_proto;
pub mod vn_unix_socket;
pub mod subcmd_decvn;

fn main() -> Result<()> {
    utils::log::init_log();
    let args = CmdArgs::parse();
    match &args.cmd {
        SubCmd::Decvn(sub) => subcmd_decvn::run(&sub),
        SubCmd::Cli(sub) => {
            tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(rcn::run(sub))
        },
    }
}

#[derive(Parser, Debug)]
#[clap(name = "rcn", author, about, version)]
struct CmdArgs {
    #[clap(subcommand)]
    cmd: SubCmd,
}

#[derive(Parser, Debug)]
enum SubCmd {
    Decvn(subcmd_decvn::CmdArgs),
    Cli(rcn::CmdArgs),
}



// async fn async_main() -> Result<()> {
//     tracing::debug!("hello");
//     rcn::run().await?;
//     Ok(())
// }

mod rcn {
    use std::{path::Path, fmt::Write};

    use anyhow::{Result, Context, bail};
    use clap::Parser;
    use tokio::net::UnixDatagram;
    use tracing::debug;

    use crate::vn_proto::{Header, MCodeType, PacketRef, RegisterRef};

    #[derive(Parser, Debug)]
    #[clap(name = "cli", author, about, version)]
    pub struct CmdArgs {

    }
    
    pub async fn run(_args: &CmdArgs) -> Result<()> {

        let cindir = std::env::var(CINDIR).with_context(||"can't get env [{CINDIR}]")?;
        let cindir_path: &Path = cindir.as_ref();

        let cn_id = 5_u32;
        
        let mut cn_socket_path = cindir_path.join("mscn");
        write!(cn_socket_path.as_mut_os_string(), "{cn_id}")?;
        tokio::fs::remove_file(&cn_socket_path).await.with_context(||format!("failed to remove unix socket path [{cn_socket_path:?}]"))?;
        let socket = UnixDatagram::bind(&cn_socket_path)
        .with_context(||format!("can't bind unix socket path [{cn_socket_path:?}]"))?;

        let ms_socket_path = cindir_path.join("msvn");
        let mut send_buf = vec![0_u8; 1700];
        let mut recv_buf = vec![0_u8; 1700];

        {
            let header = Header {
                code: MCodeType::CNISUP.code(),
                fsm_id: cn_id * 1000000,
                ..Default::default()
            };
            let len = header.write_to(&mut send_buf[..]);
            socket.send_to(&send_buf[..len], &ms_socket_path).await.with_context(||"sendto failed")?;
            debug!("header={header:?}");
            debug!("sent to [{ms_socket_path:?}], bytes [{len}]");
    
            let (recv_len, from) = socket.recv_from(&mut recv_buf).await.with_context(||"recvfrom failed")?;
            debug!("recv from [{from:?}], bytes [{recv_len}]");
            let packet = PacketRef::parse_from(&recv_buf[..recv_len]).with_context(||"parse packet failed")?;
            debug!("  {packet:?}");

            if packet.code() != MCodeType::CNISUP_ACK.code() {
                bail!("expect CNISUP_ACK but [{:?}]",packet.code())
            }
        }

        {
            let (recv_len, from) = socket.recv_from(&mut recv_buf).await.with_context(||"recvfrom failed")?;
            debug!("recv from [{from:?}], bytes [{recv_len}]");
            let packet = PacketRef::parse_from(&recv_buf[..recv_len]).with_context(||"parse packet failed")?;
            debug!("  {packet:?}");

            if packet.code() != MCodeType::REGISTER.code() {
                bail!("expect CNISUP_ACK but [{:?}]", packet.code())
            }

            let reg = RegisterRef::parse_from(packet.payload()).with_context(||"parse register packet failed")?;
            debug!("  {reg:?}");


            let header = Header {
                code: MCodeType::REGISTER_ACK.code(),
                fsm_id: cn_id * 1000000,
                ..Default::default()
            };
            let len = header.write_to2(&mut send_buf[..], &[0][..]);
            socket.send_to(&send_buf[..len], &ms_socket_path).await.with_context(||"sendto failed")?;
            debug!("header={header:?}");
            debug!("sent to [{ms_socket_path:?}], bytes [{len}]");

        }

        loop {
            let (recv_len, from) = socket.recv_from(&mut recv_buf).await.with_context(||"recvfrom failed")?;
            debug!("recv from [{from:?}], bytes [{recv_len}]");
            let packet = PacketRef::parse_from(&recv_buf[..recv_len]).with_context(||"parse packet failed")?;
            debug!("  {packet:?}");
        }
        
        // Ok(())
    }

    const CINDIR: &str = "CINDIR";
}





