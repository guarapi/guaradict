use std::io;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    // Estabelecer conexão com o servidor
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Conexão estabelecida com o servidor.");

    // Loop do REPL
    loop {
        // Ler comando do usuário
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // Enviar comando para o servidor
        if let Err(e) = send_command(&mut stream, &input).await {
            eprintln!("Erro ao enviar comando para o servidor: {}", e);
            continue;
        }

        // Verificar se o comando é QUIT e sair do loop
        if input.trim() == "QUIT" {
            break;
        }

        // Ler e exibir resposta do servidor
        if let Err(e) = read_response(&mut stream).await {
            eprintln!("Erro ao ler resposta do servidor: {}", e);
            continue;
        }
    }

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
