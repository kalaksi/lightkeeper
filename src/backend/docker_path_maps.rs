/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use serde::Deserialize;

/// Container destination -> host source. Longer destinations are applied first.
pub type BindPathMap = Vec<(String, String)>;

#[derive(Deserialize)]
struct DockerMount {
    #[serde(rename = "Source")]
    source: Option<String>,
    #[serde(rename = "Destination")]
    destination: Option<String>,
}

pub fn rewrite_args_with_bind_maps(arguments: Vec<String>, maps: &BindPathMap) -> Vec<String> {
    if maps.is_empty() {
        return arguments;
    }
    arguments.into_iter().map(|argument| rewrite_path_value(&argument, maps)).collect()
}

pub fn rewrite_path_value(value: &str, maps: &BindPathMap) -> String {
    if maps.is_empty() {
        return value.to_string();
    }
    if let Some((key, path)) = value.split_once('=') {
        if path.starts_with('/') {
            return format!("{}={}", key, rewrite_absolute_path(path, maps));
        }
    }
    rewrite_absolute_path(value, maps)
}

fn rewrite_absolute_path(path: &str, maps: &BindPathMap) -> String {
    for (destination, source) in maps {
        if path == destination {
            return source.clone();
        }
        let prefix = format!("{}/", destination.trim_end_matches('/'));
        if let Some(rest) = path.strip_prefix(&prefix) {
            return format!("{}/{}", source.trim_end_matches('/'), rest);
        }
    }
    path.to_string()
}

/// Shell that prints mount JSON for running containers. When `use_sudo` is true, runs
/// `sudo docker`/`sudo podman` (passwordless sudo assumed).
pub fn docker_mounts_shell(use_sudo: bool) -> String {
    let sudo = if use_sudo { "sudo " } else { "" };
    format!(
        "cli=; \
         if command -v docker >/dev/null 2>&1; then cli=docker; \
         elif command -v podman >/dev/null 2>&1; then cli=podman; \
         else exit 127; fi; \
         ids=$({sudo}\"$cli\" ps -q --filter name=lightkeeper-core 2>/dev/null); \
         if [ -z \"$ids\" ]; then ids=$({sudo}\"$cli\" ps -q 2>/dev/null); fi; \
         if [ -z \"$ids\" ]; then printf '%s\\n' \"LK_NO_CONTAINERS cli=$cli\"; exit 0; fi; \
         for id in $ids; do {sudo}\"$cli\" inspect -f '{{{{json .Mounts}}}}' \"$id\" 2>/dev/null || true; done",
        sudo = sudo,
    )
}

/// Picks host bind mounts from the container whose Source covers `socket_path`.
pub fn select_bind_maps_for_socket(inspect_output: &str, socket_path: &str) -> Option<BindPathMap> {
    if inspect_output.trim().starts_with("LK_") {
        return None;
    }
    for line in inspect_output.lines() {
        let line = line.trim().trim_end_matches('\r');
        if line.is_empty() || line == "[]" || line.starts_with("LK_") {
            continue;
        }
        let Ok(mounts) = serde_json::from_str::<Vec<DockerMount>>(line)
        else {
            continue;
        };
        let binds: BindPathMap = mounts
            .into_iter()
            .filter(|mount| {
                mount.source.as_deref().is_some_and(|source| source.starts_with('/')) &&
                    mount.destination.as_deref().is_some_and(|destination| !destination.is_empty())
            })
            .filter_map(|mount| Some((mount.destination?, mount.source?)))
            .collect();
        if !binds.iter().any(|(_, source)| path_is_under(socket_path, source)) {
            continue;
        }
        let aligned = binds
            .into_iter()
            .map(|(destination, source)| (destination, prefer_path_form(&source, socket_path)))
            .collect();
        return Some(sorted_bind_maps(aligned));
    }
    None
}

fn path_is_under(path: &str, root: &str) -> bool {
    for candidate_path in path_match_aliases(path) {
        for candidate_root in path_match_aliases(root) {
            if candidate_path == candidate_root {
                return true;
            }
            let prefix = format!("{}/", candidate_root.trim_end_matches('/'));
            if candidate_path.starts_with(&prefix) {
                return true;
            }
        }
    }
    false
}

/// Path as given, plus `/var`+path or the path with a leading `/var` stripped.
/// Covers Fedora aliases such as `/mnt` <-> `/var/mnt` and `/home` <-> `/var/home`.
fn path_match_aliases(path: &str) -> Vec<String> {
    let trimmed = path.trim_end_matches('/').to_string();
    let mut aliases = vec![trimmed.clone()];
    if let Some(rest) = trimmed.strip_prefix("/var/") {
        aliases.push(format!("/{rest}"));
    }
    else if trimmed.starts_with('/') {
        aliases.push(format!("/var{trimmed}"));
    }
    aliases
}

/// Prefer the `/var` or non-`/var` spelling that matches `reference` (usually the dialog socket path).
/// Only adjusts paths that share the same first component (e.g. both under `mnt` or `home`).
fn prefer_path_form(path: &str, reference: &str) -> String {
    let path = path.trim_end_matches('/');
    if path_first_component(path) != path_first_component(reference) {
        return path.to_string();
    }
    let reference_wants_var = reference.starts_with("/var/");
    if reference_wants_var && !path.starts_with("/var/") {
        return format!("/var{path}");
    }
    if !reference_wants_var {
        if let Some(rest) = path.strip_prefix("/var/") {
            return format!("/{rest}");
        }
    }
    path.to_string()
}

fn path_first_component(path: &str) -> Option<&str> {
    let trimmed = path.trim_start_matches('/');
    let without_var = trimmed.strip_prefix("var/").unwrap_or(trimmed);
    without_var.split('/').next().filter(|component| !component.is_empty())
}

fn sorted_bind_maps(mut maps: BindPathMap) -> BindPathMap {
    maps.sort_by(|left, right| right.0.len().cmp(&left.0.len()));
    maps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_key_and_known_hosts_option() {
        let maps = vec![
            (String::from("/config"), String::from("/host/config")),
            (String::from("/data"), String::from("/host/data")),
        ];
        assert_eq!(rewrite_path_value("/config/id_ecdsa", &maps), "/host/config/id_ecdsa");
        assert_eq!(
            rewrite_path_value("UserKnownHostsFile=/config/known_hosts", &maps),
            "UserKnownHostsFile=/host/config/known_hosts"
        );
    }

    #[test]
    fn select_maps_matching_socket_source() {
        let output = concat!(
            r#"[{"Type":"bind","Source":"/srv/lk/config","Destination":"/config"},"#,
            r#"{"Type":"bind","Source":"/srv/lk/data","Destination":"/data"}]"#,
            "\n",
        );
        let maps = select_bind_maps_for_socket(output, "/srv/lk/data/core.sock").unwrap();
        assert_eq!(maps[0].1, "/srv/lk/config");
        assert_eq!(maps[1].1, "/srv/lk/data");
    }

    #[test]
    fn select_maps_var_prefix_dialog_vs_plain_source() {
        let output = concat!(
            r#"[{"Type":"bind","Source":"/mnt/container_data/lightkeeper-core/config","Destination":"/config"},"#,
            r#"{"Type":"bind","Source":"/mnt/container_data/lightkeeper-core/data","Destination":"/data"}]"#,
            "\n",
        );
        let maps = select_bind_maps_for_socket(output, "/var/mnt/container_data/lightkeeper-core/data/core.sock").unwrap();
        assert_eq!(maps[0].1, "/var/mnt/container_data/lightkeeper-core/config");
        assert_eq!(maps[1].1, "/var/mnt/container_data/lightkeeper-core/data");
    }

    #[test]
    fn select_maps_plain_dialog_vs_var_prefix_source() {
        let output = concat!(
            r#"[{"Type":"bind","Source":"/var/home/ops/stack/config","Destination":"/config"},"#,
            r#"{"Type":"bind","Source":"/var/home/ops/stack/data","Destination":"/data"}]"#,
            "\n",
        );
        let maps = select_bind_maps_for_socket(output, "/home/ops/stack/data/core.sock").unwrap();
        assert_eq!(maps[0].1, "/home/ops/stack/config");
        assert_eq!(maps[1].1, "/home/ops/stack/data");
    }

    #[test]
    fn prefer_path_form_skips_unrelated_mounts() {
        let output = concat!(
            r#"[{"Type":"bind","Source":"/mnt/container_data/lightkeeper-core/data","Destination":"/data"},"#,
            r#"{"Type":"bind","Source":"/etc/localtime","Destination":"/etc/localtime"}]"#,
            "\n",
        );
        let maps = select_bind_maps_for_socket(output, "/var/mnt/container_data/lightkeeper-core/data/core.sock").unwrap();
        assert!(maps
            .iter()
            .any(|(_, source)| source == "/var/mnt/container_data/lightkeeper-core/data"));
        assert!(maps.iter().any(|(_, source)| source == "/etc/localtime"));
    }
}
