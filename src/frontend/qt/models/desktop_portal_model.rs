extern crate qmetaobject;
use qmetaobject::*;

use dbus;
use dbus::arg;
use dbus::arg::RefArg;

use std::collections::HashMap;


/// The need for this model came from poor support for portals, like file chooser, (related to sandboxing) in Qt.
/// However, things seem to be improving so this might be unneeded in the future.
#[derive(QObject, Default)]
pub struct DesktopPortalModel {
    base: qt_base_class!(trait QObject),

    open_file_chooser: qt_method!(fn(&self)),
}

impl DesktopPortalModel {
    pub fn new() -> DesktopPortalModel {
        DesktopPortalModel {
            ..Default::default()
        }
    }

    /// Calls org.freedestop.portal.FileChooser.OpenFile to open a file chooser dialog.
    pub fn open_file_chooser(&self) {
        // For arguments, see: https://github.com/diwic/dbus-rs/blob/master/dbus/examples/argument_guide.md
        // Ashpd sources (https://github.com/bilelmoussaoui/ashpd/) can also help.
        // Ashpd wasn't used to keep dependencies to minimum (would have added 80 crates (totalling 270)) since few dbus calls are needed.

        let timeout = std::time::Duration::from_millis(5000);
        let connection = dbus::blocking::Connection::new_session().unwrap();

        // Sender ID is formed according to https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org.freedesktop.portal.Request
        let sender_id = connection.unique_name().trim_start_matches(':').replace('.', "_");
        let request_path = dbus::Path::new(format!("/org/freedesktop/portal/desktop/request/{}/t", sender_id)).unwrap();

        // Wait for Response signal.
        let response_proxy = connection.with_proxy("org.freedesktop.portal.Request", &request_path, timeout);
        response_proxy.match_signal(|response: FileChooserResponse, _: &dbus::blocking::Connection, _: &dbus::Message| {
            ::log::debug!("Selected files: {:?}", response.file_uris);
            true
        }).unwrap();

        // Send the request.
        let request_proxy = connection.with_proxy("org.freedesktop.portal.Desktop", "/org/freedesktop/portal/desktop", timeout);
        let (returned_request_path,): (dbus::Path,) = request_proxy.method_call(
            "org.freedesktop.portal.FileChooser",
            "OpenFile",
            // TODO: parent window ID. Currently left empty.
            ("", "Select file", HashMap::<&str, arg::Variant<Box<dyn arg::RefArg + 'static>>>::new())
        ).unwrap();

        if returned_request_path != request_path {
            panic!("Returned DBus object path is wrong");
        }

        loop {
            connection.process(std::time::Duration::from_millis(1000)).unwrap();
        }
    }
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
