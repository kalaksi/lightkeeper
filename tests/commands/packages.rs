use std::collections::HashMap;

use lightkeeper::module::*;
use lightkeeper::module::command::*;
use lightkeeper::module::command::linux;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;

use crate::{CommandTestHarness, StubSsh2};


#[test]
fn test_update_all() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: actual output
        StubSsh2::new(r#""sudo" "apt" "upgrade" "-y""#,
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

    let module_id = linux::packages::update_all::UpdateAll::get_metadata().module_spec.id.clone();

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::update_all::UpdateAll::get_metadata(), linux::packages::update_all::UpdateAll::new_command_module),
    );

    harness.execute_command(&module_id, vec![]);

    harness.verify_next_ui_update(|display_data| {
        assert_eq!(display_data.host_state.command_invocations.len(), 1);
        assert_eq!(display_data.host_state.command_results.len(), 1);
        assert!(display_data.host_state.command_results[&module_id].progress < 100);
        assert_eq!(display_data.host_state.command_results[&module_id].message, "Reading package list");
    });

    harness.verify_next_ui_update(|display_data| {
        assert_eq!(display_data.host_state.command_invocations.len(), 1);
        assert_eq!(display_data.host_state.command_results.len(), 1);
        assert!(display_data.host_state.command_results[&module_id].progress < 100);
        assert_eq!(display_data.host_state.command_results[&module_id].message, "Reading package lists... Done\nBuilding d");
    });
}

#[test]
fn test_update_all_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "apt" "upgrade" "-y""#,
            "E: Could not open lock file /var/lib/dpkg/lock-frontend - open (13: Permission denied)\nE: Unable to acquire the dpkg frontend lock (/var/lib/dpkg/lock-frontend), are you root?", 1)
    };

    let module_id = linux::packages::update_all::UpdateAll::get_metadata().module_spec.id.clone();

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::update_all::UpdateAll::get_metadata(), linux::packages::update_all::UpdateAll::new_command_module),
    );

    harness.execute_command(&module_id, vec![]);

    // UpdateAll command uses FollowOutput, so error responses also get split into partials
    harness.verify_next_ui_update(|display_data| {
        assert_eq!(display_data.host_state.command_invocations.len(), 1);
        assert_eq!(display_data.host_state.command_results.len(), 1);
    });

    // Wait for all partial responses to complete
    loop {
        harness.verify_next_ui_update(|display_data| {
            let result = &display_data.host_state.command_results[&module_id];
            if result.progress == 100 {
                assert_eq!(result.criticality, Criticality::Error);
            }
        });

        let display_data = harness.host_manager.borrow().get_display_data();
        let host_display = display_data.hosts.get(crate::TEST_HOST_ID).unwrap();
        let result = &host_display.host_state.command_results[&module_id];
        if result.progress == 100 {
            assert_eq!(result.criticality, Criticality::Error);
            assert!(result.message.contains("Permission denied") || result.message.contains("Unable to acquire"));
            break;
        }
    }
}

#[test]
fn test_update_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "apt" "--only-upgrade" "-y" "install" "docker-ce""#,
r#"Reading package lists... Done
Building dependency tree... Done
Reading state information... Done
The following packages will be upgraded:
  docker-ce
1 upgraded, 0 newly installed, 0 to remove and 0 not upgraded.
Need to get 25.8 MB of archives.
After this operation, 6.1 MB of additional disk space will be used.
Get:1 https://download.docker.com/linux/debian bookworm/stable amd64 docker-ce amd64 5:29.0.3-1~debian.12~bookworm [25.8 MB]
Fetched 25.8 MB in 2s (12.9 MB/s)
Preparing to unpack .../docker-ce_5%3a29.0.3-1~debian.12~bookworm_amd64.deb ...
Unpacking docker-ce (5:29.0.3-1~debian.12~bookworm) over (5:29.0.1-1~debian.12~bookworm) ...
Setting up docker-ce (5:29.0.3-1~debian.12~bookworm) ...
Processing triggers for man-db (2.10.2-1) ..."#, 0)
    };

    let module_id = linux::packages::update::Update::get_metadata().module_spec.id.clone();

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::update::Update::get_metadata(), linux::packages::update::Update::new_command_module),
    );

    harness.execute_command(&module_id, vec!["docker-ce".to_string()]);

    harness.verify_next_ui_update(|display_data| {
        assert_eq!(display_data.host_state.command_invocations.len(), 1);
        assert_eq!(display_data.host_state.command_results.len(), 1);
        let result = &display_data.host_state.command_results[&module_id];
        assert!(result.progress < 100);
    });

    // Wait for multiple partial responses (message is long, so we get many partials)
    // Loop until we get the final response with progress 100
    loop {
        harness.verify_next_ui_update(|display_data| {
            let result = &display_data.host_state.command_results[&module_id];
            if result.progress == 100 {
                assert_eq!(result.criticality, Criticality::Normal);
            } else {
                assert!(result.progress < 100);
            }
        });

        // Check if we got the final response
        let display_data = harness.host_manager.borrow().get_display_data();
        let host_display = display_data.hosts.get(crate::TEST_HOST_ID).unwrap();
        let result = &host_display.host_state.command_results[&module_id];
        if result.progress == 100 {
            break;
        }
    }
}

#[test]
fn test_update_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "apt" "--only-upgrade" "-y" "install" "nonexistent-package""#,
            "E: Unable to locate package nonexistent-package", 1)
    };

    let module_id = linux::packages::update::Update::get_metadata().module_spec.id.clone();

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::update::Update::get_metadata(), linux::packages::update::Update::new_command_module),
    );

    harness.execute_command(&module_id, vec!["nonexistent-package".to_string()]);

    // Update command uses FollowOutput, so error responses also get split into partials
    harness.verify_next_ui_update(|display_data| {
        assert_eq!(display_data.host_state.command_invocations.len(), 1);
        assert_eq!(display_data.host_state.command_results.len(), 1);
    });

    // Wait for all partial responses to complete
    loop {
        harness.verify_next_ui_update(|display_data| {
            let result = &display_data.host_state.command_results[&module_id];
            if result.progress == 100 {
                assert_eq!(result.criticality, Criticality::Error);
            }
        });

        let display_data = harness.host_manager.borrow().get_display_data();
        let host_display = display_data.hosts.get(crate::TEST_HOST_ID).unwrap();
        let result = &host_display.host_state.command_results[&module_id];
        if result.progress == 100 {
            assert_eq!(result.criticality, Criticality::Error);
            assert!(result.message.contains("Unable to locate package") || result.message.contains("nonexistent-package"));
            break;
        }
    }
}

#[test]
fn test_clean_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "apt-get" "clean""#,
            "", 0)
    };

    let module_id = linux::packages::clean::Clean::get_metadata().module_spec.id.clone();

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::clean::Clean::get_metadata(), linux::packages::clean::Clean::new_command_module),
    );

    harness.execute_command(&module_id, vec![]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Info);
        assert_eq!(result.message, "Package cache cleaned");
    });
}

#[test]
fn test_clean_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "apt-get" "clean""#,
            "E: Could not open lock file /var/lib/dpkg/lock-frontend - open (13: Permission denied)", 1)
    };

    let module_id = linux::packages::clean::Clean::get_metadata().module_spec.id.clone();

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::clean::Clean::get_metadata(), linux::packages::clean::Clean::new_command_module),
    );

    harness.execute_command(&module_id, vec![]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Error);
        // Clean command puts error in the error field, not message field
        assert!(result.error.contains("Permission denied") || result.error.contains("lock"));
    });
}

#[test]
fn test_refresh_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "apt" "update""#,
r#"Hit:1 http://deb.debian.org/debian bookworm InRelease
Hit:2 http://deb.debian.org/debian-security bookworm-security InRelease
Hit:3 http://deb.debian.org/debian bookworm-updates InRelease
Reading package lists... Done"#, 0)
    };

    let module_id = linux::packages::refresh::Refresh::get_metadata().module_spec.id.clone();

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::refresh::Refresh::get_metadata(), linux::packages::refresh::Refresh::new_command_module),
    );

    harness.execute_command(&module_id, vec![]);

    // Refresh command uses FollowOutput, so it will receive partial responses
    harness.verify_next_ui_update(|display_data| {
        assert_eq!(display_data.host_state.command_invocations.len(), 1);
        assert_eq!(display_data.host_state.command_results.len(), 1);
    });

    // Wait longer for all partial responses to complete (message is split into many chunks)
    std::thread::sleep(std::time::Duration::from_millis(1000));
    
    // Verify final result - Update command returns hidden result on success
    harness.verify_command_result(&module_id, |result| {
        assert_eq!(result.progress, 100);
        assert_eq!(result.criticality, Criticality::Normal);
    });
}

#[test]
fn test_refresh_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "apt" "update""#,
            "E: Could not open lock file /var/lib/apt/lists/lock - open (13: Permission denied)", 1)
    };

    let module_id = linux::packages::refresh::Refresh::get_metadata().module_spec.id.clone();

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::refresh::Refresh::get_metadata(), linux::packages::refresh::Refresh::new_command_module),
    );

    harness.execute_command(&module_id, vec![]);

    // Refresh command uses FollowOutput, so error responses also get split into partials
    harness.verify_next_ui_update(|display_data| {
        assert_eq!(display_data.host_state.command_invocations.len(), 1);
        assert_eq!(display_data.host_state.command_results.len(), 1);
    });

    // Wait for all partial responses to complete
    loop {
        harness.verify_next_ui_update(|display_data| {
            let result = &display_data.host_state.command_results[&module_id];
            if result.progress == 100 {
                assert_eq!(result.criticality, Criticality::Error);
            }
        });

        let display_data = harness.host_manager.borrow().get_display_data();
        let host_display = display_data.hosts.get(crate::TEST_HOST_ID).unwrap();
        let result = &host_display.host_state.command_results[&module_id];
        if result.progress == 100 {
            assert_eq!(result.criticality, Criticality::Error);
            break;
        }
    }
}

#[test]
fn test_logs_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "tail" "-n" "1000" "/var/log/apt/term.log""#,
r#"Log file: /var/log/apt/term.log
Reading package lists...
Building dependency tree...
Reading state information...
The following packages will be upgraded:
  docker-ce docker-ce-cli
2 upgraded, 0 newly installed, 0 to remove and 0 not upgraded.
Setting up docker-ce (5:29.0.3-1~debian.12~bookworm) ..."#, 0)
    };

    let module_id = linux::packages::logs::Logs::get_metadata().module_spec.id.clone();

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::logs::Logs::get_metadata(), linux::packages::logs::Logs::new_command_module),
    );

    harness.execute_command(&module_id, vec![]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
        assert!(result.message.contains("Reading package lists") || result.message.contains("docker-ce"));
    });
}

#[test]
fn test_logs_with_parameters() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "tail" "-n" "500" "/var/log/apt/term.log""#,
r#"Log file: /var/log/apt/term.log
Reading package lists...
Building dependency tree..."#, 0)
    };

    let module_id = linux::packages::logs::Logs::get_metadata().module_spec.id.clone();

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::logs::Logs::get_metadata(), linux::packages::logs::Logs::new_command_module),
    );

    harness.execute_command(&module_id, vec![
        "".to_string(),
        "".to_string(),
        "1".to_string(),
        "500".to_string(),
    ]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
        assert!(result.message.contains("Reading package lists") || result.message.contains("Log file"));
    });
}

#[test]
fn test_logs_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "tail" "-n" "1000" "/var/log/apt/term.log""#,
            "tail: cannot open '/var/log/apt/term.log' for reading: No such file or directory", 1)
    };

    let module_id = linux::packages::logs::Logs::get_metadata().module_spec.id.clone();

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::packages::logs::Logs::get_metadata(), linux::packages::logs::Logs::new_command_module),
    );

    harness.execute_command(&module_id, vec![]);

    harness.verify_next_error(&module_id, |error| {
        assert_eq!(error.criticality, Criticality::Error);
        assert!(error.message.contains("No such file") || error.message.contains("term.log"));
    });
}
