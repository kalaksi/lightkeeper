extern crate qmetaobject;
use qmetaobject::*;

use dbus;
use dbus::arg;
use dbus::arg::RefArg;

use rand;

use std::collections::HashMap;
use std::os::fd::{AsRawFd, RawFd};
use std::sync::mpsc;
use std::thread;


/// The need for this model came from poor support for portals (related to sandboxing), like file chooser, in Qt.
/// However, things seem to be improving in Qt so this might be unneeded in the future.
#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct DesktopPortalModel {
    base: qt_base_class!(trait QObject),

    receiveResponses: qt_method!(fn(&self)),
    error: qt_signal!(error_message: QString),
    stop: qt_method!(fn(&mut self)),

    /// Returns token that can be used to match the response.
    openFileChooser: qt_method!(fn(&self) -> QString),
    fileChooserResponse: qt_signal!(token: QString, file_path: QString),
    openFile: qt_method!(fn(&self, file_path: QString) -> QString),
    openFileResponse: qt_signal!(token: QString),
    openedFileClosed: qt_signal!(token: QString),

    receiver: Option<mpsc::Receiver<PortalRequest>>,
    sender: Option<mpsc::Sender<PortalRequest>>,
    thread: Option<thread::JoinHandle<()>>,
    /// Key is token, value is file descriptor.
    open_files: HashMap<String, RawFd>,
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
            let handle_response = qmetaobject::queued_callback(move |portal_response: PortalResponse| {
                if let Some(self_pinned) = self_ptr.as_pinned() {

                    if let Some(response) = portal_response.file_chooser {
                        if response.status == 0 {
                            ::log::debug!("Selected files: {:?}", response.file_uris);
                            let just_path = response.file_uris[0].clone().replace("file://", "");
                            self_pinned.borrow().fileChooserResponse(QString::from(response.token), QString::from(just_path));
                        }
                        else if response.status == 2 {
                            self_pinned.borrow().error(QString::from("Unknown error occurred while choosing file"));
                        }
                    }
                    else if let Some(response) = portal_response.open_file {
                        if response.status == 0 {
                            ::log::debug!("Opened file");
                            self_pinned.borrow().openFileResponse(QString::from(response.token));
                        }
                        else if response.status == 2 {
                            self_pinned.borrow().error(QString::from("Unknown error occurred while opening file"));
                        }
                    }
                    /*
                    else if portal_response.check_invalid_fds {
                        // TODO: finding invalid fds doesn't seem to work.
                        let tokens = self_pinned.borrow_mut().find_invalid_fds();
                        ::log::debug!("Invalid fds: {:?}", tokens);
                        ::log::debug!("Files: {:?}", self_pinned.borrow().open_files);
                        for token in tokens {
                            ::log::debug!("Closed file");
                            self_pinned.borrow().openedFileClosed(QString::from(token));
                        }
                    }
                    */
                }
            });

            let dbus_connection = dbus::blocking::Connection::new_session().unwrap();
            // Sender ID is formed according to:
            // https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org.freedesktop.portal.Request

            let sender_id = dbus_connection.unique_name().trim_start_matches(':').replace('.', "_");
            let receiver = self.receiver.take().unwrap();
            let timeout = std::time::Duration::from_millis(5000);
            let recv_wait = std::time::Duration::from_millis(500);

            let thread = thread::spawn(move || {
                loop {
                    let c_handle_response = handle_response.clone();

                    let request = match receiver.recv_timeout(recv_wait) {
                        Ok(request) => request,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            c_handle_response(PortalResponse::check_invalid_fds());
                            dbus_connection.process(recv_wait).unwrap();
                            continue;
                        },
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            ::log::error!("Portal response receiver thread disconnected");
                            return;
                        }
                    };

                    if request.stop {
                        ::log::debug!("Gracefully exiting portal response receiver thread");
                        return;
                    }

                    // For arguments, see: https://github.com/diwic/dbus-rs/blob/master/dbus/examples/argument_guide.md
                    // Ashpd sources (https://github.com/bilelmoussaoui/ashpd/) can also help.
                    // Ashpd wasn't used to keep dependencies to minimum (would have added 80 crates (totalling 270))
                    // since few dbus calls are needed.
                    let token = request.token.clone();
                    let request_proxy = dbus_connection.with_proxy(
                        "org.freedesktop.portal.Desktop",
                        "/org/freedesktop/portal/desktop",
                        timeout
                    );
                    let response_proxy = dbus_connection.with_proxy(
                        "org.freedesktop.portal.Request",
                        format!("/org/freedesktop/portal/desktop/request/{}/{}", sender_id, request.token),
                        timeout
                    );
                    match request.request_type {
                        PortalRequestType::OpenFileChooser => {
                            response_proxy.match_signal(move |mut response: FileChooserResponse, _: &dbus::blocking::Connection, _: &dbus::Message| {
                                response.token = token.clone();
                                c_handle_response(PortalResponse::file_chooser(response));
                                false
                            }).unwrap();

                            let mut options = HashMap::<&str, arg::Variant<Box<dyn arg::RefArg + 'static>>>::new();
                            options.insert("handle_token", arg::Variant(Box::new(request.token)));

                            // Send the request.
                            let (_request_path,): (dbus::Path,) = request_proxy.method_call(
                                "org.freedesktop.portal.FileChooser",
                                "OpenFile",
                                // TODO: parent window ID. Currently left empty.
                                ("", "Select file", options),
                            ).unwrap();
                        },
                        PortalRequestType::OpenFile => {
                            response_proxy.match_signal(move |mut response: OpenFileResponse, _: &dbus::blocking::Connection, _: &dbus::Message| {
                                response.token = token.clone();
                                c_handle_response(PortalResponse::open_file(response));
                                false
                            }).unwrap();

                            let mut options = HashMap::<&str, arg::Variant<Box<dyn arg::RefArg + 'static>>>::new();
                            options.insert("handle_token", arg::Variant(Box::new(request.token)));
                            options.insert("writable", arg::Variant(Box::new(true)));
                            // Always asks which app to use. Portals don't currently integrate properly with desktop environments,
                            // and will remember their own settings which would then require separate mechanism for resetting them.
                            options.insert("ask", arg::Variant(Box::new(true)));

                            // Send the request.
                            let (_request_path,): (dbus::Path,) = request_proxy.method_call(
                                "org.freedesktop.portal.OpenURI",
                                "OpenFile",
                                // TODO: parent window ID. Currently left empty.
                                ("", request.file.unwrap(), options),
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

    pub fn stop(&mut self) {
        let request = PortalRequest {
            stop: true,
            ..Default::default()
        };
        self.sender.as_ref().unwrap().send(request).unwrap();
        self.thread.take().unwrap().join().unwrap();
    }

    /// Calls org.freedestop.portal.FileChooser.OpenFile to open a file chooser dialog.
    pub fn openFileChooser(&self) -> QString {
        let token = Self::get_token();
        let request = PortalRequest {
            request_type: PortalRequestType::OpenFileChooser,
            token: token.clone(),
            ..Default::default()
        };
        self.sender.as_ref().unwrap().send(request).unwrap();
        QString::from(token)
    }

    /// Calls org.freedestop.portal.OpenURI.OpenFile to open a file.
    pub fn openFile(&mut self, file_path: QString) -> QString {
        let file = std::fs::File::open(file_path.to_string()).unwrap();
        let token = Self::get_token();
        self.open_files.insert(token.clone(), file.as_raw_fd());

        let request = PortalRequest {
            request_type: PortalRequestType::OpenFile,
            token: token.clone(),
            file: Some(file),
            ..Default::default()
        };
        self.sender.as_ref().unwrap().send(request).unwrap();
        QString::from(token)
    }

    fn get_token() -> String {
        let random_number: u32 = rand::random();
        format!("{}{}", "lightkeeper_", random_number)
    }

    /* Currently not used anywhere but could be useful.
    fn find_invalid_fds(&mut self) -> Vec<String> {
        self.open_files.iter()
            .filter(|(_, fd)| { !file_utils::is_valid_fd(**fd) })
            .map(|(token, _)| { token.clone() }).collect::<Vec<String>>()
    }
    */
}


#[derive(Default)]
pub struct PortalRequest {
    request_type: PortalRequestType,
    token: String,
    file: Option<std::fs::File>,
    stop: bool,
}

#[derive(Default)]
enum PortalRequestType {
    #[default]
    Unknown,
    OpenFileChooser,
    OpenFile,
}

#[derive(Default)]
pub struct PortalResponse {
    file_chooser: Option<FileChooserResponse>,
    open_file: Option<OpenFileResponse>,
    _check_invalid_fds: bool,
}

impl PortalResponse {
    pub fn file_chooser(response: FileChooserResponse) -> PortalResponse {
        PortalResponse {
            file_chooser: Some(response),
            ..Default::default()
        }
    }

    pub fn open_file(response: OpenFileResponse) -> PortalResponse {
        PortalResponse {
            open_file: Some(response),
            ..Default::default()
        }
    }

    pub fn check_invalid_fds() -> PortalResponse {
        PortalResponse {
            _check_invalid_fds: true,
            ..Default::default()
        }
    }
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


#[derive(Debug)]
pub struct OpenFileResponse {
    pub status: u32,
    pub token: String,
}

impl arg::ReadAll for OpenFileResponse {
    fn read(iter: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        let status: u32 = iter.read()?;

        Ok(OpenFileResponse {
            status: status,
            token: String::new(),
        })
    }
}

impl dbus::message::SignalArgs for OpenFileResponse {
    const NAME: &'static str = "Response";
    const INTERFACE: &'static str = "org.freedesktop.portal.Request";
}