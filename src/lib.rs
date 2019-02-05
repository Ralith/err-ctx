//! Minimalist helper library for providing contextual errors that display in the traditional
//! "context: cause" format. Useful for applications where the primary goal is to convey detailed
//! diagnostics to a user.
//!
//! ```no_run
//! use std::{fs, error::Error};
//! use err_ctx::ResultExt;
//! fn run() -> Result<(), Box<dyn Error>> {
//!     // An error here might display as "reading foo.txt: No such file or directory"
//!     let data = fs::read("foo.txt").ctx("reading foo.txt")?;
//!     // ...
//! #     Ok(())
//! }
//! ```

use std::error::Error;
use std::fmt;

/// An error providing context for some underlying cause.
#[derive(Debug)]
pub struct Context<C> {
    context: C,
    source: Box<dyn Error + Send + Sync>,
}

impl<C> Context<C> {
    pub fn new(context: C, source: Box<dyn Error + Send + Sync>) -> Self {
        Self { context, source }
    }
}

impl<C: fmt::Display> fmt::Display for Context<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.context.fmt(f)?;
        f.write_str(": ")?;
        self.source.fmt(f)
    }
}

impl<C: fmt::Debug + fmt::Display> Error for Context<C> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.source)
    }
}

pub trait ResultExt<T, E>
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    /// If this `Result` is an `Err`, wrap the error with `context`.
    fn ctx<D>(self, context: D) -> Result<T, Context<D>>;

    /// If this `Result` is an `Err`, invoke `f` and wrap the error with its result.
    fn with_ctx<D>(self, f: impl FnOnce(&E) -> D) -> Result<T, Context<D>>;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    fn ctx<D>(self, context: D) -> Result<T, Context<D>> {
        self.map_err(|e| e.ctx(context))
    }

    fn with_ctx<D>(self, f: impl FnOnce(&E) -> D) -> Result<T, Context<D>> {
        self.map_err(|e| {
            let context = f(&e);
            e.ctx(context)
        })
    }
}

pub trait ErrorExt {
    /// Construct a `Context` wrapping this error.
    fn ctx<D>(self, context: D) -> Context<D>;
}

impl<T: Into<Box<Error + Send + Sync>>> ErrorExt for T {
    fn ctx<D>(self, context: D) -> Context<D> {
        Context {
            context,
            source: self.into(),
        }
    }
}
