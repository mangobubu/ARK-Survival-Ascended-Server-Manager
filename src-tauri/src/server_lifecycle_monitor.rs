mod auto_restart;
mod config;
mod process_exit;
mod startup;

pub(crate) use auto_restart::should_auto_restart_after_exit;
pub(crate) use config::bool_from_config;
pub(crate) use process_exit::{apply_exited_instance_status, take_exited_instance_process};
pub(crate) use startup::monitor_startup_readiness;
