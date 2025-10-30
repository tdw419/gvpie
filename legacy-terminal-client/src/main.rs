use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use gpu_protocol::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server!");

    loop {
        let mut buf = vec![0; 1024];
        let n = stream.read(&mut buf).await?;

        if n == 0 {
            println!("Connection closed by server.");
            break;
        }

        let command: Command = bincode::deserialize(&buf[..n])?;

        match command {
            Command::TextRun(text_run) => {
                println!("[{}:{}] {}", text_run.x, text_run.y, text_run.text);
            }
            Command::FillRect(fill_rect) => {
                println!(
                    "FillRect: x={}, y={}, width={}, height={}, color={:?}",
                    fill_rect.x, fill_rect.y, fill_rect.width, fill_rect.height, fill_rect.color
                );
            }
        }
    }

    Ok(())
}
