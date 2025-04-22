/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pub mod module_factory;
pub use module_factory::ModuleFactory;

pub mod metadata;
pub use metadata::Metadata;

pub mod module;
pub use module::Module;
pub use module::MetadataSupport;

pub mod connection;
pub mod command;
pub mod monitoring;

pub mod module_specification;
pub use module_specification::ModuleSpecification;
pub use module_specification::ModuleType;

pub mod platform_info;
pub use platform_info::PlatformInfo;