extern crate qmetaobject;
use std::{cell::RefCell, rc::Rc, sync::mpsc, thread};

use qmetaobject::*;

use crate::{
    configuration,
    host_manager,
    connection_manager::ConnectionManager,
    frontend::HostDisplayData,
};

use super::{CommandHandlerModel, ConfigManagerModel, HostDataManagerModel};


// This should probably be renamed to something like RequestHandlerModel.
#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct LkBackend {
    base: qt_base_class!(trait QObject),

    pub command: qt_property!(RefCell<CommandHandlerModel>; CONST),
    pub hosts: qt_property!(RefCell<HostDataManagerModel>; CONST),
    pub config: qt_property!(RefCell<ConfigManagerModel>; CONST),

    //
    // Slots
    //

    receiveUpdates: qt_method!(fn(&self)),
    reconfigure: qt_method!(fn(&mut self, config: QVariant, hosts_config: QVariant)),
    stop: qt_method!(fn(&mut self)),

    //
    // Signals
    //

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
            update_sender_prototype: Some(update_sender_prototype),
            update_receiver: Some(update_receiver),
            update_receiver_thread: None,
            host_manager: host_manager,
            connection_manager: connection_manager,
            ..Default::default()
        }
    }

    fn setup_signals(&mut self) {
        // function onHost_initialized(hostId) {
        //     let categories = LK.command.getAllHostCategories(hostId)
        //     for (const category of categories) {
        //         LK.command.refresh_monitors_of_category(hostId, category)
        //     }
        // }

        // let hosts_model = self.hosts.borrow();
        // let hosts_model_ptr = QPointer::from(&*hosts_model);

        // unsafe {
        //     qmetaobject::connections::connect(
        //         QPointer::from(&*hosts_model).cpp_ptr(),
        //         hosts_model.updateReceived.to_cpp_representation(hosts_model)
        //     );
        // }

        //     &HostDataManagerModel::updateDisplayData, move || {
        //     self_pinned.borrow().hosts.borrow_mut().process_update(new_display_data);
        // });
    }

    fn receiveUpdates(&mut self) {
        // Shouldn't (and can't) be run more than once.
        let update_receiver = if let Some(receiver) = self.update_receiver.take() {
            receiver
        } else {
            return;
        };

        let self_ptr = QPointer::from(&*self);
        let mut is_set_up = false;

        let process_update = qmetaobject::queued_callback(move |new_display_data: HostDisplayData| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
                // TODO
                // if new_display_data.host_state.just_initialized {
                //     ::log::debug!("Host {} initialized", new_display_data.host_state.host.name);
                //     self_pinned.borrow().command.
                // }
                // else if new_display_data.host_state.just_initialized_from_cache {
                //     ::log::debug!("Host {} initialized from cache", new_display_data.host_state.host.name);
                // }

                if !is_set_up {
                    // First, set up some signals.
                    self_pinned.borrow_mut().setup_signals();
                    is_set_up = true;
                }

                self_pinned.borrow().hosts.borrow_mut().process_update(new_display_data);
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

    fn reconfigure(&mut self, main_config: QVariant, hosts_config: QVariant) {
        let main_config = configuration::Configuration::from_qvariant(main_config).unwrap();
        let hosts_config = configuration::Hosts::from_qvariant(hosts_config).unwrap();

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