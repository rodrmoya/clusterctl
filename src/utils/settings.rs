/*
 * Raspberry Pi cluster manager.
 *
 * Copyright (C) 2020-2021 Rodrigo Moya <rodrigo@gnome.org>
 */

use super::logging::LogLevel;

pub struct ClusterSettings
{
    pub inventory_file: String,
    pub verbosity_level: LogLevel
}