use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt};
use std::error::Error;
use gpu_protocol::{Command, TextRun};

pub async fn start_server() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_client(socket).await;
        });
    }
}

async fn handle_client(mut socket: TcpStream) {
    println!("New client connected!");

    // Simulate the output of the emulated COBOL program
    let cobol_output = "Hello from COBOL!";

    // AI-powered greeting
    let ai_greeting = "Welcome to the GPU-native AI OS! Your COBOL program says: ";
    let combined_output = format!("{}{}", ai_greeting, cobol_output);

    let text_run = TextRun {
        x: 0,
        y: 0,
        text: combined_output,
    };
    let command = Command::TextRun(text_run);
    let serialized = bincode::serialize(&command).unwrap();

    if let Err(e) = socket.write_all(&serialized).await {
        eprintln!("Failed to write to socket: {}", e);
    }
}
