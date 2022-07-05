pub mod monitoring_module;
pub use monitoring_module::MonitoringModule;
pub use monitoring_module::Monitor;
pub use monitoring_module::MonitoringData;
pub use monitoring_module::DataPoint;
pub use crate::frontend::DisplayStyle;
pub use crate::frontend::DisplayOptions;

pub mod linux;
pub use linux::Uptime;
pub use linux::Docker;
pub mod network;
pub use network::Ping;
pub use network::Ssh;