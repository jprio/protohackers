use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use unescape::unescape;

static ADDR: &str = "0.0.0.0:8080";
fn main() {
    env_logger::init();
    async_main();
}

#[derive(Serialize, Deserialize)]
struct Request {
    method: String,
    number: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    method: String,
    prime: bool,
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

/// Suck everything out of a socket and spit it back.
async fn async_handle(stream: tokio::net::TcpStream) -> std::io::Result<()> {
    let mut line = String::new();
    let mut stream = BufReader::new(stream);

    stream.read_line(&mut line).await?;

    log::debug!("received {}", line);
    let resp: Response = respond(line.as_str());
    let str_resp = json!(&resp).to_string() + "\n";
    println!("{}", str_resp);
    stream.write_all(str_resp.as_bytes()).await?;
    //stream.write_u8(b'\n').await?;
    stream.flush().await?;
    stream.shutdown().await?;
    log::debug!("Server responded and shut down.");

    Ok(())
}

fn respond(request: &str) -> Response {
    let r: Response = match serde_json::from_str::<Request>(request) {
        Ok(request) => Response {
            method: "isPrime".to_string(),
            prime: primes::is_prime(request.number),
        },
        Err(_) => Response {
            method: "isPrime)".to_string(),
            prime: false,
        },
    };
    println!("{:#?}", r);
    return r;
}

#[cfg(test)]
mod tests {

    use super::*;
    use primes;
    use serde_json;
    use serde_json::json;
    use serde_json::Value;
    #[tokio::test]
    async fn test_connect_async() {}
    #[test]
    fn test_isprime() {
        let t1 = 3;
        assert_eq!(true, primes::is_prime(t1));
        log::info!("Is {} prime ? {} ", { t1 }, { primes::is_prime(t1) });
    }
    #[test]
    fn test_parse_request() {
        let req1 = "{\"method\":\"isPrime\",\"number\":123}";
        let v: Value = serde_json::from_str(req1).unwrap();
        assert_eq!("isPrime", v["method"]);

        let req2 = "{\"method\":\"isPrime\",\"number\":123, \"color\":3}";
        let r: Request = serde_json::from_str(req2).unwrap();
        assert_eq!(123, r.number);

        let req3 = "{\"method\":\"isPrime\",\"number\":12c}";
        let r3: Response = match serde_json::from_str::<&str>(req3) {
            Ok(_) => Response {
                method: "isPrime".to_string(),
                prime: true,
            },
            Err(_) => Response {
                method: "isPrime".to_string(),
                prime: false,
            },
        };
    }
    #[test]
    fn test_respond() {
        let req1 = "{\"method\":\"isPrime\",\"number\":123}\n";
        let resp = respond(req1);

        println!("{}", json!(&resp).to_string());
    }
}
