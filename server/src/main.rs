use std::net::TcpListener;
use std::net::TcpStream;
use std::io::{Read, Write};
fn main() -> std::io::Result<()>{
       let listener = TcpListener::bind("127.0.0.1:7000")?;
    //fd = socket
    //bind(fd, socketaddr *, sizeof(serveraddr *))
    //listen(fd)
    println!("server socket bind listen 127.0.0.1:7000");
    loop { 
        //accept loop
        // listener.accept() {}
        match listener.accept() {
            Ok((mut client_socket, addr)) => {
                println!("client {addr} connected to server");
                loop {
                    let mut bytes = [0u8, 128];
                    let res = client_socket.read(&mut bytes)?;
                    print!("{}", String::from_utf8_lossy(&bytes[0.. res as usize]));
                    if res > 0 {
                        client_socket.write(&bytes[0..res as usize])?;
                    } else {
                        break;
                    }
                }
            },
            Err(e) => {eprintln!("failed: {}", e)},
        }
    }

}
