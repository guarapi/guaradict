use std::io;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use guaradict_core::commands::client;

#[tokio::main]
async fn main() -> io::Result<()> {
    loop {
        match connect_and_interact().await {
            Ok(()) => break,
            Err(err) => {
                eprintln!("Erro na conexão: {}", err);
                time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
    Ok(())
}

async fn connect_and_interact() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:13141").await?;
    println!("Conexão estabelecida com o servidor.");

    let mut rl = DefaultEditor::new().expect("Erro iniciando REPL");
    if rl.load_history("history.txt").is_err() {
        println!("Arquivo de histórico não encontrado");
    }

    loop {
        let readline = rl.readline("(db-name-placeholder): ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                if line.trim().eq_ignore_ascii_case("QUIT") {
                    break;
                }

                match client::Command::parse(&line) {
                    Ok(command) => {
                        let command_str = command.execute();
                        if let Err(err) = send_command(&mut stream, &command_str).await {
                            eprintln!("Erro ao enviar comando: {}", err);
                            return Err(err);
                        }
                        if let Err(err) = read_response(&mut stream).await {
                            eprintln!("Erro ao ler resposta: {}", err);
                            return Err(err);
                        }
                    }
                    Err(err) => {
                        println!("Erro ao analisar comando: {:?}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Erro: {:?}", err);
                break;
            }
        }
    }

    rl.save_history("history.txt").unwrap();

    Ok(())
}

async fn send_command(stream: &mut TcpStream, command: &str) -> io::Result<()> {
    stream.write_all(command.as_bytes()).await?;
    Ok(())
}

async fn read_response(stream: &mut TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    println!("{}", String::from_utf8_lossy(&buffer[..n]));
    Ok(())
}
