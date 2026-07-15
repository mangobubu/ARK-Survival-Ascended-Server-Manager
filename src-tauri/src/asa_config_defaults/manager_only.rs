use crate::asa_config_metadata::AsaConfigDefault;

use super::{DefaultValue, Target, default};

pub(super) const DEFAULTS: &[AsaConfigDefault] = &[
    default(
        "visibility",
        DefaultValue::Text("public"),
        Target::ManagerOnly,
    ),
    default(
        "crossTransfer",
        DefaultValue::Bool(true),
        Target::GameUserSettingsServerSettings,
    ),
    default("autoRestart", DefaultValue::Bool(true), Target::ManagerOnly),
    default(
        "restartTime",
        DefaultValue::Text("04:00"),
        Target::ManagerOnly,
    ),
    default(
        "saveInterval",
        DefaultValue::U32(15),
        Target::GameUserSettingsServerSettings,
    ),
    default("backupRetention", DefaultValue::U32(7), Target::ManagerOnly),
    default(
        "autoUpdateMods",
        DefaultValue::Bool(true),
        Target::ManagerOnly,
    ),
    default(
        "restartOnCrash",
        DefaultValue::Bool(true),
        Target::ManagerOnly,
    ),
    default("saveOnStop", DefaultValue::Bool(true), Target::ManagerOnly),
    default(
        "destroyWildDinos",
        DefaultValue::Bool(false),
        Target::LaunchArgument,
    ),
    default(
        "tribeAlliances",
        DefaultValue::Bool(true),
        Target::GameUserSettingsServerSettings,
    ),
    default(
        "processPriority",
        DefaultValue::Text("aboveNormal"),
        Target::ManagerOnly,
    ),
    default(
        "cpuAffinity",
        DefaultValue::Text("自动"),
        Target::ManagerOnly,
    ),
    default(
        "memoryWarningGb",
        DefaultValue::U32(24),
        Target::ManagerOnly,
    ),
    default(
        "compressBackups",
        DefaultValue::Bool(true),
        Target::ManagerOnly,
    ),
    default(
        "snapshotBeforeRestart",
        DefaultValue::Bool(true),
        Target::ManagerOnly,
    ),
    default(
        "logLevel",
        DefaultValue::Text("normal"),
        Target::ManagerOnly,
    ),
    default("rotateSizeMb", DefaultValue::U32(100), Target::ManagerOnly),
    default(
        "logRetentionDays",
        DefaultValue::U32(14),
        Target::ManagerOnly,
    ),
    default(
        "logPath",
        DefaultValue::Text("ShooterGame/Saved/Logs"),
        Target::ManagerOnly,
    ),
];
