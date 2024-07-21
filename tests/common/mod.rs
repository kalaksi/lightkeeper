
use lightkeeper::*;


static mut MAIN_CONFIG: Option<Configuration> = None;
static mut HOSTS_CONFIG: Option<configuration::Hosts> = None;
static mut GROUP_CONFIG: Option<configuration::Groups> = None;
const CONFIG_DIR: &str = "test-env";

/// Unsafe because of static mutable variables for caching configs.
pub unsafe fn setup() -> (String, Configuration, configuration::Hosts, configuration::Groups) {

    if MAIN_CONFIG.is_none() {
        let (main_config, hosts_config, group_config) = match Configuration::read(CONFIG_DIR) {
            Ok(configuration) => configuration,
            Err(error) => {
                panic!("Error while reading configuration files: {}", error);
            }
        };

        MAIN_CONFIG = Some(main_config);
        HOSTS_CONFIG = Some(hosts_config);
        GROUP_CONFIG = Some(group_config);
    }
    
    (CONFIG_DIR.to_string(), MAIN_CONFIG.clone().unwrap(),HOSTS_CONFIG.clone().unwrap(), GROUP_CONFIG.clone().unwrap())
}