use std::net::SocketAddr;

const DEFAULT_CONFIG: &'static str = r#"# esse é o arquivo de configuração do serviço que tem um servidor websocket e conecta a serviços tcps

# isso é um arquivo de exemplo, descomente as linhas definindo listen e connect para o serviço funcionar

# uma lista de ipv4s ou ipv6s ou portas separados por (;), as aspas são obrigatórias
#listen = "127.0.0.1:9601;[::1]:9601"
# um ipv4, ipv6 ou porta, as aspas são obrigatórias
#connect = "127.0.0.1:19259"
"#;

pub fn load_config() -> Result<(Vec<SocketAddr>, SocketAddr), ()> {
    let filename = if cfg!(debug_assertions) {
        std::env::current_dir()
            .map_err(|error| {
                println!("erro ao obter o caminho do exe atual: {error:?}");
            })?
            .join("config.toml")
    } else {
        std::env::current_exe()
            .map_err(|error| {
                println!("erro ao obter o caminho do exe atual: {error:?}");
            })?
            .parent()
            .unwrap_or(std::path::Path::new(""))
            .join("config.toml")
    };

    if !filename.exists() {
        let _ = std::fs::write(&filename, DEFAULT_CONFIG);
    }

    let text = std::fs::read_to_string(&filename).map_err(|error| {
        println!("erro ao ler {}: {error:?}", filename.display());
    })?;

    let Config { listen, connect } = toml::from_str(&text).map_err(|error| {
        println!(
            "o arquivo de config em {} não está no formato correto: {error:?}",
            filename.display()
        );
    })?;

    let listen = tcp_over_ws::addr::parse_many_socket_addr(&listen);

    if listen.is_empty() {
        println!("nenhum endereço de escuta válido configurado");
    }

    let Some(connect) = tcp_over_ws::addr::parse_one_socket_addr(&connect) else {
        println!("o endereço de conecção não é válido");
        return Err(());
    };

    Ok((listen, connect))
}

#[derive(serde::Deserialize)]
struct Config {
    listen: String,
    connect: String,
}
