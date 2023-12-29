use bytes::{BufMut, BytesMut};
use clap::Parser;
use anyhow::{Result, Context};
use tracing::{debug, info, warn};
use std::io::{self, Read};

use crate::vn_proto::{PacketRef, MCodeType, RegisterRef, RequestChannelRef, OpenRtpConnectRef, RequestChannelAckRef, OpenRtpConnectAck, ResFromTagRef, PlayRef, CancelRef, CloseRtpConnect, CloseRtpConnectAck, PlayAckRef};

pub fn run(_args: &CmdArgs) -> Result<()> {
    info!("enter text and press ctrl+D when completed");
    
    
    let mut read_buf = Vec::new();
    {
        let stdin = io::stdin();
        let mut reader = stdin.lock();

        reader.read_to_end(&mut read_buf).with_context(||"read stdin failed")?;
    }
    let text = std::str::from_utf8(&read_buf[..]).with_context(||"invalid input text")?;
    decode_text(text)?;

    // let mut lines = Vec::new();
    // {
    //     let stdin = io::stdin();
    //     let mut reader = stdin.lock().lines();
    
    //     while let Some(r) = reader.next() {
    //         let line = r.with_context(||"read stdin failed")?;
    //         lines.push(line);
    //     }
    // }
    // debug!("--------");

    // decode_lines(lines.iter().map(|x|x.as_str()))?;
    
    debug!("done");

    Ok(())
}

fn decode_text(text: &str) -> Result<()> {
    decode_lines(text.lines())
}

fn decode_lines<'a, I>(lines: I) -> Result<()> 
where
    I: Iterator<Item = &'a str>
{
    let mut bin_buf = BytesMut::new();
    {
        for line in lines {
            // debug!("line=[{line:?}]");
            let line = line.trim();
            if !line.is_empty() {
                parse_line(&line, &mut bin_buf)?;
            }
        }
        // debug!("--------");
    }
    

    let data = &bin_buf[..];
    debug!("parsed length [{}]", bin_buf.len());
    debug!("parsed content {data:02x?}");

    
    let packet = PacketRef::parse_from(&bin_buf[..]).with_context(||"invalid packet")?;
    print_packet(&packet)?;
    Ok(())
}

fn print_packet(packet: &PacketRef<'_>) -> Result<()> {
    info!("{packet:?}");

    let r = MCodeType::try_from(packet.code()).ok();
    if let Some(code_type) = r {
        match code_type {
            MCodeType::REGISTER => {
                let r = RegisterRef::parse_from(packet.payload()).with_context(||"invalid Register packet")?;
                info!("{r:#?}");
            }
            MCodeType::REQUESTCHANNEL => {
                let r = RequestChannelRef::parse_from(packet.payload()).with_context(||"invalid RequestChannel packet")?;
                info!("{r:#?}");
            }
            MCodeType::REQUESTCHANNEL_ACK => {
                let r = RequestChannelAckRef::parse_from(packet.payload()).with_context(||"invalid RequestChannelAck packet")?;
                info!("{r:#?}");
            }
            MCodeType::OPENRTPCONNECT => {
                let r = OpenRtpConnectRef::parse_from(packet.payload()).with_context(||"invalid OpenRtpConnect packet")?;
                info!("{r:#?}");
            }
            MCodeType::OPENRTPCONNECT_ACK => {
                let r = OpenRtpConnectAck::parse_from(packet.payload()).with_context(||"invalid OpenRtpConnectAck packet")?;
                info!("{r:#?}");
            }
            MCodeType::RESFROMTAG => {
                let r = ResFromTagRef::parse_from(packet.payload()).with_context(||"invalid ResFromTag packet")?;
                info!("{r:#?}");
            }
            MCodeType::PLAY => {
                let r = PlayRef::parse_from(packet.payload()).with_context(||"invalid Play packet")?;
                info!("{r:#?}");
            }
            MCodeType::PLAY_ACK => {
                let r = PlayAckRef::parse_from(packet.payload()).with_context(||"invalid PlayAck packet")?;
                info!("{r:#?}");
            }
            MCodeType::CANCEL => {
                let r = CancelRef::parse_from(packet.payload()).with_context(||"invalid Cancel packet")?;
                info!("{r:#?}");
            }
            MCodeType::CLOSERTPCONNECT => {
                let r = CloseRtpConnect::parse_from(packet.payload()).with_context(||"invalid CloseRtpConnect packet")?;
                info!("{r:#?}");
            }
            MCodeType::CLOSERTPCONNECT_ACK => {
                let r = CloseRtpConnectAck::parse_from(packet.payload()).with_context(||"invalid CloseRtpConnectAck packet")?;
                info!("{r:#?}");
            }
            MCodeType::RELEASECHANNEL => {
                // no payload
            }
            
            _ => {
                warn!("Not imple code");
            }
        }
    } else {
        warn!("unknown code");
    }

    Ok(())
}

fn parse_line<B: BufMut>(line: &str, buf: &mut B) -> Result<u64> {
    let mut parts = line.split_whitespace();
    let offset = parts.next().with_context(||"no offset part")?;
    let offset: u64 = offset.parse().with_context(||format!("invalid offset [{offset}]"))?;
    
    for (index, part) in parts.enumerate() {
        if part.len() != 2 {
            break;
        }

        let r = u8::from_str_radix(part, 16);
        match r {
            Ok(v) => {
                buf.put_u8(v);
            },
            Err(e) => {
                if index == 2 {
                    break;
                } else {
                    return Err(e.into())
                }
            },
        }
    }

    Ok(offset)
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;

    use super::{parse_line, decode_text};

    #[test]
    fn poc() {
        tracing_subscriber::fmt()
        .with_max_level(tracing::metadata::LevelFilter::INFO)
        .with_target(false)
        .init();

        decode_text(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_vn_packet/REQUESTCHANNEL.txt"))).unwrap();

        decode_text(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_vn_packet/REQUESTCHANNEL_ACK.txt"))).unwrap();

        decode_text(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_vn_packet/OPENRTPCONNECT.txt"))).unwrap();

        decode_text(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_vn_packet/OPENRTPCONNECT_ACK.txt"))).unwrap();

        decode_text(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_vn_packet/RESFROMTAG.txt"))).unwrap();

        decode_text(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_vn_packet/PLAY.txt"))).unwrap();
        
        decode_text(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_vn_packet/CANCEL.txt"))).unwrap();

        decode_text(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_vn_packet/CLOSERTPCONNECT.txt"))).unwrap();

        decode_text(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_vn_packet/RELEASECHANNEL.txt"))).unwrap();

        decode_text(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_vn_packet/CLOSERTPCONNECT_ACK.txt"))).unwrap();

        decode_text(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_vn_packet/PLAY_ACK.txt"))).unwrap();

    }

    #[test]
    fn test_parse_line() {
        let mut buf = BytesMut::new();

        let offset = parse_line("0\t00 35 00 01 00 2d c6 c2  00 00 80 00 00 00 3c 01 \t.5...-........<.", &mut buf).unwrap();
        assert_eq!(offset, 0);
        assert_eq!(&buf.split()[..], &[
            0x00, 0x35, 0x00, 0x01, 0x00, 0x2d, 0xc6, 0xc2,  
            0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x3c, 0x01
        ][..]);

        let offset = parse_line("23\t00 35 00 01 00 2d c6 c2  00 00 80 00 00 00 3c 01 \t.5...-........<.", &mut buf).unwrap();
        assert_eq!(offset, 23);
        assert_eq!(&buf.split()[..], &[
            0x00, 0x35, 0x00, 0x01, 0x00, 0x2d, 0xc6, 0xc2,  
            0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x3c, 0x01
        ][..]);

        let offset = parse_line("12\t00 35       \t.5", &mut buf).unwrap();
        assert_eq!(offset, 12);
        assert_eq!(&buf.split()[..], &[
            0x00, 0x35
        ][..]);
    }

}


#[derive(Parser, Debug)]
#[clap(name = "decvn", author, about, version)]
pub struct CmdArgs {
}

