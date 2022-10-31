
pub enum LogType {
    Info,
    Debug,
    Warn,
    Error,
}

#[link(name = "hbatandroid")]
extern {
    fn bridge_backendLog(msg: *const u8, len: usize, lvl: i32);
}

pub fn log(msg: &str, logtype: LogType) {
    let typ = match logtype {
        LogType::Info => 0,
        LogType::Debug => 1,
        LogType::Warn => 2,
        LogType::Error => 3,
    };
    let strslice = msg.as_bytes();
    unsafe {
        bridge_backendLog(strslice.as_ptr(), strslice.len(), typ);
    }
}
