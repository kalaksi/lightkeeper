extern crate qmetaobject;
use std::{cell::RefCell, rc::Rc, sync::mpsc, thread};

use qmetaobject::*;

use crate::{
    connection_manager::{CachePolicy, ConnectionManager},
    frontend::{HostDisplayData, UIUpdate},
    host_manager,
    module::monitoring::MonitoringData,
    metrics,
};

use super::{
    CommandHandlerModel,
    ConfigManagerModel,
    HostDataManagerModel,
    MetricsManagerModel
};


// This should probably be renamed to something like RequestHandlerModel.
#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct LkBackend {
    base: qt_base_class!(trait QObject),

    pub command: qt_property!(RefCell<CommandHandlerModel>; CONST),
    pub hosts: qt_property!(RefCell<HostDataManagerModel>; CONST),
    pub config: qt_property!(RefCell<ConfigManagerModel>; CONST),
    pub metrics: qt_property!(RefCell<MetricsManagerModel>; CONST),

    //
    // Slots
    //

    receiveUpdates: qt_method!(fn(&self)),
    reload: qt_method!(fn(&mut self)),
    stop: qt_method!(fn(&mut self)),

    //
    // Signals
    //

    reloaded: qt_signal!(error: QString),

    //
    // Private properties
    //

    update_sender_prototype: Option<mpsc::Sender<UIUpdate>>,
    update_receiver: Option<mpsc::Receiver<UIUpdate>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,

    connection_manager: ConnectionManager,
    host_manager: Rc<RefCell<host_manager::HostManager>>,
}

#[allow(non_snake_case)]
impl LkBackend {
    pub fn new(
        update_sender_prototype: mpsc::Sender<UIUpdate>,
        update_receiver: mpsc::Receiver<UIUpdate>,
        host_manager: Rc<RefCell<host_manager::HostManager>>,
        connection_manager: ConnectionManager,
        host_data_model: HostDataManagerModel,
        // host_manager: Rc<RefCell<host_manager::HostManager>>,
        command_model: CommandHandlerModel,
        metrics_model: MetricsManagerModel,
        config_model: ConfigManagerModel) -> LkBackend {

        LkBackend {
            hosts: RefCell::new(host_data_model),
            command: RefCell::new(command_model),
            config: RefCell::new(config_model),
            metrics: RefCell::new(metrics_model),
            update_sender_prototype: Some(update_sender_prototype),
            update_receiver: Some(update_receiver),
            update_receiver_thread: None,
            host_manager: host_manager,
            connection_manager: connection_manager,
            ..Default::default()
        }
    }

    fn receiveUpdates(&mut self) {
        // Shouldn't (and can't) be run more than once.
        let update_receiver = if let Some(receiver) = self.update_receiver.take() {
            receiver
        } else {
            return;
        };

        let self_ptr = QPointer::from(&*self);
        let process_host_update = qmetaobject::queued_callback(move |new_data: HostDisplayData| {
            if let Some(self_pinned) = self_ptr.as_pinned() {

                // Special case for host initialization. Proper monitor processing is started after initialization step.
                if new_data.host_state.just_initialized {
                    ::log::debug!("Host {} initialized", new_data.host_state.host.name);
                    self_pinned.borrow().command.borrow_mut().refresh_host_monitors(new_data.host_state.host.name, None);
                    return;
                }
                else if new_data.host_state.just_initialized_from_cache {
                    ::log::debug!("Host {} initialized from cache", new_data.host_state.host.name);
                    self_pinned.borrow().command.borrow_mut().refresh_host_monitors(new_data.host_state.host.name, Some(CachePolicy::OnlyCache));
                    return;
                }

                self_pinned.borrow().hosts.borrow_mut().process_update(new_data);
            }
        });

        let self_ptr = QPointer::from(&*self);
        let process_chart_update = qmetaobject::queued_callback(move |response: metrics::tmserver::TMSResponse| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
                self_pinned.borrow().metrics.borrow_mut().process_update(response);
            }
        });

        let self_ptr = QPointer::from(&*self);
        let process_chart_insert = qmetaobject::queued_callback(move |(host_id, new_monitoring_data): (String, MonitoringData)| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
                for data_point in new_monitoring_data.values {
                    self_pinned.borrow().metrics.borrow_mut().insert_data_point(&host_id, &new_monitoring_data.monitor_id, data_point);
                }
            }
        });

        let thread = std::thread::spawn(move || {
            loop {
                match update_receiver.recv() {
                    Ok(received_data) => {
                        match received_data {
                            UIUpdate::Host(display_data) => {
                                if let Some(new_monitoring_data) = &display_data.new_monitoring_data {
                                    if new_monitoring_data.1.display_options.use_with_charts {
                                        process_chart_insert((display_data.host_state.host.name.clone(), new_monitoring_data.1.clone()));
                                    }
                                }
                                process_host_update(display_data);
                            }
                            UIUpdate::Chart(metrics) => process_chart_update(metrics),
                            UIUpdate::Stop() => {
                                ::log::debug!("Gracefully exiting UI state receiver thread");
                                return;
                            }
                        };
                    },
                    Err(error) => {
                        ::log::error!("Stopped UI state receiver thread: {}", error);
                        return;
                    }
                }
            }
        });

        self.update_receiver_thread = Some(thread);
    }

    pub fn new_update_sender(&self) -> mpsc::Sender<UIUpdate> {
        self.update_sender_prototype.clone().unwrap()
    }

    fn reload(&mut self) {
        match self.config.borrow_mut().reload_configuration() {
            Ok((main_config, hosts_config)) => {
                self.connection_manager.configure(&hosts_config, &main_config.cache_settings);
                self.host_manager.borrow_mut().configure(&hosts_config);
                self.command.borrow_mut().configure(
                    &main_config,
                    &hosts_config,
                    self.connection_manager.new_request_sender(),
                    self.host_manager.borrow().new_state_update_sender()
                );

                // `self.metrics` doesn't have to be reconfigured.

                self.host_manager.borrow_mut().start_receiving_updates();
                self.connection_manager.start_processing_requests();
                self.command.borrow_mut().start_processing_responses();

                self.reloaded(QString::from(""));
            },
            Err(error) => {
                let error = format!("Failed to reload configuration: {}", error);
                ::log::error!("{}", error);
                self.reloaded(QString::from(error));
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(thread) = self.update_receiver_thread.take() {
            if let Err(error) = self.new_update_sender().send(UIUpdate::Stop()) {
                ::log::error!("Failed to send stop signal to UI state receiver: {}", error);
            }

            thread.join().unwrap();
        }

        self.command.borrow_mut().stop();
        self.host_manager.borrow_mut().stop();
        self.connection_manager.stop();
        self.metrics.borrow_mut().stop();
    }
}