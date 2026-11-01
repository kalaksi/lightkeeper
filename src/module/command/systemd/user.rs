/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::host::Host;
use crate::module::platform_info;

pub mod start;
pub use start::Start;

pub mod stop;
pub use stop::Stop;

pub mod restart;
pub use restart::Restart;

pub mod mask;
pub use mask::Mask;

pub mod unmask;
pub use unmask::Unmask;

pub mod logs;
pub use logs::Logs;

pub(crate) fn supports_systemctl_user(host: &Host) -> bool {
    host.platform.is_same_or_greater(platform_info::Flavor::Debian, "9") ||
        host.platform.is_same_or_greater(platform_info::Flavor::Ubuntu, "20") ||
        host.platform.is_same_or_greater(platform_info::Flavor::RedHat, "7") ||
        host.platform.is_same_or_greater(platform_info::Flavor::CentOS, "7") ||
        host.platform.is_same_or_greater(platform_info::Flavor::AlmaLinux, "7") ||
        host.platform.is_same_or_greater(platform_info::Flavor::Rocky, "7") ||
        host.platform.is_same_or_greater(platform_info::Flavor::AmazonLinux, "2") ||
        host.platform.is_same_or_greater(platform_info::Flavor::Fedora, "22") ||
        host.platform.is_same_or_greater(platform_info::Flavor::NixOS, "20") ||
        host.platform.is_same_or_greater(platform_info::Flavor::OpenSUSE, "12") ||
        host.platform.os_flavor == platform_info::Flavor::ArchLinux
}
