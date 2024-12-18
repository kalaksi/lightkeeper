#![allow(clippy::redundant_field_names)]
#![allow(clippy::needless_return)]
#![forbid(unsafe_code)]

use clap::Parser;
use lightkeeper::*;

#[derive(Parser, Clone)]
pub struct Args {
    #[clap(short, long, default_value = "")]
    pub config_dir: String,
    #[clap(long)]
    pub monitoring_module_info: bool,
    #[clap(long)]
    pub command_module_info: bool,
    #[clap(long)]
    pub connector_module_info: bool,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    if args.monitoring_module_info {
        let module_factory = ModuleFactory::new();
        print!("{}", module_factory.get_monitoring_module_info());
        return;
    }
    if args.command_module_info {
        let module_factory = ModuleFactory::new();
        print!("{}", module_factory.get_command_module_info());
        return;
    }
    if args.connector_module_info {
        let module_factory = ModuleFactory::new();
        print!("{}", module_factory.get_connector_module_info());
        return;
    }

    loop {
        log::info!("Lightkeeper starting...");

        let (main_config, hosts_config, group_config) = match Configuration::read(&args.config_dir) {
            Ok(configuration) => configuration,
            Err(error) => {
                log::error!("Error while reading configuration files: {}", error);
                break;
            }
        };

        let exit_reason = lightkeeper::run(&args.config_dir, &main_config, &hosts_config, &group_config, false);

        match exit_reason {
            ExitReason::Quit => break,
            ExitReason::Error => break,
            ExitReason::Restart => continue,
        };
    }
}
