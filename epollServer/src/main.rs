use std::os::unix::io::{AsRawFd, RawFd};
use std::io;
use std::io::prelude::*;
use std::io::{Read, Write, Error};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;

// const HTTP_RESP: &[u8] = b"HTTP/1.1 200 OK
// content-type: text/html
// content-length: 5

// Hello";

#[allow(unused_macros)]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

const READ_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLIN;
// const WRITE_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLOUT;

fn epoll_create() -> io::Result<RawFd> {
    let fd = syscall!(epoll_create1(0))?;
    if let Ok(flags) = syscall!(fcntl(fd, libc::F_GETFD)) {
        let _ = syscall!(fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC));
    }

    Ok(fd)
}

fn listener_read_event(key: u64) -> libc::epoll_event {
    libc::epoll_event {
        events: READ_FLAGS as u32,
        u64: key,
    }
}

// fn listener_write_event(key: u64) -> libc::epoll_event {
//     libc::epoll_event {
//         events: WRITE_FLAGS as u32,
//         u64: key,
//     }
// }

fn add_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event))?;
    Ok(())
}

fn modify_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_MOD, fd, &mut event))?;
    Ok(())
}

fn close(fd: RawFd) {
    let _ = syscall!(close(fd));
}

fn remove_interest(epoll_fd: RawFd, fd: RawFd) -> io::Result<()> {
    syscall!(epoll_ctl(
        epoll_fd,
        libc::EPOLL_CTL_DEL,
        fd,
        std::ptr::null_mut()
    ))?;
    Ok(())
}

pub struct RequestContext {
    pub stream: TcpStream,
    pub content_length: usize,
    pub buf: Vec<u8>,
}

impl RequestContext {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buf: Vec::new(),
            content_length: 0,
        }
    }

    fn read_cb(&mut self, key: u64, epoll_fd: RawFd, buf: &mut [u8]) -> Result<usize, Error> {
        //clean out buffer
        //write in it
        // let mut buf = [0u8; 4096];
        // match self.stream.read(&mut buf) {
        //     Ok(_) => {
        //         if let Ok(data) = std::str::from_utf8(&buf) {
        //             // self.parse_and_set_content_length(data);
        //         }
        //     }
        //     Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
        //     Err(e) => {
        //         return Err(e);
        //     }
        // };
        let res = self.stream.read(buf)?;
        if res > 0 {
            // stream.write(&bytes[..res as usize])?;
            // println!("client {} connected to server", );
            print!("{}> {}", self.stream.peer_addr()?, String::from_utf8_lossy(&buf[0.. res as usize]));
            return Ok(res as usize);
        } else {
            return Ok(res as usize);
        }
        self.buf.extend_from_slice(&buf);
        if self.buf.len() >= self.content_length {
            println!("got all data: {} bytes", self.buf.len());
            // modify_interest(epoll_fd, self.stream.as_raw_fd(), listener_write_event(key))?;
            // self.write_resp(100, epoll_fd, &buf[..]);
        } else {
            modify_interest(epoll_fd, self.stream.as_raw_fd(), listener_read_event(key))?;
        }
        Ok(res as usize)
    }

    // fn parse_and_set_content_length(&mut self, data: &str) {
    //     if data.contains("HTTP") {
    //         if let Some(content_length) = data
    //             .lines()
    //             .find(|l| l.to_lowercase().starts_with("content-length: "))
    //         {
    //             if let Some(len) = content_length
    //                 .to_lowercase()
    //                 .strip_prefix("content-length: ")
    //             {
    //                 self.content_length = len.parse::<usize>().expect("content-length is valid");
    //                 println!("set content length: {} bytes", self.content_length);
    //             }
    //         }
    //     }
    // }

    // fn write_cb(&mut self, key: u64, epoll_fd: RawFd) -> io::Result<()> {
    //     //read from buffer
    //     //write to client
    //     match self.stream.write(b"connected to server") {
    //         Ok(_) => println!("answered from request {}", key),
    //         Err(e) => eprintln!("could not answer to request {}, {}", key, e),
    //     };
    //     // self.stream.shutdown(std::net::Shutdown::Both)?;
    //     // let fd = self.stream.as_raw_fd();
    //     // remove_interest(epoll_fd, fd)?;
    //     // close(fd);
    //     Ok(())
    // }

    fn write_resp(&mut self, key: u64, epoll_fd: RawFd, resp: &[u8]) -> io::Result<()> {
        println!("{}> {}", self.stream.peer_addr()?, String::from_utf8_lossy(resp));
        self.stream.write(resp)?;
        Ok(())
    }
}


fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7000")?;
    listener.set_nonblocking(true)?;
    let listener_fd = listener.as_raw_fd();

    let epoll_fd = epoll_create().expect("can create epoll queue");
    let mut key = 100;

    add_interest(epoll_fd, listener_fd, listener_read_event(key))?;

    let mut events: Vec<libc::epoll_event> = Vec::with_capacity(1024);
    let mut request_contexts: HashMap<u64, RequestContext> = HashMap::new();
    //define buffer here;
    let mut bytes = [0u8; 128];
    loop {
        events.clear();
        let res = match syscall!(epoll_wait(
            epoll_fd,
            events.as_mut_ptr() as *mut libc::epoll_event,
            1024,
            1000 as libc::c_int,
        )) {
            Ok(v) => v,
            Err(e) => panic!("error during epoll wait: {}", e),
        };
        // safe as long as the kernel does nothing wrong - copied from mio
        unsafe { events.set_len(res as usize) };
        println!("requests in flight: {}", request_contexts.len());
        for ev in &events {
            match ev.u64 {
                100 => {
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            stream.set_nonblocking(true)?;
                            println!("new client: {}", addr);
                            key += 1;
                            add_interest(epoll_fd, stream.as_raw_fd(), listener_read_event(key))?;
                            request_contexts.insert(key, RequestContext::new(stream));
                        }
                        Err(e) => eprintln!("couldn't accept: {}", e),
                    };
                    modify_interest(epoll_fd, listener_fd, listener_read_event(100))?;
                    // add_interest(epoll_fd, listener_fd, listener_read_event(key))?;
                }
                key => {
                    let mut to_delete = None;
                    if let Some(context) = request_contexts.get_mut(&key) {
                        let events: u32 = ev.events;
                        match events {
                            v if v as i32 & libc::EPOLLIN == libc::EPOLLIN => {
                                let res = context.read_cb(key, epoll_fd, &mut bytes)?;
                                context.write_resp(key, epoll_fd, &bytes[..res as usize]);
                                to_delete = Some(key);
                                // add_interest(epoll_fd, listener_fd, listener_read_event(key))?;

                            }
                            v => println!("unexpected events: {}", v),
                        };
                    }
                    if let Some(key) = to_delete {
                        request_contexts.remove(&key);
                    }
                }
            }
        }
    }
    Ok(())
}