//! Bindings for `sigset_t` on linux

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg(unix)]

mod sys {
    pub use libc::{sigaddset, sigdelset, sigemptyset, sigfillset, sigismember, sigset_t};
}

use core::fmt;
use core::mem::MaybeUninit;
use libc::c_int;

pub use sys::sigset_t;

pub struct SigSet {
    set: sys::sigset_t,
}

impl SigSet {
    pub fn empty() -> Self {
        unsafe {
            let mut set = MaybeUninit::uninit();
            sys::sigemptyset(set.as_mut_ptr());
            Self {
                set: set.assume_init(),
            }
        }
    }

    pub fn all() -> Self {
        unsafe {
            let mut set = MaybeUninit::uninit();
            sys::sigfillset(set.as_mut_ptr());
            Self {
                set: set.assume_init(),
            }
        }
    }

    pub fn as_ptr(&self) -> *const sigset_t {
        &self.set as *const sigset_t
    }

    pub fn as_mut_ptr(&mut self) -> *mut sigset_t {
        &mut self.set as *mut sigset_t
    }

    pub fn from_raw(set: sigset_t) -> Self {
        Self { set }
    }
    pub fn into_raw(self) -> sigset_t {
        self.set
    }

    pub fn add(&mut self, sig: Signal) -> Result<(), InvalidSignalError> {
        unsafe {
            let ret = sys::sigaddset(self.as_mut_ptr(), sig.into_raw());

            if ret < 0 {
                Err(InvalidSignalError(()))
            } else {
                Ok(())
            }
        }
    }

    pub fn remove(&mut self, sig: Signal) -> Result<(), InvalidSignalError> {
        unsafe {
            let ret = sys::sigdelset(self.as_mut_ptr(), sig.into_raw());

            if ret < 0 {
                Err(InvalidSignalError(()))
            } else {
                Ok(())
            }
        }
    }

    pub fn contains(&self, sig: Signal) -> Result<bool, InvalidSignalError> {
        unsafe {
            let ret = sys::sigismember(self.as_ptr(), sig.into_raw());

            if ret < 0 {
                Err(InvalidSignalError(()))
            } else {
                Ok(ret != 0)
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Signal(c_int);

impl Signal {
    pub const fn new(sig: c_int) -> Self {
        Self(sig)
    }

    pub const fn into_raw(self) -> c_int {
        self.0
    }
}

macro_rules! declare_signals {
    {
        $(
            $( #[$attr:meta] )*
            $vis:vis const $sig:ident;
        )*
    } => {
        impl Signal {
            $(
                $( #[$attr] )*
                $vis const $sig: Signal = Signal::new(libc::$sig);
            )*
        }
    }
}

declare_signals! {
    // Program Error
    pub const SIGFPE;
    pub const SIGILL;
    pub const SIGSEGV;
    pub const SIGBUS;
    pub const SIGABRT;
    pub const SIGIOT;
    pub const SIGTRAP;
    pub const SIGSYS;

    // Termination
    pub const SIGTERM;
    pub const SIGINT;
    pub const SIGQUIT;
    pub const SIGKILL;
    pub const SIGHUP;

    // Alarm
    pub const SIGALRM;
    pub const SIGVTALRM;
    pub const SIGPROF;

    // Async I/O
    pub const SIGIO;
    pub const SIGURG;
    pub const SIGPOLL;

    // Job Control
    pub const SIGCHLD;
    pub const SIGCONT;
    pub const SIGSTOP;
    pub const SIGTSTP;
    pub const SIGTTIN;
    pub const SIGTTOU;

    // Operation Error
    pub const SIGPIPE;
    pub const SIGXCPU;
    pub const SIGXFSZ;

    // Misc
    pub const SIGUSR1;
    pub const SIGUSR2;
    pub const SIGWINCH;
}

pub struct InvalidSignalError(());

impl fmt::Display for InvalidSignalError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Invalid signal")
    }
}

impl fmt::Debug for InvalidSignalError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("InvalidSignalError")
            .field("message", &format_args!("{}", self))
            .finish()
    }
}

#[cfg(feature = "std")]
mod with_std {
    use super::*;

    use std::io::Error as IOError;

    impl From<InvalidSignalError> for IOError {
        fn from(_: InvalidSignalError) -> IOError {
            IOError::from_raw_os_error(libc::EINVAL)
        }
    }

    impl std::error::Error for InvalidSignalError {}
}
