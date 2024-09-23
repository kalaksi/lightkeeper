extern crate qmetaobject;
use std::{cell::RefCell, rc::Rc, sync::mpsc, thread};

use qmetaobject::*;

use crate::{
    connection_manager::{CachePolicy, ConnectionManager},
    frontend::HostDisplayData,
    host_manager
};

use super::{
    CommandHandlerModel,
    ConfigManagerModel,
    HostDataManagerModel,
    ChartManagerModel
};


// This should probably be renamed to something like RequestHandlerModel.
#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct LkBackend {
    base: qt_base_class!(trait QObject),

    pub command: qt_property!(RefCell<CommandHandlerModel>; CONST),
    pub hosts: qt_property!(RefCell<HostDataManagerModel>; CONST),
    pub config: qt_property!(RefCell<ConfigManagerModel>; CONST),
    pub charts: qt_property!(RefCell<ChartManagerModel>; CONST),

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

    update_sender_prototype: Option<mpsc::Sender<HostDisplayData>>,
    update_receiver: Option<mpsc::Receiver<HostDisplayData>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,

    connection_manager: ConnectionManager,
    host_manager: Rc<RefCell<host_manager::HostManager>>,
}

#[allow(non_snake_case)]
impl LkBackend {
    pub fn new(
        update_sender_prototype: mpsc::Sender<HostDisplayData>,
        update_receiver: mpsc::Receiver<HostDisplayData>,
        host_manager: Rc<RefCell<host_manager::HostManager>>,
        connection_manager: ConnectionManager,
        host_data_model: HostDataManagerModel,
        // host_manager: Rc<RefCell<host_manager::HostManager>>,
        command_model: CommandHandlerModel,
        config_model: ConfigManagerModel) -> LkBackend {

        LkBackend {
            hosts: RefCell::new(host_data_model),
            command: RefCell::new(command_model),
            config: RefCell::new(config_model),
            charts: RefCell::new(ChartManagerModel::default()),
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

        let process_update = qmetaobject::queued_callback(move |new_data: HostDisplayData| {
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

        let thread = std::thread::spawn(move || {
            loop {
                match update_receiver.recv() {
                    Ok(received_data) => {
                        if received_data.stop {
                            ::log::debug!("Gracefully exiting UI state receiver thread");
                            return;
                        }
                        else {
                            process_update(received_data);
                        }
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

    pub fn new_update_sender(&self) -> mpsc::Sender<HostDisplayData> {
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
            self.new_update_sender()
                .send(HostDisplayData::stop()).unwrap();
            thread.join().unwrap();
        }

        self.command.borrow_mut().stop();
        self.host_manager.borrow_mut().stop();
        self.connection_manager.stop();
    }
}