use crate::ffi::OsStr;
use crate::fmt;
use crate::io;
use crate::sys::fs::File;
use crate::sys::pipe::AnonPipe;
use crate::sys::{unsupported, Void};
use crate::sys_common::process::{CommandEnv, DefaultEnvKey};

const EXIT_SUCCESS: u8 = 0;
const EXIT_FAILURE: u8 = 1;

////////////////////////////////////////////////////////////////////////////////
// Command
////////////////////////////////////////////////////////////////////////////////

pub struct Command {
    env: CommandEnv<DefaultEnvKey>
}

// passed back to std::process with the pipes connected to the child, if any
// were requested
pub struct StdioPipes {
    pub stdin: Option<AnonPipe>,
    pub stdout: Option<AnonPipe>,
    pub stderr: Option<AnonPipe>,
}

pub enum Stdio {
    Inherit,
    Null,
    MakePipe,
}

impl Command {
    pub fn new(_program: &OsStr) -> Command {
        Command {
            env: Default::default()
        }
    }

    pub fn arg(&mut self, _arg: &OsStr) {
    }

    pub fn env_mut(&mut self) -> &mut CommandEnv<DefaultEnvKey> {
        &mut self.env
    }

    pub fn cwd(&mut self, _dir: &OsStr) {
    }

    pub fn stdin(&mut self, _stdin: Stdio) {
    }

    pub fn stdout(&mut self, _stdout: Stdio) {
    }

    pub fn stderr(&mut self, _stderr: Stdio) {
    }

    pub fn spawn(&mut self, _default: Stdio, _needs_stdin: bool)
        -> io::Result<(Process, StdioPipes)> {
        unsupported()
    }
}

impl From<AnonPipe> for Stdio {
    fn from(pipe: AnonPipe) -> Stdio {
        pipe.diverge()
    }
}

impl From<File> for Stdio {
    fn from(file: File) -> Stdio {
        file.diverge()
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitStatus(i32);

impl ExitStatus {
    fn exited(&self) -> bool {
        self.0 & 0x7F == 0
    }

    pub fn success(&self) -> bool {
        self.code() == Some(0)
    }

    pub fn code(&self) -> Option<i32> {
        if self.exited() {
            Some((self.0 >> 8) & 0xFF)
        } else {
            None
        }
    }

    pub fn signal(&self) -> Option<i32> {
        if !self.exited() {
            Some(self.0 & 0x7F)
        } else {
            None
        }
    }
}

impl From<i32> for ExitStatus {
    fn from(a: i32) -> ExitStatus {
        ExitStatus(a)
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.code() {
            write!(f, "exit code: {}", code)
        } else {
            let signal = self.signal().unwrap();
            write!(f, "signal: {}", signal)
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitCode(u8);

impl ExitCode {
    pub const SUCCESS: ExitCode = ExitCode(EXIT_SUCCESS as _);
    pub const FAILURE: ExitCode = ExitCode(EXIT_FAILURE as _);

    pub fn as_i32(&self) -> i32 {
        self.0 as i32
    }
}

pub struct Process(Void);

impl Process {
    pub fn id(&self) -> u32 {
        match self.0 {}
    }

    pub fn kill(&mut self) -> io::Result<()> {
        match self.0 {}
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        match self.0 {}
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        match self.0 {}
    }
}
