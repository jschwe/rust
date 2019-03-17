use crate::error::Error as StdError;
use crate::ffi::{OsString, OsStr};
use crate::fmt;
use crate::io;
use crate::path::{self, PathBuf};
use crate::str;
use crate::sys::{unsupported, Void};

pub fn errno() -> i32 {
    0
}

pub fn error_string(_errno: i32) -> String {
    "operation successful".to_string()
}

pub fn getcwd() -> io::Result<PathBuf> {
    unsupported()
}

pub fn chdir(_: &path::Path) -> io::Result<()> {
    unsupported()
}

pub struct SplitPaths<'a>(&'a Void);

pub fn split_paths(_unparsed: &OsStr) -> SplitPaths {
    panic!("unsupported")
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        match *self.0 {}
    }
}

#[derive(Debug)]
pub struct JoinPathsError;

pub fn join_paths<I, T>(_paths: I) -> Result<OsString, JoinPathsError>
    where I: Iterator<Item=T>, T: AsRef<OsStr>
{
    Err(JoinPathsError)
}

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "not supported on hermit yet".fmt(f)
    }
}

impl StdError for JoinPathsError {
    fn description(&self) -> &str {
        "not supported on hermit yet"
    }
}

pub fn current_exe() -> io::Result<PathBuf> {
    unsupported()
}

pub struct Env(Void);

impl Iterator for Env {
    type Item = (OsString, OsString);
    fn next(&mut self) -> Option<(OsString, OsString)> {
        match self.0 {}
    }
}

pub fn env() -> Env {
    panic!("not supported on HermitCore")
}

pub fn getenv(_k: &OsStr) -> io::Result<Option<OsString>> {
    //Ok(GetEnvSysCall::perform(k))
    Ok(None)
}

pub fn setenv(_k: &OsStr, _v: &OsStr) -> io::Result<()> {
    //Ok(SetEnvSysCall::perform(k, Some(v)))
    Ok(())
}

pub fn unsetenv(_k: &OsStr) -> io::Result<()> {
   // Ok(SetEnvSysCall::perform(k, None))
   Ok(())
}

pub fn temp_dir() -> PathBuf {
    panic!("no filesystem on hermit")
}

pub fn home_dir() -> Option<PathBuf> {
    None
}

pub fn exit(_code: i32) -> ! {
    loop {}
}

pub fn getpid() -> u32 {
    panic!("no pids on hermit")
}
