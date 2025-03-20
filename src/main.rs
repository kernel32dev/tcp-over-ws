mod cli;
mod config;

use async_tungstenite::tungstenite::client::IntoClientRequest;

fn main() {
    if let Some(result) = serviceator::lifecycle::define_service(
        main,
        serviceator::ServiceInfo {
            service_name: "ws_to_tcp".into(),
            display_name: "ws_to_tcp".into(),
            description: "Serviço que expõe serviços TCP locais via conecções WS".into(),
        },
    ) {
        match result {
            Ok(()) => return,
            Err(error) => {
                println!("erro ao definir o serviço: {error:?}");
                std::process::exit(1)
            }
        }
    }

    cli::cli();

    if cfg!(debug_assertions) {
        let connect_request = "ws://127.0.0.1:9601".into_client_request().unwrap();
        std::thread::spawn(move || {
            tcp_over_ws::tcp_to_ws_service(
                connect_request.clone(),
                vec!["127.0.0.1:19258".parse().unwrap()],
                tcp_over_ws::DEFAULT_TIMEOUT_MS,
            )
        });
    }

    let (listen, connect_addr) = config::load_config().unwrap_or_else(|()| std::process::exit(1));

    let error = tcp_over_ws::ws_to_tcp_service(connect_addr, listen).unwrap_err();

    println!("erro ao escutar: {error:?}");
    std::process::exit(1)
}
