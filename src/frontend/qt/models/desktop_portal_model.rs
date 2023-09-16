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
#[allow(non_snake_case)]
pub struct DesktopPortalModel {
    base: qt_base_class!(trait QObject),

    receiveResponses: qt_method!(fn(&self)),
    exit: qt_method!(fn(&mut self)),

    /// Returns token that can be used to match the response.
    openFileChooser: qt_method!(fn(&self) -> QString),
    fileChooserResponse: qt_signal!(token: QString, file_path: QString),

    receiver: Option<mpsc::Receiver<PortalRequest>>,
    sender: Option<mpsc::Sender<PortalRequest>>,
    thread: Option<thread::JoinHandle<()>>,
}

#[allow(non_snake_case)]
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

    pub fn receiveResponses(&mut self) {
        if self.thread.is_none() {
            // (Unfortunately) all dbus stuff should be run in one thread.
            // See the docs and https://github.com/diwic/dbus-rs/issues/375

            let self_ptr = QPointer::from(&*self);
            let handle_file_chooser_response = qmetaobject::queued_callback(move |response: FileChooserResponse| {
                if let Some(self_pinned) = self_ptr.as_pinned() {
                    if response.status == 0 {
                        ::log::debug!("Selected files: {:?}", response.file_uris);
                        let just_path = response.file_uris[0].clone().replace("file://", "");
                        self_pinned.borrow().fileChooserResponse(QString::from(response.token), QString::from(just_path));
                    }
                }
            });

            let dbus_connection = dbus::blocking::Connection::new_session().unwrap();
            // Sender ID is formed according to https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org.freedesktop.portal.Request
            let sender_id = dbus_connection.unique_name().trim_start_matches(':').replace('.', "_");
            let receiver = self.receiver.take().unwrap();
            let timeout = std::time::Duration::from_millis(5000);
            let recv_wait = std::time::Duration::from_millis(500);

            let thread = thread::spawn(move || {
                loop {
                    let request = match receiver.recv_timeout(recv_wait) {
                        Ok(request) => request,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            dbus_connection.process(recv_wait).unwrap();
                            continue;
                        },
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            ::log::error!("Portal response receiver thread disconnected");
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
                                format!("/org/freedesktop/portal/desktop/request/{}/{}", sender_id, request.token),
                                timeout
                            );

                            let c_handle_response = handle_file_chooser_response.clone();
                            let c_token = request.token.clone();
                            response_proxy.match_signal(move |mut response: FileChooserResponse, _: &dbus::blocking::Connection, _: &dbus::Message| {
                                response.token = c_token.clone();
                                c_handle_response(response);
                                false
                            }).unwrap();

                            let mut options = HashMap::<&str, arg::Variant<Box<dyn arg::RefArg + 'static>>>::new();
                            options.insert("handle_token", arg::Variant(Box::new(request.token)));

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
                                ("", "Select file", options),
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

    pub fn exit(&mut self) {
        self.sender.as_ref().unwrap().send(PortalRequest::exit()).unwrap();
        self.thread.take().unwrap().join().unwrap();
    }

    /// Calls org.freedestop.portal.FileChooser.OpenFile to open a file chooser dialog.
    pub fn openFileChooser(&self) -> QString {
        let token = Self::get_token();
        self.sender.as_ref().unwrap().send(PortalRequest::file_chooser(token.clone())).unwrap();
        QString::from(token)
    }

    fn get_token() -> String {
        let random_number: u32 = rand::random();
        format!("{}{}", "lightkeeper_", random_number)
    }
}


#[derive(Default)]
pub struct PortalRequest {
    request_type: PortalRequestType,
    token: String,
    exit: bool,
}

impl PortalRequest {
    pub fn file_chooser(token: String) -> PortalRequest {
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
    pub token: String,
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
            status: status,
            token: String::new(),
            file_uris: file_uris,
        })
    }
}

impl dbus::message::SignalArgs for FileChooserResponse {
    const NAME: &'static str = "Response";
    const INTERFACE: &'static str = "org.freedesktop.portal.Request";
}
