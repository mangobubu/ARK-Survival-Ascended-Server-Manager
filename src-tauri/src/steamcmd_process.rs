#[cfg(windows)]
use std::{process::Stdio, time::Duration};

#[cfg(windows)]
use tokio::{process::Command, time::timeout};

#[cfg(windows)]
use windows_sys::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    GetProcessIoCounters, IO_COUNTERS, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

#[cfg(windows)]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Copy, Default)]
pub(crate) struct ProcessTransferCounters {
    read: u64,
    write: u64,
    other: u64,
}

impl ProcessTransferCounters {
    pub(crate) fn estimated_download_delta_since(self, baseline: Self) -> u64 {
        [
            self.read.saturating_sub(baseline.read),
            self.write.saturating_sub(baseline.write),
            self.other.saturating_sub(baseline.other),
        ]
        .into_iter()
        .max()
        .unwrap_or(0)
    }
}

#[cfg(windows)]
pub(crate) fn process_transfer_counters(
    process_id: Option<u32>,
) -> Option<ProcessTransferCounters> {
    let process_id = process_id?;
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
        if handle.is_null() {
            return None;
        }

        let mut counters = IO_COUNTERS::default();
        let ok = GetProcessIoCounters(handle, &mut counters);
        let _ = CloseHandle(handle);
        if ok == 0 {
            None
        } else {
            Some(ProcessTransferCounters {
                read: counters.ReadTransferCount,
                write: counters.WriteTransferCount,
                other: counters.OtherTransferCount,
            })
        }
    }
}

#[cfg(not(windows))]
pub(crate) fn process_transfer_counters(
    _process_id: Option<u32>,
) -> Option<ProcessTransferCounters> {
    None
}

#[cfg(windows)]
pub(crate) async fn kill_process_tree(pid: u32) {
    let mut command = Command::new("taskkill");
    command
        .arg("/PID")
        .arg(pid.to_string())
        .arg("/T")
        .arg("/F")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command.creation_flags(CREATE_NO_WINDOW);

    let _ = timeout(Duration::from_secs(5), command.status()).await;
}

#[cfg(not(windows))]
pub(crate) async fn kill_process_tree(_pid: u32) {}
