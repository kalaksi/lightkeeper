/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pub mod uptime;
pub use uptime::Uptime;

pub mod kernel;
pub use kernel::Kernel;

pub mod interface;
pub use interface::Interface;

pub mod package;
pub use package::Package;

pub mod who;
pub use who::Who;

pub mod load;
pub use load::Load;

pub mod ram;
pub use ram::Ram;