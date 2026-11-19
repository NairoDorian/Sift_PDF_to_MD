//! Windows job object that ties model servers to this process (from ZER0's `job_object.rs`,
//! MIT, see NOTICE): created with JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, so the kernel ends every
//! llama-server when the last handle (held by this process) goes away, even after a crash
//! (D2). Unlike ZER0, membership is unconditional: a Sift model server never outlives Sift.

#[cfg(windows)]
pub fn register(child: &std::process::Child) {
    use std::os::windows::io::AsRawHandle;
    use std::sync::OnceLock;
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation, SetInformationJobObject,
    };

    struct Job(isize);
    static JOB: OnceLock<Option<Job>> = OnceLock::new();
    let job = JOB.get_or_init(|| unsafe {
        let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if h.is_null() {
            log::warn!("CreateJobObjectW failed: model servers are not tied to Sift");
            return None;
        }
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if SetInformationJobObject(
            h,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) == 0
        {
            log::warn!("SetInformationJobObject failed");
            return None;
        }
        Some(Job(h as isize))
    });
    if let Some(j) = job
        && unsafe { AssignProcessToJobObject(j.0 as _, child.as_raw_handle() as _) } == 0
    {
        log::warn!("AssignProcessToJobObject failed");
    }
}

#[cfg(not(windows))]
pub fn register(_child: &std::process::Child) {
    // Sift is Windows only (MASTER_PLAN VD1).
}
