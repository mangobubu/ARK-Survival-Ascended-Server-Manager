use crate::asa_config_metadata::AsaConfigDefault;

use super::{DefaultValue, Target, default};

pub(super) const DEFAULTS: &[AsaConfigDefault] = &[
    default(
        "networkTickRate",
        DefaultValue::U32(30),
        Target::EngineIniIpNetDriver,
    ),
    default(
        "maxClientRate",
        DefaultValue::U32(100000),
        Target::EngineIniIpNetDriver,
    ),
    default(
        "customEngineIniSettings",
        DefaultValue::Text(""),
        Target::EngineIniIpNetDriver,
    ),
];
