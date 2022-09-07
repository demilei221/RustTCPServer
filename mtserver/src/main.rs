use std::net::TcpListener;
use std::net::TcpStream;
use std::io::{Read, Write, Error};
use std::thread;

fn handle_client(mut stream: TcpStream) -> Result<(), Error> {
    println!("client {} connected to server", stream.peer_addr()?);
    let mut bytes = [0u8, 128];
    loop {
        let res = stream.read(&mut bytes)?;
        if res > 0 {
            stream.write(&bytes[..res as usize])?;
            // println!("client {} connected to server", );
            print!("{}> {}", stream.peer_addr()?, String::from_utf8_lossy(&bytes[0.. res as usize]));
        } else {
            return Ok(());
        }
    }
}


fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:7000")?;
    println!("server socket bind listen 127.0.0.1:7000");
    for stream in listener.incoming() {
        match stream {
            Err(e) => {eprintln!("failed: {}", e)}
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
        }
    }
    Ok(())
}
