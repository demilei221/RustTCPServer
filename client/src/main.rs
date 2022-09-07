use std::net::TcpStream;
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:7000";
    let mut client = TcpStream::connect(addr)?;

    let stdin = std::io::stdin();
    let mut input = String::new();
    loop {
        input.clear();
        print!("{addr}> ");
        std::io::stdout().flush()?;
        stdin.read_line(&mut input)?;

        client.write(input.as_bytes())?;
        loop {
            let mut bytes = [0u8; 128];
            let res = client.read(&mut bytes)?;
            if res > 0 {
                print!("{}", String::from_utf8_lossy(&bytes[0.. res as usize]));
            } 
            if res < bytes.len() {
                break;
            }
        }
        println!("");
    }
}
