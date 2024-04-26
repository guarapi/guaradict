use std::io;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use guaradict_core::commands::Command;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Estabelecer conexão com o servidor
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
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

                match Command::parse(&line) {
                    Ok(command) => {
                        let command_str = command.execute();
                        send_command(&mut stream, &command_str).await?;
                        read_response(&mut stream).await?;
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
