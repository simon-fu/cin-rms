
// use std::path::Path;

// use anyhow::Result;
// use tokio::net::UnixDatagram;

// pub struct VnUnixSocket {
//     socket: UnixDatagram,
// }

// impl VnUnixSocket {
//     pub fn bind<P>(path: P) -> Result<Self>
//     where
//         P: AsRef<Path>,
//     {
//         let socket = UnixDatagram::bind(path)?;
//         Ok(Self {
//             socket,
//         })
//     }


// }
