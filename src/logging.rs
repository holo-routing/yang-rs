use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::OnceLock;

use crate::ffi;

static LOG_CALLBACK: OnceLock<Box<dyn LogCallback>> = OnceLock::new();

/// A custom logger to pass to libyang.
pub trait LogCallback: Send + Sync + 'static {
    fn log<'a>(
        &'a self,
        level: ffi::LY_LOG_LEVEL::Type,
        msg: Option<Cow<'a, str>>,
        data_path: Option<Cow<'a, str>>,
        schema_path: Option<Cow<'a, str>>,
        line: u64,
    );
}

/// Set the log level to [`ffi::LY_LOG_LEVEL::LY_LLDBG`]
pub(crate) fn set_log_level_trace() {
    unsafe { ffi::ly_log_level(ffi::LY_LOG_LEVEL::LY_LLDBG) };
}

/// Set the log level to [`ffi::LY_LOG_LEVEL::LY_LLVRB`]
pub(crate) fn set_log_level_debug() {
    unsafe { ffi::ly_log_level(ffi::LY_LOG_LEVEL::LY_LLVRB) };
}

/// Set the log level to [`ffi::LY_LOG_LEVEL::LY_LLWRN`]
pub(crate) fn set_log_level_warn() {
    unsafe { ffi::ly_log_level(ffi::LY_LOG_LEVEL::LY_LLWRN) };
}

/// Set the log level to [`ffi::LY_LOG_LEVEL::LY_LLERR`]
pub(crate) fn set_log_level_error() {
    unsafe { ffi::ly_log_level(ffi::LY_LOG_LEVEL::LY_LLERR) };
}

/// An error returned when the logging callback has already been initialized.
#[derive(Debug)]
pub struct LoggingCallbackAlreadySet {
    _private: (),
}

impl std::fmt::Display for LoggingCallbackAlreadySet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Logging callback already set")
    }
}

impl std::error::Error for LoggingCallbackAlreadySet {}

/// Initialize the logging callback.
///
/// The callback can only be initialized once.
pub(crate) fn init_logger<C>(
    callback: C,
) -> Result<(), LoggingCallbackAlreadySet>
where
    C: LogCallback,
{
    unsafe { ffi::ly_log_options(ffi::LY_LOLOG | ffi::LY_LOSTORE_LAST) };
    LOG_CALLBACK
        .set(Box::new(callback))
        .map_err(|_| LoggingCallbackAlreadySet { _private: () })?;
    unsafe { ffi::ly_set_log_clb(Some(log_callback)) };
    Ok(())
}

extern "C" fn log_callback(
    level: ffi::LY_LOG_LEVEL::Type,
    msg: *const c_char,
    data_path: *const c_char,
    schema_path: *const c_char,
    line: u64,
) {
    let msg = if !msg.is_null() {
        // SAFETY: we assume that the arguments passed to the callback
        // are valid null terminated string valids for the entire
        // execution of this function.
        let cstr = unsafe { CStr::from_ptr(msg) };
        Some(cstr.to_string_lossy())
    } else {
        None
    };

    let data_path = if !data_path.is_null() {
        // SAFETY: we assume that the arguments passed to the callback
        // are valid null terminated string valids for the entire
        // execution of this function.
        let cstr = unsafe { CStr::from_ptr(data_path) };
        Some(cstr.to_string_lossy())
    } else {
        None
    };

    let schema_path = if !schema_path.is_null() {
        // SAFETY: we assume that the arguments passed to the callback
        // are valid null terminated string valids for the entire
        // execution of this function.
        let cstr = unsafe { CStr::from_ptr(schema_path) };
        Some(cstr.to_string_lossy())
    } else {
        None
    };

    if let Some(cb) = LOG_CALLBACK.get() {
        cb.log(level, msg, data_path, schema_path, line);
    }
}

/// A logger that to log libyang message using the `log` crate.
#[derive(Debug, Default)]
pub struct DefaultLogger {
    _private: (),
}

impl LogCallback for DefaultLogger {
    fn log<'a>(
        &'a self,
        level: ffi::LY_LOG_LEVEL::Type,
        msg: Option<Cow<'a, str>>,
        data_path: Option<Cow<'a, str>>,
        schema_path: Option<Cow<'a, str>>,
        line: u64,
    ) {
        let level = match level {
            ffi::LY_LOG_LEVEL::LY_LLERR => log::Level::Error,
            ffi::LY_LOG_LEVEL::LY_LLWRN => log::Level::Warn,
            ffi::LY_LOG_LEVEL::LY_LLVRB => log::Level::Info,
            ffi::LY_LOG_LEVEL::LY_LLDBG => log::Level::Debug,
            unknown => {
                log::error!("Unexpected log level {unknown} from libyang3, logging as debug");
                log::Level::Debug
            }
        };
        let msg = msg.unwrap_or_else(|| Cow::from(""));
        log::log! {
            target: "libyang3",
            level,
            "schema_path={schema_path:?}, data_path={data_path:?}, line={line}, msg={msg}",
        }
    }
}
