use core::time;
use env_logger::Builder;
use log::LevelFilter;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

static ADDR: &str = "0.0.0.0:8080";
fn main() {
    env_logger::init();
    //sync_main();
    async_main();
}
fn sync_main() {
    let listener = TcpListener::bind(ADDR).expect("Unable to create listener.");
    log::info!("Listening on {}", listener.local_addr().unwrap());
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // séquentiel
                //handle(stream);
                // multi-threaded
                std::thread::spawn(move || handle(stream));
            }
            Err(e) => {
                println!("Connection failed : {}", e)
            }
        }
    }
}

fn async_main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();

    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(ADDR).await.unwrap();
        log::info!("Listening on {}", listener.local_addr().unwrap());

        loop {
            if let Ok((sock, addr)) = listener.accept().await {
                log::debug!("Accepted connection from {}", addr);

                // Instead of spawning a thread, we spawn a task.
                rt.spawn(async move { async_handle(sock).await });
            }
        }
    });
}

fn handle(mut stream: TcpStream) {
    println!("====================================");
    log::info!(
        "Connection established from {:?}",
        stream.peer_addr().unwrap()
    );
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    log::debug!("Request : {:#?}", http_request);
    let response = format!("{}{}", "HTTP/1.1 200 OK\r\n\r\n", "plop");

    log::debug!("Response: {}", &response);
    let ten_millis = time::Duration::from_millis(1000);

    thread::sleep(ten_millis);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

/// Suck everything out of a socket and spit it back.
async fn async_handle(mut stream: tokio::net::TcpStream) -> std::io::Result<()> {
    let mut buf = Vec::new();
    let n_read = stream.read_to_end(&mut buf).await?;
    log::debug!("Client read {} bytes.", n_read);

    log::debug!("received {:?}", String::from_utf8_lossy(&buf[0..10]));
    stream.write_all(&buf[..n_read]).await?;
    stream.flush().await?;
    stream.shutdown().await?;
    log::debug!("Server responded and shut down.");

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    const MESSAGE_SIZE: usize = 50;
    #[tokio::test]
    async fn test_connect_async() {
        let mut handles = vec![];
        for _ in 1..10 {
            handles.push(tokio::spawn(async move {
                let mut stream = TcpStream::connect(ADDR).unwrap();

                // Buffer the bytes
                let _bytes_written = stream.write(b"Hello\r\n\r\n").unwrap();
                // Tell TCP to send the buffered data on the wire
                stream.flush().unwrap();

                log::info!("send");
                // Array with a fixed size
                let mut rx_bytes = [0u8; MESSAGE_SIZE];
                // Read from the current data in the TcpStream
                stream.read(&mut rx_bytes);

                let received = std::str::from_utf8(&rx_bytes).expect("valid utf8");
                println!("received : {:#?}", received);
            }));
        }
        futures::future::join_all(handles).await;
    }
    #[test]
    fn test_connect() {
        let mut stream = TcpStream::connect(ADDR).unwrap();

        // Buffer the bytes
        let _bytes_written = stream.write(b"Hello\r\n\r\n").unwrap();
        // Tell TCP to send the buffered data on the wire
        stream.flush().unwrap();
        //stream.shutdown(Shutdown::Both);
        log::info!("send");
    }
}
