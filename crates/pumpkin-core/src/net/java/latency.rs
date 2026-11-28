use tokio::net::TcpStream;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawSocket, AsSocket, OwnedSocket};

/// A socket reference retained for querying the kernel's TCP RTT estimate.
pub struct TcpLatency {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    socket: OwnedFd,
    #[cfg(windows)]
    socket: OwnedSocket,
}

impl TcpLatency {
    pub(crate) fn new(stream: &TcpStream) -> Option<Self> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            Some(Self {
                socket: stream.as_fd().try_clone_to_owned().ok()?,
            })
        }
        #[cfg(windows)]
        {
            Some(Self {
                socket: stream.as_socket().try_clone_to_owned().ok()?,
            })
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            let _ = stream;
            None
        }
    }

    pub(crate) fn get(&self) -> Option<u32> {
        #[cfg(target_os = "linux")]
        type TcpInfo = libc::tcp_info;
        #[cfg(target_os = "macos")]
        type TcpInfo = libc::tcp_connection_info;

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            #[cfg(target_os = "linux")]
            const OPTION: libc::c_int = libc::TCP_INFO;
            #[cfg(target_os = "macos")]
            const OPTION: libc::c_int = libc::TCP_CONNECTION_INFO;

            #[cfg(target_os = "linux")]
            const RTT_END: usize =
                std::mem::offset_of!(TcpInfo, tcpi_rtt) + std::mem::size_of::<u32>();
            #[cfg(target_os = "macos")]
            const RTT_END: usize =
                std::mem::offset_of!(TcpInfo, tcpi_srtt) + std::mem::size_of::<u32>();

            let mut info = std::mem::MaybeUninit::<TcpInfo>::zeroed();
            let mut len = std::mem::size_of::<TcpInfo>() as libc::socklen_t;
            // SAFETY: the owned socket is live, and info/len describe a writable TcpInfo buffer.
            let result = unsafe {
                libc::getsockopt(
                    self.socket.as_raw_fd(),
                    libc::IPPROTO_TCP,
                    OPTION,
                    info.as_mut_ptr().cast(),
                    &raw mut len,
                )
            };
            if result != 0 || (len as usize) < RTT_END {
                return None;
            }
            // SAFETY: the integer-only struct was zeroed, and getsockopt returned the RTT field.
            let info = unsafe { info.assume_init() };
            #[cfg(target_os = "linux")]
            {
                Some(info.tcpi_rtt / 1_000)
            } // Linux reports microseconds.
            #[cfg(target_os = "macos")]
            {
                Some(info.tcpi_srtt)
            } // Darwin reports milliseconds.
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::Networking::WinSock::{SIO_TCP_INFO, TCP_INFO_v0, WSAIoctl};
            let version = 0_u32;
            let mut info = std::mem::MaybeUninit::<TCP_INFO_v0>::zeroed();
            let mut len = 0_u32;
            // SAFETY: all buffers are live and correctly sized; null OVERLAPPED makes this synchronous.
            let result = unsafe {
                WSAIoctl(
                    self.socket.as_raw_socket() as _,
                    SIO_TCP_INFO,
                    (&raw const version).cast(),
                    std::mem::size_of_val(&version) as u32,
                    info.as_mut_ptr().cast(),
                    std::mem::size_of::<TCP_INFO_v0>() as u32,
                    &raw mut len,
                    std::ptr::null_mut(),
                    None,
                )
            };
            if result != 0 || (len as usize) < std::mem::size_of::<TCP_INFO_v0>() {
                return None;
            }
            // SAFETY: WSAIoctl successfully initialized the complete buffer above.
            Some(unsafe { info.assume_init() }.RttUs / 1_000)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        None
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos", windows)))]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    #[tokio::test]
    async fn tcp_rtt_remains_available_after_stream_is_split() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap())
            .await
            .unwrap();
        let (server, _) = listener.accept().await.unwrap();
        let latency = TcpLatency::new(&server).unwrap();
        let (mut read, _) = server.into_split();
        let (_, mut write) = client.into_split();
        write.write_all(b"ping").await.unwrap();
        let mut packet = [0; 4];
        read.read_exact(&mut packet).await.unwrap();
        assert_eq!(&packet, b"ping");
        assert!(
            latency.get().is_some(),
            "connected TCP socket has no RTT estimate"
        );
    }
}
