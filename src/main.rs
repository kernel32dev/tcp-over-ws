mod cli;
mod config;

use async_tungstenite::tungstenite::client::IntoClientRequest;

fn main() {
//     // let connect_request = "wss://fb-getech.app-pratico.com.br".into_client_request().unwrap();
//     let connect_request = "wss://mm.app-pratico.com.br".into_client_request().unwrap();
//     let listen = ["127.0.0.1:19258".parse().unwrap()];

//     let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
//     let enter_guard = rt.enter();
//     let Ok(server) = rt.block_on(tcp_over_ws::bind(&listen[..])) else {
//         return;
//     };
//     drop(enter_guard);
//     let _ = std::thread::spawn(move || {
//         let _enter_guard = rt.enter();
//         let _ = rt.block_on(tcp_over_ws::tcp_to_ws_service(connect_request, server, tcp_over_ws::DEFAULT_TIMEOUT_MS));
//     }).join();
// }
// fn main2() {
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

    if false && cfg!(debug_assertions) {
        let connect_request = "ws://127.0.0.1:9601".into_client_request().unwrap();
        std::thread::spawn(move || {
            let listen = ["127.0.0.1:19258".parse().unwrap()];
        
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            let _enter_guard = rt.enter();
            let Ok(server) = rt.block_on(tcp_over_ws::bind(&listen[..])) else {
                return;
            };
            let _ = rt.block_on(tcp_over_ws::tcp_to_ws_service(connect_request, server, tcp_over_ws::DEFAULT_TIMEOUT_MS));
        });
    }

    let (listen, connect_addr) = config::load_config().unwrap_or_else(|()| std::process::exit(1));

    match tcp_over_ws::ws_to_tcp_service(connect_addr, listen) {
        Ok(()) => {}
        Err(error) => {
            println!("erro ao escutar: {error:?}");
            std::process::exit(1)
        }
    }
}
