use std::collections::HashMap;

use lightkeeper::module::*;
use lightkeeper::module::command::*;
use lightkeeper::module::command::docker;
use lightkeeper::module::command::docker::compose;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;

use crate::{CommandTestHarness, StubSsh2};


#[test]
fn test_inspect() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "curl" "-s" "--unix-socket" "/var/run/docker.sock" "http://localhost/containers/test-container/json?all=true""#,
r#"{
  "Id": "8001819398ea4b320c0604a26f0d0f15ebd0353bc76f113f1d9ac629a83458d8",
  "Created": "2025-11-16T10:49:42.466460495Z",
  "Path": "/docker-entrypoint.sh",
  "Args": [
    "service1",
    "-g",
    "daemon off;"
  ],
  "State": {
    "Status": "running",
    "Running": true,
    "Paused": false,
    "Restarting": false,
    "OOMKilled": false,
    "Dead": false,
    "Pid": 22639,
    "ExitCode": 0,
    "Error": "",
    "StartedAt": "2025-12-01T01:30:34.423090046Z",
    "FinishedAt": "2025-12-01T01:30:34.222362896Z"
  },
  "Image": "sha256:28714d18af1a1a1ad4b147eb6fb9b043fd8aa2eec52556c506b532c251870ee0",
  "ResolvConfPath": "/var/lib/docker/containers/8001819398ea4b320c0604a26f0d0f15ebd0353bc76f113f1d9ac629a83458d8/resolv.conf",
  "HostnamePath": "/var/lib/docker/containers/8001819398ea4b320c0604a26f0d0f15ebd0353bc76f113f1d9ac629a83458d8/hostname",
  "HostsPath": "/var/lib/docker/containers/8001819398ea4b320c0604a26f0d0f15ebd0353bc76f113f1d9ac629a83458d8/hosts",
  "LogPath": "",
  "Name": "/project1-service1-1",
  "RestartCount": 0,
  "Driver": "overlay2",
  "Platform": "linux",
  "MountLabel": "",
  "ProcessLabel": "",
  "AppArmorProfile": "docker-default",
  "ExecIDs": null,
  "HostConfig": {
    "Binds": [
      "/mnt/containers/project1/data/www:/var/www:ro",
    ],
    "ContainerIDFile": "",
    "LogConfig": {
      "Type": "journald",
      "Config": {
        "tag": "container/{{.Name}}/{{.ID}}"
      }
    },
    "NetworkMode": "project1_default",
    "PortBindings": {
      "8080/tcp": [
        {
          "HostIp": "",
          "HostPort": "8080"
        }
      ],
      "8443/tcp": [
        {
          "HostIp": "",
          "HostPort": "8443"
        }
      ]
    },
    "RestartPolicy": {
      "Name": "unless-stopped",
      "MaximumRetryCount": 0
    },
    "AutoRemove": false,
    "VolumeDriver": "",
    "VolumesFrom": null,
    "ConsoleSize": [
      0,
      0
    ],
    "CapAdd": [
      "CAP_CHOWN",
      "CAP_DAC_OVERRIDE",
      "CAP_SETGID",
      "CAP_SETUID"
    ],
    "CapDrop": [
      "ALL"
    ],
    "CgroupnsMode": "private",
    "Dns": [],
    "DnsOptions": [],
    "DnsSearch": [],
    "ExtraHosts": [],
    "GroupAdd": null,
    "IpcMode": "private",
    "Cgroup": "",
    "Links": null,
    "OomScoreAdj": 0,
    "PidMode": "",
    "Privileged": false,
    "PublishAllPorts": false,
    "ReadonlyRootfs": false,
    "SecurityOpt": null,
    "UTSMode": "",
    "UsernsMode": "",
    "ShmSize": 67108864,
    "Runtime": "runc",
    "Isolation": "",
    "CpuShares": 0,
    "Memory": 0,
    "NanoCpus": 0,
    "CgroupParent": "",
    "BlkioWeight": 0,
    "BlkioWeightDevice": null,
    "BlkioDeviceReadBps": null,
    "BlkioDeviceWriteBps": null,
    "BlkioDeviceReadIOps": null,
    "BlkioDeviceWriteIOps": null,
    "CpuPeriod": 0,
    "CpuQuota": 0,
    "CpuRealtimePeriod": 0,
    "CpuRealtimeRuntime": 0,
    "CpusetCpus": "",
    "CpusetMems": "",
    "Devices": null,
    "DeviceCgroupRules": null,
    "DeviceRequests": null,
    "MemoryReservation": 0,
    "MemorySwap": 0,
    "MemorySwappiness": null,
    "OomKillDisable": null,
    "PidsLimit": null,
    "Ulimits": null,
    "CpuCount": 0,
    "CpuPercent": 0,
    "IOMaximumIOps": 0,
    "IOMaximumBandwidth": 0,
    "MaskedPaths": [
      "/proc/asound",
      "/proc/acpi",
      "/proc/interrupts",
      "/proc/kcore",
      "/proc/keys",
      "/proc/latency_stats",
      "/proc/timer_list",
      "/proc/timer_stats",
      "/proc/sched_debug",
      "/proc/scsi",
      "/sys/firmware",
      "/sys/devices/virtual/powercap"
    ],
    "ReadonlyPaths": [
      "/proc/bus",
      "/proc/fs",
      "/proc/irq",
      "/proc/sys",
      "/proc/sysrq-trigger"
    ]
  },
  "GraphDriver": {
    "Data": {
      "ID": "8001819398ea4b320c0604a26f0d0f15ebd0353bc76f113f1d9ac629a83458d8",
      "LowerDir": "/var/lib/docker/overlay2/ef873d2b74c318f0e18ade2a86f8bf617b7f893ec3c4982d6578422c2627e3de-init/diff:/var/lib/docker/overlay2/d145ff321f948a01219ce9000527a861d705cf265fe4bd988ea604ef5fe8ce76/diff:/var/lib/docker/overlay2/af2807a95789177c935ae3fcbedef680994e0bc092707e9abb80ced5a9e800dd/diff:/var/lib/docker/overlay2/33bd2b7c8fd6836f742c581047f26ab118e94e899434e011cee15be663fd5351/diff:/var/lib/docker/overlay2/fccb1d625b4639980898fde02e4d711ffe02c0f2a39ee909f70f1319a9d4ebed/diff:/var/lib/docker/overlay2/81613d55b42ae7d8e046135303bb24ee1104b46849830ac081549e21802caa99/diff:/var/lib/docker/overlay2/aed573124f949e636514ef6a15227cfb23f8735a30aa906eface0c4ea9ddd695/diff:/var/lib/docker/overlay2/b43547d7581806dfe09f8254994af9f6b9fce1eed185cc5b0fd5fb00de123b66/diff",
      "MergedDir": "/var/lib/docker/overlay2/ef873d2b74c318f0e18ade2a86f8bf617b7f893ec3c4982d6578422c2627e3de/merged",
      "UpperDir": "/var/lib/docker/overlay2/ef873d2b74c318f0e18ade2a86f8bf617b7f893ec3c4982d6578422c2627e3de/diff",
      "WorkDir": "/var/lib/docker/overlay2/ef873d2b74c318f0e18ade2a86f8bf617b7f893ec3c4982d6578422c2627e3de/work"
    },
    "Name": "overlay2"
  },
  "Mounts": [
    {
      "Type": "bind",
      "Source": "/mnt/containers/project1/data/www",
      "Destination": "/var/www",
      "Mode": "ro",
      "RW": false,
      "Propagation": "rprivate"
    }
  ],
  "Config": {
    "Hostname": "8001819398ea",
    "Domainname": "",
    "User": "",
    "AttachStdin": false,
    "AttachStdout": true,
    "AttachStderr": true,
    "ExposedPorts": {
      "80/tcp": {},
      "8080/tcp": {},
      "8443/tcp": {}
    },
    "Tty": false,
    "OpenStdin": false,
    "StdinOnce": false,
    "Env": [
      "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
      "service1_VERSION=1.28.0",
      "NJS_VERSION=0.8.10",
      "NJS_RELEASE=1~bookworm",
      "PKG_RELEASE=1~bookworm",
      "DYNPKG_RELEASE=1~bookworm"
    ],
    "Cmd": [
      "service1",
      "-g",
      "daemon off;"
    ],
    "Image": "service1:stable",
    "Volumes": null,
    "WorkingDir": "",
    "Entrypoint": [
      "/docker-entrypoint.sh"
    ],
    "Labels": {
      "com.docker.compose.config-hash": "6467660183e83d452c9e10e2cc4a865b9a403330ab071bcbf29828d8dc7594b2",
      "com.docker.compose.container-number": "1",
      "com.docker.compose.depends_on": "",
      "com.docker.compose.image": "sha256:28714d18af1a1a1ad4b147eb6fb9b043fd8aa2eec52556c506b532c251870ee0",
      "com.docker.compose.oneoff": "False",
      "com.docker.compose.project": "project1",
      "com.docker.compose.project.config_files": "/mnt/containers/project1/docker-compose.yml",
      "com.docker.compose.project.working_dir": "/mnt/containers/project1",
      "com.docker.compose.replace": "17d00f1461e343803f368e7cf3c500db9615fe9009b462b744c6e5bbd01270cc",
      "com.docker.compose.service": "service1",
      "com.docker.compose.version": "2.35.1",
      "maintainer": "service1 Docker Maintainers <docker-maint@nginx.com>"
    },
    "StopSignal": "SIGQUIT"
  },
  "NetworkSettings": {
    "SandboxID": "a5044e48aa6ee0b8ea2b7aaebcc5b941ce9a598b8f3acc7e794486702eeb777a",
    "SandboxKey": "/var/run/docker/netns/a5044e48aa6e",
    "Ports": {
      "80/tcp": null,
      "8080/tcp": [
        {
          "HostIp": "0.0.0.0",
          "HostPort": "8080"
        },
        {
          "HostIp": "::",
          "HostPort": "8080"
        }
      ]
    },
    "Networks": {
      "project1_default": {
        "IPAMConfig": null,
        "Links": null,
        "Aliases": [
          "project1-service1-1",
          "service1"
        ],
        "DriverOpts": null,
        "GwPriority": 0,
        "NetworkID": "2207c3dfc519ad38805c56c3571912c3718069cf287e897d25b0d672359c5f29",
        "EndpointID": "a6786b6db54dd6529dc0cbffaeca3ed2738c5fea144283af1937488600847e9a",
        "Gateway": "172.21.0.1",
        "IPAddress": "172.21.0.2",
        "MacAddress": "00:00:00:00:00:00",
        "IPPrefixLen": 16,
        "IPv6Gateway": "",
        "GlobalIPv6Address": "",
        "GlobalIPv6PrefixLen": 0,
        "DNSNames": [
          "project1-service1-1",
          "service1",
          "8001819398ea"
        ]
      }
    }
  }
}"#, 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (docker::inspect::Inspect::get_metadata(), docker::inspect::Inspect::new_command_module),
    );

    let module_id = docker::inspect::Inspect::get_metadata().module_spec.id;

    harness.execute_command(&module_id, vec!["test-container".to_string()]);
    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Info);
    });
}

// Docker Compose command tests

#[test]
fn test_compose_up_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/mnt/containers/project1/docker-compose.yml" "up" "-d""#,
            "", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Up::get_metadata(), compose::Up::new_command_module),
    );

    let module_id = compose::Up::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["/mnt/containers/project1/docker-compose.yml".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
        assert_eq!(result.message, "");
    });
}

#[test]
fn test_compose_up_with_service() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/mnt/containers/project1/docker-compose.yml" "up" "-d" "service1""#,
            "", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Up::get_metadata(), compose::Up::new_command_module),
    );

    let module_id = compose::Up::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec![
        "/mnt/containers/project1/docker-compose.yml".to_string(),
        "project1".to_string(),
        "service1".to_string(),
    ]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
    });
}

#[test]
fn test_compose_up_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/nonexistent/docker-compose.yml" "up" "-d""#,
            "Error: can't find a suitable configuration file", 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Up::get_metadata(), compose::Up::new_command_module),
    );

    let module_id = compose::Up::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["/nonexistent/docker-compose.yml".to_string()]);

    harness.verify_next_error(&module_id, |error| {
        assert_eq!(error.criticality, Criticality::Error);
        assert!(error.message.contains("can't find") || error.message.contains(&module_id));
    });
}

#[test]
fn test_compose_start_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/mnt/containers/project1/docker-compose.yml" "start""#,
            "", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Start::get_metadata(), compose::Start::new_command_module),
    );

    let module_id = compose::Start::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["/mnt/containers/project1/docker-compose.yml".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
    });
}

#[test]
fn test_compose_start_with_service() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/mnt/containers/project1/docker-compose.yml" "start" "service1""#,
            "", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Start::get_metadata(), compose::Start::new_command_module),
    );

    let module_id = compose::Start::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec![
        "/mnt/containers/project1/docker-compose.yml".to_string(),
        "project1".to_string(),
        "service1".to_string(),
    ]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
    });
}

#[test]
fn test_compose_start_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/nonexistent/docker-compose.yml" "start""#,
            "Error: can't find a suitable configuration file", 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Start::get_metadata(), compose::Start::new_command_module),
    );

    let module_id = compose::Start::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["/nonexistent/docker-compose.yml".to_string()]);

    harness.verify_next_error(&module_id, |error| {
        assert_eq!(error.criticality, Criticality::Error);
        assert!(error.message.contains("can't find") || error.message.contains(&module_id));
    });
}

#[test]
fn test_compose_stop_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/mnt/containers/project1/docker-compose.yml" "stop""#,
            "", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Stop::get_metadata(), compose::Stop::new_command_module),
    );

    let module_id = compose::Stop::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["/mnt/containers/project1/docker-compose.yml".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
        assert_eq!(result.message, "");
    });
}

#[test]
fn test_compose_stop_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/nonexistent/docker-compose.yml" "stop""#,
            "Error: can't find a suitable configuration file", 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Stop::get_metadata(), compose::Stop::new_command_module),
    );

    let module_id = compose::Stop::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["/nonexistent/docker-compose.yml".to_string()]);

    harness.verify_next_error(&module_id, |error| {
        assert_eq!(error.criticality, Criticality::Error);
        assert!(error.message.contains("can't find") || error.message.contains(&module_id));
    });
}

#[test]
fn test_compose_restart_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/mnt/containers/project1/docker-compose.yml" "restart""#,
            "", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Restart::get_metadata(), compose::Restart::new_command_module),
    );

    let module_id = compose::Restart::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["/mnt/containers/project1/docker-compose.yml".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
        assert_eq!(result.message, "");
    });
}

#[test]
fn test_compose_restart_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/nonexistent/docker-compose.yml" "restart""#,
            "Error: can't find a suitable configuration file", 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Restart::get_metadata(), compose::Restart::new_command_module),
    );

    let module_id = compose::Restart::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["/nonexistent/docker-compose.yml".to_string()]);

    harness.verify_next_error(&module_id, |error| {
        assert_eq!(error.criticality, Criticality::Error);
        assert!(error.message.contains("can't find") || error.message.contains(&module_id));
    });
}

#[test]
fn test_compose_logs_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/mnt/containers/project1/docker-compose.yml" "logs" "-t" "--tail" "1000" "service1""#,
            r#"project1-service1-1  | 2025-12-01T10:00:00.000Z Starting service
project1-service1-1  | 2025-12-01T10:00:01.000Z Service started successfully"#, 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Logs::get_metadata(), compose::Logs::new_command_module),
    );

    let module_id = compose::Logs::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec![
        "/mnt/containers/project1/docker-compose.yml".to_string(),
        "project1".to_string(),
        "service1".to_string(),
        "".to_string(),
        "".to_string(),
        "1000".to_string(),
    ]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
        assert!(result.message.contains("Starting service"));
        // Prefix should be removed
        assert!(!result.message.contains("project1-service1-1  |"));
    });
}

#[test]
fn test_compose_logs_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/mnt/containers/project1/docker-compose.yml" "logs" "-t" "--tail" "1000" "nonexistent""#,
            "Error: no such service: nonexistent", 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Logs::get_metadata(), compose::Logs::new_command_module),
    );

    let module_id = compose::Logs::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec![
        "/mnt/containers/project1/docker-compose.yml".to_string(),
        "project1".to_string(),
        "nonexistent".to_string(),
        "".to_string(),
        "".to_string(),
        "1000".to_string(),
    ]);

    harness.verify_next_error(&module_id, |error| {
        assert_eq!(error.criticality, Criticality::Error);
        assert!(error.message.contains("no such service") || error.message.contains(&module_id));
    });
}

#[test]
fn test_compose_pull_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/mnt/containers/project1/docker-compose.yml" "pull""#,
            "Pulling service1...\nlatest: Pulling from library/nginx\nStatus: Downloaded newer image", 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Pull::get_metadata(), compose::Pull::new_command_module),
    );

    let module_id = compose::Pull::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["/mnt/containers/project1/docker-compose.yml".to_string()]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
        assert!(result.message.contains("Pulling"));
    });
}

#[test]
fn test_compose_pull_with_local_image() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/mnt/containers/project1/docker-compose.yml" "pull""#,
            r#"Pulling service1...
Error response from daemon: dial tcp 127.0.0.1:80: connect: connection refused
0 errors occurred"#, 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Pull::get_metadata(), compose::Pull::new_command_module),
    );

    let module_id = compose::Pull::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["/mnt/containers/project1/docker-compose.yml".to_string()]);

    // Should succeed because local image errors are ignored
    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
    });
}

#[test]
fn test_compose_pull_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "docker" "compose" "-f" "/mnt/containers/project1/docker-compose.yml" "pull""#,
            r#"Pulling service1...
Error response from daemon: pull access denied
1 errors occurred"#, 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Pull::get_metadata(), compose::Pull::new_command_module),
    );

    let module_id = compose::Pull::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec!["/mnt/containers/project1/docker-compose.yml".to_string()]);

    harness.verify_next_error(&module_id, |error| {
        assert_eq!(error.criticality, Criticality::Error);
        assert!(error.message.contains("pull access denied") || error.message.contains("errors occurred") || error.message.contains(&module_id));
    });
}

#[test]
fn test_compose_build_success() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "docker" "compose" "--progress=plain" "-f" "/mnt/containers/project1/docker-compose.yml" "build" "service1""#,
            r#"Building service1
Step 1/5 : FROM nginx:latest
Step 2/5 : COPY . /usr/share/nginx/html
Step 3/5 : RUN echo 'Build complete'
Step 4/5 : EXPOSE 80
Step 5/5 : CMD ["nginx", "-g", "daemon off;"]
Successfully built abc123"#, 0)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Build::get_metadata(), compose::Build::new_command_module),
    );

    let module_id = compose::Build::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec![
        "/mnt/containers/project1/docker-compose.yml".to_string(),
        "project1".to_string(),
        "service1".to_string(),
    ]);

    harness.verify_next_command_result(&module_id, |result| {
        assert_eq!(result.criticality, Criticality::Normal);
        // Build command returns hidden message with build output
        assert!(result.message.contains("built")
          || result.message.contains("Step")
          || result.message.contains("Building")
          || result.message.is_empty()
        );
    });
}

#[test]
fn test_compose_build_error() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "docker" "compose" "--progress=plain" "-f" "/mnt/containers/project1/docker-compose.yml" "build" "service1""#,
            r#"Building service1
Step 1/5 : FROM nginx:latest
ERROR: failed to solve: failed to fetch"#, 1)
    };

    let mut harness = CommandTestHarness::new_command_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (compose::Build::get_metadata(), compose::Build::new_command_module),
    );

    let module_id = compose::Build::get_metadata().module_spec.id.clone();

    harness.execute_command(&module_id, vec![
        "/mnt/containers/project1/docker-compose.yml".to_string(),
        "project1".to_string(),
        "service1".to_string(),
    ]);

    harness.verify_next_ui_update(|display_data| {
        assert_eq!(display_data.host_state.command_invocations.len(), 1);
        assert_eq!(display_data.host_state.command_results.len(), 1);
        let result = &display_data.host_state.command_results[&module_id];
        assert!(result.progress < 100);
    });

    // Check additional partial updates (message is ~86 chars, so we get multiple partials)
    harness.verify_next_ui_update(|display_data| {
        let result = &display_data.host_state.command_results[&module_id];
        assert!(result.progress < 100);
    });

    harness.verify_next_ui_update(|display_data| {
        let result = &display_data.host_state.command_results[&module_id];
        assert!(result.progress < 100);
    });

    harness.verify_next_ui_update(|display_data| {
        let result = &display_data.host_state.command_results[&module_id];
        assert!(result.progress < 100);
    });

    harness.verify_next_ui_update(|display_data| {
        let result = &display_data.host_state.command_results[&module_id];
        assert_eq!(result.progress, 100);
        assert_eq!(result.criticality, Criticality::Error);
        assert!(result.message.contains("ERROR") || result.message.contains("failed") || result.message.contains("Building"));
    });
}