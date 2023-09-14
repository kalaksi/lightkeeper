extern crate qmetaobject;
use qmetaobject::*;

use dbus;
use dbus::arg;
use dbus::arg::RefArg;

use rand;

use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;


/// The need for this model came from poor support for portals (related to sandboxing), like file chooser, in Qt.
/// However, things seem to be improving in Qt so this might be unneeded in the future.
#[derive(QObject, Default)]
pub struct DesktopPortalModel {
    base: qt_base_class!(trait QObject),

    receive_responses: qt_method!(fn(&self)),
    open_file_chooser: qt_method!(fn(&self) -> u32),

    file_chosen: qt_signal!(file_path: QString),

    receiver: Option<mpsc::Receiver<PortalRequest>>,
    sender: Option<mpsc::Sender<PortalRequest>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl DesktopPortalModel {
    pub fn new() -> DesktopPortalModel {
        let (sender, receiver) = mpsc::channel::<PortalRequest>();

        DesktopPortalModel {
            receiver: Some(receiver),
            sender: Some(sender),
            thread: None,
            ..Default::default()
        }
    }

    pub fn receive_responses(&mut self) {
        if self.thread.is_none() {
            // (Unfortunately) all dbus stuff should be run in one thread.
            // See the docs and https://github.com/diwic/dbus-rs/issues/375

            let self_ptr = QPointer::from(&*self);
            let handle_response = qmetaobject::queued_callback(move |file_chooser_response: FileChooserResponse| {
                if let Some(self_pinned) = self_ptr.as_pinned() {
                    ::log::debug!("Selected files: {:?}", file_chooser_response.file_uris);
                    let just_path = file_chooser_response.file_uris[0].clone().replace("file://", "");
                    self_pinned.borrow().file_chosen(QString::from(just_path));
                }
            });

            let dbus_connection = dbus::blocking::Connection::new_session().unwrap();
            // Sender ID is formed according to https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org.freedesktop.portal.Request
            let sender_id = dbus_connection.unique_name().trim_start_matches(':').replace('.', "_");
            let receiver = self.receiver.take().unwrap();
            let timeout = std::time::Duration::from_millis(5000);
            let recv_wait = std::time::Duration::from_millis(1000);

            let thread = thread::spawn(move || {
                loop {
                    let request = match receiver.recv_timeout(recv_wait) {
                        Ok(request) => request,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            dbus_connection.process(recv_wait).unwrap();
                            continue;
                        },
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            ::log::debug!("Portal response receiver thread disconnected");
                            return;
                        }
                    };

                    if request.exit {
                        ::log::debug!("Gracefully exiting portal response receiver thread");
                        return;
                    }

                    // For arguments, see: https://github.com/diwic/dbus-rs/blob/master/dbus/examples/argument_guide.md
                    // Ashpd sources (https://github.com/bilelmoussaoui/ashpd/) can also help.
                    // Ashpd wasn't used to keep dependencies to minimum (would have added 80 crates (totalling 270))
                    // since few dbus calls are needed.
                    match request.request_type {
                        PortalRequestType::FileChooser => {
                            let response_proxy = dbus_connection.with_proxy(
                                "org.freedesktop.portal.Request",
                                // TODO: use token? (requires parameters to dbus call).
                                format!("/org/freedesktop/portal/desktop/request/{}/t", sender_id),
                                timeout
                            );

                            let c_handle_response = handle_response.clone();
                            response_proxy.match_signal(move |response: FileChooserResponse, _: &dbus::blocking::Connection, _: &dbus::Message| {
                                c_handle_response(response);
                                false
                            }).unwrap();

                            // Send the request.
                            let request_proxy = dbus_connection.with_proxy(
                                "org.freedesktop.portal.Desktop",
                                "/org/freedesktop/portal/desktop",
                                timeout
                            );
                            let (_request_path,): (dbus::Path,) = request_proxy.method_call(
                                "org.freedesktop.portal.FileChooser",
                                "OpenFile",
                                // TODO: parent window ID. Currently left empty.
                                ("", "Select file", HashMap::<&str, arg::Variant<Box<dyn arg::RefArg + 'static>>>::new())
                            ).unwrap();

                        },
                        _ => {
                            ::log::warn!("Unknown portal request type");
                        }
                    }

                    dbus_connection.process(recv_wait).unwrap();
                }
            });

            self.thread = Some(thread);
        }
    }

    /// Calls org.freedestop.portal.FileChooser.OpenFile to open a file chooser dialog.
    pub fn open_file_chooser(&self) -> u32 {
        let token: u32 = rand::random();
        self.sender.as_ref().unwrap().send(PortalRequest::file_chooser(token)).unwrap();
        token
    }
}


#[derive(Default)]
pub struct PortalRequest {
    request_type: PortalRequestType,
    token: u32,
    exit: bool,
}

impl PortalRequest {
    pub fn file_chooser(token: u32) -> PortalRequest {
        PortalRequest {
            request_type: PortalRequestType::FileChooser,
            token: token,
            ..Default::default()
        }
    }

    pub fn exit() -> PortalRequest {
        PortalRequest {
            exit: true,
            ..Default::default()
        }
    }
}

#[derive(Default)]
enum PortalRequestType {
    #[default]
    Unknown,
    FileChooser,
}



#[derive(Debug)]
pub struct FileChooserResponse {
    pub status: u32,
    pub file_uris: Vec<String>,
}

impl arg::ReadAll for FileChooserResponse {
    fn read(iter: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        // Use this to debug:
        // let refarg = iter.get_refarg().unwrap();
        // println!("{:?}", refarg);

        // Example response from dbus-monitor:
        // signal time=1694539245.502865 sender=:1.44 -> destination=:1.12464 serial=2607 path=/org/freedesktop/portal/desktop/request/1_12464/t; interface=org.freedesktop.portal.Request; member=Response
        // uint32 0
        // array [
        //    dict entry(
        //       string "uris"
        //       variant             array [
        //             string "file:///home/user/file.txt"
        //          ]
        //    )
        // ]

        let status: u32 = iter.read()?;

        let results: arg::PropMap = iter.read()?;
        let file_uris = results.get("uris").unwrap().as_iter().unwrap().next().unwrap().as_iter().unwrap().map(|uris| {
            uris.as_str().unwrap().to_string()
        }).collect::<Vec<String>>();

        Ok(FileChooserResponse {
            status,
            file_uris: file_uris,
        })
    }
}

impl dbus::message::SignalArgs for FileChooserResponse {
    const NAME: &'static str = "Response";
    const INTERFACE: &'static str = "org.freedesktop.portal.Request";
}
