use std::{collections::HashMap, time::Duration};

use lightkeeper::enums::HostStatus;
use lightkeeper::{frontend::UIUpdate, module::*};
use lightkeeper::module::command::*;
use lightkeeper::module::platform_info::*;

use crate::{CommandTestHarness, StubSsh2};


#[test]
fn test_update_all() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: actual output
        StubSsh2::new(r#""apt" "upgrade" "-y""#,
r#"Reading package lists... Done
Building dependency tree... Done
Reading state information... Done
Calculating upgrade... Done
The following packages will be upgraded:
  docker-ce docker-ce-cli
2 upgraded, 0 newly installed, 0 to remove and 0 not upgraded.
Need to get 50.3 MB of archives.
After this operation, 12.3 MB of additional disk space will be used.
Do you want to continue? [Y/n] y
Get:1 https://download.docker.com/linux/debian bookworm/stable amd64 docker-ce-cli amd64 5:29.0.3-1~debian.12~bookworm [24.5 MB]
Get:2 https://download.docker.com/linux/debian bookworm/stable amd64 docker-ce amd64 5:29.0.3-1~debian.12~bookworm [25.8 MB]
Fetched 50.3 MB in 3s (15.2 MB/s)
(Reading database ... 123456 files and directories currently installed.)
Preparing to unpack .../docker-ce-cli_5%3a29.0.3-1~debian.12~bookworm_amd64.deb ...
Unpacking docker-ce-cli (5:29.0.3-1~debian.12~bookworm) over (5:29.0.1-1~debian.12~bookworm) ...
Preparing to unpack .../docker-ce_5%3a29.0.3-1~debian.12~bookworm_amd64.deb ...
Unpacking docker-ce (5:29.0.3-1~debian.12~bookworm) over (5:29.0.1-1~debian.12~bookworm) ...    
Setting up docker-ce-cli (5:29.0.3-1~debian.12~bookworm) ...
Setting up docker-ce (5:29.0.3-1~debian.12~bookworm) ...
Processing triggers for man-db (2.10.2-1) ..."#, 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::update_all::UpdateAll::get_metadata(), linux::packages::update_all::UpdateAll::new_command_module),
    );

    harness.execute_command(&linux::packages::update_all::UpdateAll::get_metadata().module_spec.id, vec![]);

    let update = harness.ui_update_receiver.recv_timeout(Duration::from_secs(1));
    assert_eq!(update.is_ok(), true);
    let ui_update = update.unwrap();
    match ui_update {
        UIUpdate::Host(display_data) => {
            assert_eq!(display_data.host_state.status, HostStatus::Pending);
            assert_eq!(display_data.host_state.command_invocations.len(), 1);
            // TODO: more
        },
        _ => {},
    }
}
