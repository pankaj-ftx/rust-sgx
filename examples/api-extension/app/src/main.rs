#![feature(sgx_platform)]
//use std::os::fortanix_sgx::usercalls::raw::connect_stream;
//use std::os::fortanix_sgx::io::FromRawFd::*;
use std::io::Write;
fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
    let name = "hello_world.txt";
    let message = "Hello, world!";
    let mut s  = std::net::TcpStream::connect(name)?;

    
    s.write(message.as_bytes())?;
    s.flush()?;
    Ok(())
}
