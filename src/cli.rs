use clap::{Parser, Subcommand};

/// Prático Web
#[derive(Parser)]
#[command(name = "tcp_to_ws", version = "1.0", about = "Serviço do Prático")]
struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Top-level commands
#[derive(Subcommand)]
enum Commands {
    /// Install the service
    Install,
    /// Uninstall the service
    Uninstall,
    /// Start the service
    Start,
    /// Stop the service
    Stop,
    /// Restart the service
    Restart,
    /// Get the service status
    Status,
}

pub fn cli() {
    let CliArgs {
        command: Some(command),
    } = CliArgs::parse()
    else {
        return;
    };
    let result = match command {
        Commands::Install => {
            serviceator::management::install().map(|install| {
                if install {
                    println!("serviço instalado");
                } else {
                    println!("o serviço já estava instalado");
                }
            })
        },
        Commands::Uninstall => {
            serviceator::management::uninstall().map(|uninstalled| {
                match uninstalled {
                    Some(true) => println!("serviço desinstalado"),
                    Some(false) => println!("serviço não está instalado"),
                    None => println!("serviço está desinstalando"),
                }
            })
        },
        Commands::Start => {
            serviceator::management::start().map(|()| {
                println!("serviço está iniciando")
            })
        },
        Commands::Stop => {
            serviceator::management::stop().map(|stopped| {
                if stopped {
                    println!("serviço está parado");
                } else {
                    println!("serviço está parando");
                }
            })
        },
        Commands::Restart => {
            serviceator::management::stop().and_then(|stopped| {
                if stopped {
                    println!("serviço está parado");
                    serviceator::management::start().map(|()| {
                        println!("serviço está iniciando")
                    })
                } else {
                    println!("serviço está parando, tente iniciar novamente depois");
                    Ok(())
                }
            })
        },
        Commands::Status => {
            serviceator::management::status().map(|status| {
                let status = match status {
                    serviceator::ServiceStatus::Stopped => "parado",
                    serviceator::ServiceStatus::Starting => "iniciando",
                    serviceator::ServiceStatus::Stopping => "parando",
                    serviceator::ServiceStatus::Running => "executando",
                    serviceator::ServiceStatus::Unpausing => "despausando",
                    serviceator::ServiceStatus::Pausing => "pausando",
                    serviceator::ServiceStatus::Paused => "pausado",
                };
                println!("serviço está {status}");
            })
        },
    };
    match result {
        Ok(()) => {
            std::process::exit(0)
        },
        Err(error) => {
            println!("erro: {error:?}");
            std::process::exit(1)
        },
    }
}
