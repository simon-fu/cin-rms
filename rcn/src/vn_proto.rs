use std::{fmt, net::{Ipv4Addr, IpAddr}, marker::PhantomData};

use anyhow::{Result, bail, Context};
use bytes::{Buf, BufMut};
use num_enum::TryFromPrimitive;

use crate::utils::common::{EnumHexU16, EnumNum};

pub const HEADER_LENGTH: usize = 12;

#[allow(non_camel_case_types)]
#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq,)]
#[derive(TryFromPrimitive)]
pub enum MCodeType {
    HEARTBEAT                = 0xffff,
    REGISTER                 = 0xff01,
    REGISTER_ACK             = 0xff02,
    CNISUP                   = 0xff03,
    CNISUP_ACK               = 0xff04,
    
    REQUESTCHANNEL           = 0x1,
    REQUESTCHANNEL_ACK       = 0x2,
    PLAY                     = 0x3,
    PLAY_ACK                 = 0x4,
    COLLECTDIGIT             = 0x5,
    COLLECTDIGIT_ACK         = 0x6,
    RECORD                   = 0x7,
    RECORD_ACK               = 0x8,
    SENDFAX                  = 0x9,
    SENDFAX_ACK              = 0xa,
    RECEIVEFAX               = 0xb,
    RECEIVEFAX_ACK           = 0xc,
    OPENRTPCONNECT           = 0xd,
    OPENRTPCONNECT_ACK       = 0xe,
    SETRTPCONNECT            = 0xf,
    SETRTPCONNECT_ACK        = 0x10,
    CLOSERTPCONNECT          = 0x11,
    CLOSERTPCONNECT_ACK      = 0x12,
    CANCEL                   = 0x13,
    RELEASECHANNEL           = 0x14,
    FAXEVENT                 = 0x16,
    AUDIODETECT              = 0x17,
    AUDIODETECT_ACK          = 0x18,
    DTMFRCV                  = 0x19,
    DTMFRCV_ACK              = 0x1a,
    GET3PARTYPORT            = 0x1b,
    GET3PARTYPORT_ACK        = 0x1c,
    BRIDGE                   = 0x1d,
    BRIDGE_ACK               = 0x1e,
    HTTPDOWNLOAD             = 0x1f,
    THEARTBEAT               = 0x20,
    UNBRIDGE                 = 0x21,
    RESETLIFETIMER           = 0x22,
    INFODTMF                 = 0x23,
    NBUPINFO                 = 0x24, 
    MODIFYCHANNEL            = 0x25,
    MODIFYCHANNEL_ACK        = 0x26, 
    ADDVIDEO_ACK             = 0x27, 
    ERASEVIDEO_ACK           = 0x28, 
    OPENRTMPCONNECT           = 0x29,
    OPENRTMPCONNECT_ACK       = 0x2a,
    CLOSERTMPCONNECT          = 0x2b,
    CLOSERTMPCONNECT_ACK      = 0x2c,
    FACERECOG                 = 0x2d,
    FACERECOG_ACK             = 0x2e,
    RESFROMTAG                = 0x2f,
    AGORASUBSCRIBE           = 0x30, 
    AGORAUNSUBSCRIBE         = 0x31, 
    IVRMSGNAMELISTLENGTH      = 0x32,
}

impl MCodeType {
    pub fn code(&self) -> u16 {
        *self as u16
    }
}


pub type MCode = EnumHexU16<MCodeType>;


#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq,)]
#[derive(TryFromPrimitive)]
pub enum TagType {
    MEDIAINFO               = 0x01,
    FILENAME                = 0x02,
    RTPINFO                 = 0x06,
}

impl TagType {
    pub fn code(&self) -> u8 {
        *self as u8
    }
}


pub struct PacketRef<'a> {
    data: &'a [u8],
}

impl<'a> PacketRef<'a> {
    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < HEADER_LENGTH {
            bail!("data too short, [{}]", data.len())
        }

        let mut buf = data;

        let length = buf.get_u16() as usize;
 
        if length > buf.len() {
            bail!("too large field.length, expect [{}] but [{}]", buf.len(), length)
        }

        Ok(Self{data})
    }

    pub fn length(&self) -> usize {
        (&self.data[0..]).get_u16() as usize
    }

    pub fn code(&self) -> u16 {
        (&self.data[2..]).get_u16()
    }

    pub fn fsm_id(&self) -> u32 {
        (&self.data[4..]).get_u32()
    }

    pub fn key(&self) -> i16 {
        (&self.data[8..]).get_i16()
    }

    pub fn sn(&self) -> u16 {
        (&self.data[10..]).get_u16()
    }

    pub fn payload(&self) -> &'a [u8] {
        let len = self.length();
        &self.data[HEADER_LENGTH..len+2]
    }

    pub fn cn_path_data(&self) -> &'a [u8] {
        let len = self.length();
        &self.data[len..]
    }

    pub fn cn_path_utf8(&self) -> Result<&'a str> {
        let s = std::str::from_utf8(self.cn_path_data())?;
        Ok(s)
    }

    pub fn to_header(&self) -> Header {
        Header {
            // length: self.length(),
            code: self.code(),
            fsm_id: self.fsm_id(),
            key: self.key(),
            sn: self.sn(),
        }
    }
}

impl<'a> fmt::Debug for PacketRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("Packet");
        
        // builder.field("length", &self.length());

        // match self.mcode() {
        //     Some(mcode) => builder.field("code", &mcode),
        //     None => builder.field("code", &format_args!("0x{:02X}", self.code())),
        // };
        
        builder
        .field("length", &self.length())
        .field("code", &MCode::new(self.code()))
        .field("fsm_id", &self.fsm_id())
        .field("key", &self.key())
        .field("sn", &self.sn())
        .field("payload", &self.payload().len())
        .finish()
    }
}

#[derive(Default)]
pub struct Header {
    // pub length: usize,  // 2 bytes
    pub code: u16,      // 2 bytes
    pub fsm_id: u32,    // 4 bytes
    pub key: i16,       // 2 bytes
    pub sn: u16,        // 2 bytes
}

impl Header {
    pub fn write_to<B: BufMut>(&self, buf: B) -> usize {
        let empty: [u8; 0] = [];
        self.write_to2(buf, &empty[..])
    }

    pub fn write_to2<B1: BufMut, B2: Buf>(&self, mut buf: B1, payload: B2) -> usize {
        let len = HEADER_LENGTH + payload.remaining();
        buf.put_u16(len as u16 - 2);
        buf.put_u16(self.code);
        buf.put_u32(self.fsm_id);
        buf.put_i16(self.key);
        buf.put_u16(self.sn);
        buf.put(payload);
        len 
    }
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("Header");
        builder
        .field("code", &format_args!("{:02X?}", MCode::new(self.code)))
        .field("fsm_id", &self.fsm_id)
        .field("key", &self.key)
        .field("sn", &self.sn)
        .finish()
    }
}


#[derive(Debug)]
pub struct RegisterRef<'a> {
    pub ip: Ipv4Addr,      // 2 bytes
    pub media_info: MediaInfoRef<'a>,
}

impl<'a> RegisterRef<'a> {
    const MIN_LEN: usize = 4 + MediaInfoRef::MIN_LEN;

    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < Self::MIN_LEN {
            bail!("Register at least [{}] bytes but [{}]", Self::MIN_LEN, data.len())
        }

        let tag = TagRef::parse_from(&data[4..])?;
        if tag.tag_type() != Some(TagType::MEDIAINFO) {
            bail!("Register expect MEDIAINFO tag but [{:?}]", tag.tag_type() )
        }

        let (_n, media_info) = MediaInfoRef::parse_from(tag.payload())?;

        Ok(Self {
            ip: Ipv4Addr::new(data[0], data[1], data[2], data[3]),
            media_info,
        })
    }
}

#[derive(Debug)]
pub struct MediaInfoRef<'a> {
    pub support_t38: bool,
    pub audio_codecs: Vec<CodecDescRef<'a>>,
    pub video_codecs: Vec<CodecDescRef<'a>>,
    pub fax_codecs: Vec<CodecDescRef<'a>>,
}

impl<'a> MediaInfoRef<'a> {
    const MIN_LEN: usize = 4;

    pub fn parse_from(data: &'a [u8]) -> Result<(usize, Self)> {
        if data.len() < Self::MIN_LEN {
            bail!("Mediainfo at least [{}] bytes but [{}]", Self::MIN_LEN, data.len())
        }

        let mut buf = data;

        let support_t38 = buf.get_u8() != 1;

        let (parsed_len, audio_codecs) = CodecDescRef::parse_vec_from(buf)?;
        buf.advance(parsed_len);

        let (parsed_len, video_codecs) = CodecDescRef::parse_vec_from(buf)?;
        buf.advance(parsed_len);

        let (parsed_len, fax_codecs) = CodecDescRef::parse_vec_from(buf)?;
        buf.advance(parsed_len);

        Ok((data.len()-buf.len(), Self{
            support_t38, 
            audio_codecs,
            video_codecs,
            fax_codecs,
        }))
    }
}

pub struct CodecDescRef<'a> {
    index: u8,
    payload_type: u8,
    mapdata: &'a [u8],
}

impl<'a> CodecDescRef<'a> {
    pub fn parse_vec_from(data: &'a[u8]) -> Result<(usize, Vec<Self>)> {
        let mut buf = data;
        let count = buf.get_u8() as usize;
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            let (len, obj) = Self::parse_from(buf)?;
            v.push(obj);
            buf.advance(len);
        }
        Ok((data.len()-buf.len(), v))
    }

    pub fn parse_from(data: &'a [u8]) -> Result<(usize, Self)> {
        if data.len() < 3 {
            bail!("codec at least 3 bytes but [{}]", data.len())
        }

        let mut buf = data;

        let index = buf.get_u8();
        let payload_type = buf.get_u8();

        let pos = find_str_null(buf).with_context(||"Not found null for codec map str")?;

        Ok((pos + 3, Self{
            index, 
            payload_type,
            mapdata: &buf[..pos],
        }))
    }

    pub fn len(&self) -> usize {
        self.mapdata.len() + 3
    }

    pub fn index(&self) -> u8 {
        self.index
    }

    pub fn payload_type(&self) -> u8 {
        self.payload_type
    }

    pub fn map_str_data(&self) -> &'a [u8] {
        self.mapdata
    }

    pub fn map_str_utf8(&self) -> Option<&'a str> {
        std::str::from_utf8(self.mapdata).ok()
    }
}

impl<'a> fmt::Debug for CodecDescRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("CodecDesc");

        builder
        .field("index", &self.index)
        .field("payload_type", &self.payload_type);

        match self.map_str_utf8() {
            Some(v) => builder.field("mapstr", &v),
            None => builder.field("mapdata", &self.mapdata.len()),
        };
        
        builder.finish()
    }
}

#[derive(Clone)]
pub struct TagRef<'a> {
    tag: u8,
    payload: &'a [u8],
}

impl<'a> TagRef<'a> {
    const MIN_LEN: usize = 3;
    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < Self::MIN_LEN {
            bail!("tag at least 3 bytes but [{}]", data.len())
        }

        let mut buf = data;

        let tag = buf.get_u8();
        let length = buf.get_u16() as usize;
 
        if length > buf.len() {
            bail!("too large tag.length, expect [{}] but [{}]", buf.len(), length)
        }

        Ok(Self{
            tag, 
            payload: &data[Self::MIN_LEN..Self::MIN_LEN+length],
        })
    }

    pub fn tag_code(&self) -> u8 {
        self.tag
    }

    pub fn tag_type(&self) -> Option<TagType> {
        TagType::try_from(self.tag_code()).ok()
    }

    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

impl<'a> fmt::Debug for TagRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("Tag");

        match self.tag_type() {
            Some(v) => builder.field("type", &v),
            None => builder.field("type", &format_args!("0x{:02X}", self.tag_code())),
        };
        
        builder
        .field("payload", &self.payload().len())
        .finish()
    }
}


#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq,)]
#[derive(TryFromPrimitive)]
pub enum IceType {
    Simple = 0, // no stun，dtls，srtp
    Webrtc = 1, // has stun，dtls，srtp
    StunOnly = 2, // has stun, no dtls, srtp
}

pub type IceCode = EnumNum<u8, IceType>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq,)]
#[derive(TryFromPrimitive)]
pub enum MediaType {
    AudioOnly = 1, 
    AudioVideo = 2,
    Image = 3,
    Agora = 4,
    Rtmp = 8,
    TRtc = 9,
    TRtcVideo = 10,
    BRtc = 11,
    VideoOnly = 12,
    PRtc = 13,
}

pub type MediaCode = EnumNum<u8, MediaType>;


// #[derive(Debug)]
pub struct RequestChannelRef<'a> {
    fixed_part1: RequestChannelPart1<'a>,
    as_call_id: &'a [u8],
    agora_info: Option<&'a [u8]>,
    fixed_part2: RequestChannelPart2<'a>,
    webrtc: StrIter<'a>,
}

impl<'a> RequestChannelRef<'a> {
    const PART1_LEN: usize = 4;
    const PART2_LEN: usize = 6;
    const MIN_LEN: usize = Self::PART1_LEN + 1 + Self::PART2_LEN + 1;

    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < Self::MIN_LEN {
            bail!("RequestChannel at least [{}] bytes but [{}]", Self::MIN_LEN, data.len())
        }

        let mut buf = data;

        let fixed_part1 = RequestChannelPart1(&buf[..Self::PART1_LEN]);
        buf.advance(Self::PART1_LEN);

        let pos = find_str_null(buf).with_context(||"Not found null for as_call_id")?;
        let as_call_id = &buf[..pos];
        buf.advance(pos+1);

        
        let agora_info = match fixed_part1.media_type_code() {
            4 | 7 => {
                let pos = find_str_null(buf).with_context(||"Not found null for agora_info")?;
                let info = &buf[..pos];
                buf.advance(pos+1);
                Some(info)
            },
            _ => None,
        };

        if buf.len() < Self::PART2_LEN {
            bail!("RequestChannel part2 at least [{}] bytes but [{}]", Self::PART2_LEN, buf.len())
        }

        let fixed_part2 = RequestChannelPart2(&buf[..Self::PART2_LEN]);
        buf.advance(Self::PART2_LEN);

        let webrtc = StrIter(&buf[..]);
        buf.advance(buf.len());
        
        Ok(Self {
            fixed_part1,
            as_call_id,
            agora_info,
            fixed_part2,
            webrtc,
        })
    }

    pub fn part1<'b>(&'b self) -> &'b RequestChannelPart1<'a> {
        &self.fixed_part1
    }

    pub fn part2<'b>(&'b self) -> &'b RequestChannelPart2<'a> {
        &self.fixed_part2
    }
}

impl<'a> fmt::Debug for RequestChannelRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("RequestChannel");

        builder
        .field("ice", &IceCode::new(self.part1().ice_type_code()))
        .field("life", &self.part1().life_seconds())
        .field("ice", &MediaCode::new(self.part1().media_type_code()));

        fmt_struct_field_str(&mut builder, "as_call_id", self.as_call_id);

        match &self.agora_info {
            Some(info) => {
                fmt_struct_field_str(&mut builder, "agora_info", *info)
            },
            None => builder.field("agora_info", &Option::<&str>::None),
        };

        builder
        .field("is_nbup", &self.part2().is_nbup())
        .field("ptime", &self.part2().ptime())
        .field("is_caller", &self.part2().is_caller())
        .field("codec", &self.part2().codec_code())
        .field("amr_mode", &self.part2().amr_mode())
        // .field("redirect", &self.part2().redirect_code())
        // .field("ip_type", &self.part2().ip_type_code())
        ;

        builder.field("webrtc", &self.webrtc);
        
        builder.finish()
    }
}

pub struct RequestChannelPart1<'a>(&'a [u8]);
impl<'a> RequestChannelPart1<'a> {
    fn ice_type_code(&self) -> u8 {
        self.0[0]
    }

    fn life_seconds(&self) -> u16 {
        (&self.0[1..3]).get_u16()
    }

    fn media_type_code(&self) -> u8 {
        self.0[3]
    }
}

pub struct RequestChannelPart2<'a>(&'a [u8]);
impl<'a> RequestChannelPart2<'a> {
    pub fn is_nbup(&self) -> bool {
        self.0[0] != 0
    }

    pub fn ptime(&self) -> u8 {
        self.0[1]
    }

    pub fn is_caller(&self) -> bool {
        self.0[2] != 0
    }

    pub fn codec_code(&self) -> u8 {
        self.0[3]
    }

    pub fn amr_mode(&self) -> u16 {
        (&self.0[4..6]).get_u16()
    }
}



pub struct RequestChannelAckRef<'a> {
    fixed_part1: RequestChannelAckPart1<'a>,
    webrtc: StrIter<'a>,
}

impl<'a> RequestChannelAckRef<'a> {
    const PART1_LEN: usize = 8;
    const MIN_LEN: usize = Self::PART1_LEN + 1;

    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < Self::MIN_LEN {
            bail!("RequestChannelAck packet at least [{}] bytes but [{}]", Self::MIN_LEN, data.len())
        }

        let mut buf = data;

        let fixed_part1 = RequestChannelAckPart1(&buf[..Self::PART1_LEN]);
        buf.advance(Self::PART1_LEN);


        let webrtc = StrIter(&buf[..]);
        buf.advance(buf.len());
        
        Ok(Self {
            fixed_part1,
            webrtc,
        })
    }

    pub fn part1<'b>(&'b self) -> &'b RequestChannelAckPart1<'a> {
        &self.fixed_part1
    }

}

impl<'a> fmt::Debug for RequestChannelAckRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("RequestChannelAck");

        builder
        .field("result", &self.part1().result())
        .field("audio_port", &self.part1().audio_port())
        .field("video_port", &self.part1().video_port())
        .field("fax_port", &self.part1().fax_port())
        .field("media_type", &self.part1().media_type())
        ;

        builder.field("webrtc", &self.webrtc);
        
        builder.finish()
    }
}


pub struct RequestChannelAckPart1<'a>(&'a [u8]);
impl<'a> RequestChannelAckPart1<'a> {
    pub fn result(&self) -> u8 {
        self.0[0]
    }

    pub fn audio_port(&self) -> u16 {
        (&self.0[1..3]).get_u16()
    }

    pub fn video_port(&self) -> u16 {
        (&self.0[3..5]).get_u16()
    }

    pub fn fax_port(&self) -> u16 {
        (&self.0[5..7]).get_u16()
    }

    pub fn media_type(&self) -> u8 {
        self.0[7]
    }
}


pub struct OpenRtpConnectRef<'a> {
    num_tags: u8,
    tag_iter: TagIter<'a>,
}

impl<'a> OpenRtpConnectRef<'a> {
    const MIN_LEN: usize = 1;

    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < Self::MIN_LEN {
            bail!("OpenRtpConnect at least [{}] bytes but [{}]", Self::MIN_LEN, data.len())
        }

        Ok(Self {
            num_tags: data[0],
            tag_iter: TagIter(&data[1..]),
        })
    }

    pub fn rtpinfo_iter(&self) -> impl Iterator<Item = Result<RtpInfoRef<'a>>> + Clone {
        self.tag_iter.clone().map(|x| {
            match x {
                Ok(tag) => {
                    if tag.tag_code() != TagType::RTPINFO.code() {
                        bail!("expect tag [{:?}] but [{:?}]", TagType::RTPINFO, tag.tag_code())
                    }
                    RtpInfoRef::parse_from(tag.payload())
                },
                Err(e) => Err(e),
            }
        })
    }
}


impl<'a> fmt::Debug for OpenRtpConnectRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("OpenRtpConnect");

        builder
        .field("num", &self.num_tags);


        builder.field("rtpinfos", &ResultIterDebug::new(self.rtpinfo_iter()));
        
        builder.finish()
    }
}



macro_rules! define_u8_packet {
    ($type_name:ident) => {
        #[derive(Debug)]
        pub struct $type_name(u8);

        impl $type_name {
            const MIN_LEN: usize = 1;

            pub fn parse_from(data: & [u8]) -> Result<Self> {
                if data.len() < Self::MIN_LEN {
                    bail!("{} at least [{}] bytes but [{}]", stringify!($func_name), Self::MIN_LEN, data.len())
                }
                Ok(Self(data[0]))
            }

            pub fn value(&self) -> u8 {
                self.0
            }
        }
    };
}

define_u8_packet!(OpenRtpConnectAck);

define_u8_packet!(CloseRtpConnect);

define_u8_packet!(CloseRtpConnectAck);

pub struct ResFromTagRef<'a>(&'a [u8]);


impl<'a> ResFromTagRef<'a> {
    const MIN_LEN: usize = 1;

    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < Self::MIN_LEN {
            bail!("ResFromTag at least [{}] bytes but [{}]", Self::MIN_LEN, data.len())
        }

        let mut buf = data;

        let pos = find_str_null(buf).with_context(||"Not found null for ResFromTag")?;
        let slice = &buf[..pos];
        buf.advance(pos+1);

        Ok(Self(slice))
    }
}


impl<'a> fmt::Debug for ResFromTagRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_tuple("ResFromTag");
        
        match std::str::from_utf8(self.0) {
            Ok(v) => builder.field(&v),
            Err(e) => builder.field(&Result::<(), std::str::Utf8Error>::Err(e)),
        };
        
        builder.finish()
    }
}

pub struct PlayRef<'a> {
    part1: PlayPart1<'a>,
    tags: TagIter<'a>,
}

impl<'a> PlayRef<'a> {
    const PART1_LEN: usize = 16;
    const MIN_LEN: usize = Self::PART1_LEN;

    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < Self::MIN_LEN {
            bail!("Play at least [{}] bytes but [{}]", Self::MIN_LEN, data.len())
        }

        let mut buf = data;

        let part1 = PlayPart1(&buf[..Self::PART1_LEN]);
        buf.advance(Self::PART1_LEN);

        let tags = TagIter(buf);
        buf.advance(buf.len());

        Ok(Self{
            part1,
            tags,
        })
    }
}

impl<'a> fmt::Debug for PlayRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("Play");
        builder
        .field("interval", &self.part1.interval())
        .field("play_times", &self.part1.play_times())
        .field("max_duration", &self.part1.max_duration())
        .field("key_mask", &self.part1.key_mask())
        .field("record", &self.part1.record())
        .field("speech_barge", &self.part1.speech_barge())
        .field("erase_dtmf", &self.part1.erase_dtmf())
        .field("num_tlv", &self.part1.num_tlv())
        .field("tags", &TagIterDebug(self.tags.clone()))
        ;
        
        builder.finish()
    }
}


pub struct PlayPart1<'a>(&'a [u8]);

impl<'a> PlayPart1<'a> {
    pub fn interval(&self) -> u32 {
        (&self.0[0..4]).get_u32()
    }

    pub fn play_times(&self) -> u16 {
        (&self.0[4..6]).get_u16()
    }

    pub fn max_duration(&self) -> u32 {
        (&self.0[6..10]).get_u32()
    }

    pub fn key_mask(&self) -> u16 {
        (&self.0[10..12]).get_u16()
    }

    pub fn record(&self) -> bool {
        self.0[12] != 0
    }

    pub fn speech_barge(&self) -> bool {
        self.0[13] != 0
    }

    pub fn erase_dtmf(&self) -> bool {
        self.0[14] != 0
    }

    pub fn num_tlv(&self) -> u8 {
        self.0[15]
    }
}






pub struct PlayAckRef<'a> {
    part1: PlayAckPart1<'a>,
    tags: TagIter<'a>,
}

impl<'a> PlayAckRef<'a> {
    const PART1_LEN: usize = 5;
    const MIN_LEN: usize = Self::PART1_LEN;

    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < Self::MIN_LEN {
            bail!("PlayAck at least [{}] bytes but [{}]", Self::MIN_LEN, data.len())
        }

        let mut buf = data;

        let part1 = PlayAckPart1(&buf[..Self::PART1_LEN]);
        buf.advance(Self::PART1_LEN);

        let tags = TagIter(buf);
        buf.advance(buf.len());

        Ok(Self{
            part1,
            tags,
        })
    }
}

impl<'a> fmt::Debug for PlayAckRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("PlayAck");
        builder
        .field("result", &self.part1.result())
        .field("play_duration", &self.part1.play_duration())
        .field("tags", &TagIterDebug(self.tags.clone()))
        ;
        
        builder.finish()
    }
}


pub struct PlayAckPart1<'a>(&'a [u8]);

impl<'a> PlayAckPart1<'a> {
    pub fn result(&self) -> u8 {
        self.0[0]
    }

    pub fn play_duration(&self) -> u32 {
        (&self.0[1..5]).get_u32()
    }
}







#[derive(Debug)]
pub struct FilenameRef<'a> {
    format: u8,
    filename: StrRef<'a>,
}

impl<'a> FilenameRef<'a> {
    const MIN_LEN: usize = 1;

    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < Self::MIN_LEN {
            bail!("Filename at least [{}] bytes but [{}]", Self::MIN_LEN, data.len())
        }

        let mut buf = data;

        let format = buf.get_u8();

        let (_n, filename) = StrRef::from_str_null(&buf[..])
        .with_context(||"Not found null for filename")?;
        buf.advance(buf.len());

        Ok(Self{
            format,
            filename,
        })
    }

    pub fn format(&self) -> u8 {
        self.format
    }

    pub fn filename(&self) -> &StrRef<'a> {
        &self.filename
    }
}


pub struct CancelRef<'a>(&'a [u8]);

impl<'a> CancelRef<'a> {
    const MIN_LEN: usize = 2;

    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < Self::MIN_LEN {
            bail!("Cancel at least [{}] bytes but [{}]", Self::MIN_LEN, data.len())
        }
        Ok(Self(&data[..Self::MIN_LEN]))
    }

    pub fn op_code(&self) -> u16 {
        (&self.0[0..2]).get_u16()
    }
}

impl<'a> fmt::Debug for CancelRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Cancel")
        .field(&MCode::new(self.op_code()))
        .finish()
    }
}


struct ResultIterDebug<I, T, E>(I, PhantomData<T>, PhantomData<E>);

impl<I, T, E> ResultIterDebug<I, T, E> {
    pub fn new(iter: I) -> Self {
        Self(iter, Default::default(), Default::default())
    }
}

impl<I, T, E> fmt::Debug for ResultIterDebug<I, T, E> 
where
    I: Iterator<Item = Result<T, E>> + Clone,
    T: fmt::Debug,
    E: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_list();
        for r in self.0.clone() {
            match r {
                Ok(v) => builder.entry(&v),
                Err(e) => builder.entry(&Result::<(), E>::Err(e)),
            };
        }
        builder.finish()
    }
}


#[derive(Clone)]
pub struct TagIter<'a>(&'a [u8]);

impl<'a> Iterator for TagIter<'a> {
    type Item = Result<TagRef<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
           return None 
        }

        match TagRef::parse_from(self.0) {
            Ok(tag) => {
                self.0.advance(tag.payload().len() + TagRef::MIN_LEN);
                Some(Ok(tag))
            },
            Err(e) => {
                self.0.advance(self.0.len());
                Some(Err(e.into()))
            },
        }
    }
}

impl<'a> fmt::Debug for TagIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_list();
        for r in self.clone() {
            match r {
                Ok(v) => builder.entry(&v),
                Err(e) => builder.entry(&Result::<()>::Err(e)),
            };
        }
        builder.finish()
    }
}


#[derive(Clone)]
pub struct TagIterDebug<'a>(TagIter<'a>);

impl<'a> fmt::Debug for TagIterDebug<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_list();
        for r in self.0.clone() {
            match r {
                Ok(v) => builder.entry(&TagDebug(v)),
                Err(e) => builder.entry(&Result::<()>::Err(e)),
            };
        }
        builder.finish()
    }
}

#[derive(Clone)]
pub struct TagDebug<'a>(TagRef<'a>);

impl<'a> fmt::Debug for TagDebug<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        match self.0.tag_type() {
            None => {
                fmt::Debug::fmt(&self.0, f)
            },
            Some(ttype) => {
                let mut builder = f.debug_struct("Tag");
                builder.field("type", &ttype);
                match ttype {
                    TagType::MEDIAINFO => builder.field("value", &MediaInfoRef::parse_from(self.0.payload())),
                    TagType::FILENAME => builder.field("value", &FilenameRef::parse_from(self.0.payload())),
                    TagType::RTPINFO => builder.field("value", &RtpInfoRef::parse_from(self.0.payload())),
                };

                builder.finish()
            },
        }
    }
}


#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq,)]
#[derive(TryFromPrimitive)]
pub enum RtpMediaType {
    Audio = 0, 
    Video = 1,
    T38 = 2,
}

pub type RtpMediaTypeCode = EnumNum<u8, RtpMediaType>;

pub struct RtpInfoRef<'a> {
    fixed_part1: RtpInfoPart1<'a>,
    attribute: &'a [u8],
    fixed_part2: RtpInfoPart2<'a>,
    part3: StrIter<'a>,
}

impl<'a> RtpInfoRef<'a> {
    const PART1_LEN: usize = 9;
    const PART2_LEN: usize = 2;
    const MIN_LEN: usize = Self::PART1_LEN + 1 + Self::PART2_LEN + 6;

    pub fn parse_from(data: &'a [u8]) -> Result<Self> {
        if data.len() < Self::MIN_LEN {
            bail!("RtpInfo at least [{}] bytes but [{}]", Self::MIN_LEN, data.len())
        }

        let mut buf = data;

        let fixed_part1 = RtpInfoPart1(&buf[..Self::PART1_LEN]);
        buf.advance(Self::PART1_LEN);

        let pos = find_str_null(buf).with_context(||"Not found null for as_call_id")?;
        let attribute = &buf[..pos];
        buf.advance(pos+1);


        if buf.len() < Self::PART2_LEN {
            bail!("RtpInfo part2 at least [{}] bytes but [{}]", Self::PART2_LEN, buf.len())
        }

        let fixed_part2 = RtpInfoPart2(&buf[..Self::PART2_LEN]);
        buf.advance(Self::PART2_LEN);


        let part3 = StrIter(&buf[..]);
        buf.advance(buf.len());
        
        Ok(Self {
            fixed_part1,
            attribute,
            fixed_part2,
            part3,
        })
    }

    pub fn part1<'b>(&'b self) -> &'b RtpInfoPart1<'a> {
        &self.fixed_part1
    }

    pub fn part2<'b>(&'b self) -> &'b RtpInfoPart2<'a> {
        &self.fixed_part2
    }
}

impl<'a> fmt::Debug for RtpInfoRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("RtpInfo");

        builder
        .field("ip", &self.part1().ip())
        .field("port", &self.part1().port())
        .field("media_type", &RtpMediaTypeCode::new(self.part1().media_type()))
        .field("internal_pltyp", &self.part1().internal_pltyp())
        .field("nego_pltyp", &self.part1().nego_pltyp())
        ;

        fmt_struct_field_str(&mut builder, "attribute", self.attribute);


        builder
        .field("tele_event", &self.part2().tele_event())
        .field("direction", &self.part2().direction())
        ;

        builder.field("desc", &self.part3);
        
        builder.finish()
    }
}


pub struct RtpInfoPart1<'a>(&'a [u8]);
impl<'a> RtpInfoPart1<'a> {
    pub fn ip(&self) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(self.0[0], self.0[1], self.0[2], self.0[3]))
    }

    pub fn port(&self) -> u16 {
        (&self.0[4..6]).get_u16()
    }

    pub fn media_type(&self) -> u8 {
        self.0[6]
    }

    pub fn internal_pltyp(&self) -> u8 {
        self.0[7]
    }

    pub fn nego_pltyp(&self) -> u8 {
        self.0[8]
    }
}

pub struct RtpInfoPart2<'a>(&'a [u8]);
impl<'a> RtpInfoPart2<'a> {
    pub fn tele_event(&self) -> u8 {
        self.0[0]
    }

    pub fn direction(&self) -> u8 {
        self.0[1]
    }
}

// #[derive(Clone)]
struct StrIter<'a>(&'a [u8]);

impl<'a> Iterator for StrIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None
        }

        match find_str_null(self.0) {
            Some(pos) => {
                let s = &self.0[..pos];
                self.0 = &self.0[pos+1..];
                Some(s)
            },
            None => {
                self.0 = &self.0[self.0.len()..];
                None
            },
        }
    }
}

impl<'a> fmt::Debug for StrIter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_list();
        for data in Self(self.0) {
            match std::str::from_utf8(data) {
                Ok(v) => builder.entry(&v),
                Err(e) => builder.entry(&Result::<(), std::str::Utf8Error>::Err(e)),
            };
        }
        builder.finish()
    }
}

#[derive(Clone)]
pub struct StrRef<'a>(&'a [u8]);

impl<'a> StrRef<'a> {
    pub fn from_str_null(buf: &'a [u8]) -> Option<(usize, Self)> {
        let r = find_str_null(buf);
        match r {
            Some(pos) => {
                let me = Self(&buf[..pos]);
                Some((pos+1, me))
            },
            None => None,
        }
    }
}

impl<'a> fmt::Debug for StrRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match std::str::from_utf8(self.0) {
            Ok(v) => fmt::Debug::fmt(&v, f),
            Err(e) => fmt::Debug::fmt(&e, f),
        }
    }
}


fn fmt_struct_field_str<'a, 'b, 'c>(builder: &'a mut fmt::DebugStruct<'b, 'c>, name: &str, data: &[u8]) -> &'a mut fmt::DebugStruct<'b, 'c> {

    match std::str::from_utf8(data) {
        Ok(v) => builder.field(name, &v),
        Err(e) => builder.field(name, &Result::<(), std::str::Utf8Error>::Err(e)),
    };

    builder
}

fn find_str_null(buf: &[u8]) -> Option<usize> {
    buf.iter().position(|x|*x==0)
}


