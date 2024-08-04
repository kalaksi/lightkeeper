extern crate qmetaobject;
use std::{cell::RefCell, sync::mpsc, thread};

use qmetaobject::*;

use crate::frontend::HostDisplayData;

use super::{CommandHandlerModel, ConfigManagerModel, HostDataManagerModel};

// This should probably be renamed to something like RequestHandlerModel.
#[allow(non_snake_case)]
#[derive(QObject, Default)]
pub struct LkBackend{
    base: qt_base_class!(trait QObject),

    pub command: qt_property!(RefCell<CommandHandlerModel>; CONST),
    pub hosts: qt_property!(RefCell<HostDataManagerModel>; CONST),
    pub config: qt_property!(RefCell<ConfigManagerModel>; CONST),

    //
    // Slots
    //

    receiveUpdates: qt_method!(fn(&self)),
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
}

#[allow(non_snake_case)]
impl LkBackend {
    pub fn new(
        update_sender_prototype: mpsc::Sender<HostDisplayData>,
        update_receiver: mpsc::Receiver<HostDisplayData>,
        host_data_model: HostDataManagerModel,
        command_model: CommandHandlerModel,
        config_model: ConfigManagerModel) -> LkBackend {

        LkBackend {
            hosts: RefCell::new(host_data_model),
            command: RefCell::new(command_model),
            config: RefCell::new(config_model),
            update_sender_prototype: Some(update_sender_prototype),
            update_receiver: Some(update_receiver),
            update_receiver_thread: None,
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

        let process_update = qmetaobject::queued_callback(move |new_display_data: HostDisplayData| {
            if let Some(self_pinned) = self_ptr.as_pinned() {
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

    pub fn stop(&mut self) {
        self.new_update_sender()
            .send(HostDisplayData::stop()).unwrap();

        if let Some(thread) = self.update_receiver_thread.take() {
            thread.join().unwrap();
        }
    }
}