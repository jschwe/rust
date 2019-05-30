use crate::error::Error as StdError;
use crate::ffi::{OsString, OsStr};
use crate::fmt;
use crate::io;
use crate::path::{self, PathBuf};
use crate::str;
use crate::sys::{unsupported, Void};
use crate::collections::HashMap;
use crate::vec;
use crate::sync::Mutex;
use crate::sync::atomic::{AtomicUsize, Ordering};
use crate::sync::Once;

extern "C" {
    fn sys_getpid() -> u32;
    fn sys_exit(arg: i32) ->!;
}

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

pub fn split_paths(_unparsed: &OsStr) -> SplitPaths<'_> {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

static ENV: AtomicUsize = AtomicUsize::new(0);
static ENV_INIT: Once = Once::new();

type EnvStore = Mutex<HashMap<OsString, OsString>>;

fn get_env_store() -> Option<&'static EnvStore> {
    unsafe { (ENV.load(Ordering::Relaxed) as *const EnvStore).as_ref() }
}

fn create_env_store() -> &'static EnvStore {
    ENV_INIT.call_once(|| {
        ENV.store(Box::into_raw(Box::new(EnvStore::default())) as _, Ordering::Relaxed)
    });
    unsafe {
        &*(ENV.load(Ordering::Relaxed) as *const EnvStore)
    }
}

pub type Env = vec::IntoIter<(OsString, OsString)>;

pub fn env() -> Env {
    let clone_to_vec = |map: &HashMap<OsString, OsString>| -> Vec<_> {
        map.iter().map(|(k, v)| (k.clone(), v.clone()) ).collect()
    };

    get_env_store()
        .map(|env| clone_to_vec(&env.lock().unwrap()) )
        .unwrap_or_default()
        .into_iter()
}

pub fn getenv(k: &OsStr) -> io::Result<Option<OsString>> {
    Ok(get_env_store().and_then(|s| s.lock().unwrap().get(k).cloned() ))
}

pub fn setenv(k: &OsStr, v: &OsStr) -> io::Result<()> {
    let (k, v) = (k.to_owned(), v.to_owned());
    create_env_store().lock().unwrap().insert(k, v);
    Ok(())
}

pub fn unsetenv(k: &OsStr) -> io::Result<()> {
    if let Some(env) = get_env_store() {
        env.lock().unwrap().remove(k);
    }
    Ok(())
}

pub fn temp_dir() -> PathBuf {
    panic!("no filesystem on hermit")
}

pub fn home_dir() -> Option<PathBuf> {
    None
}

pub fn exit(code: i32) -> ! {
    unsafe {
        sys_exit(code);
    }
}

pub fn getpid() -> u32 {
    unsafe {
        sys_getpid()
    }
}
