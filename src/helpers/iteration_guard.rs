#[derive(Eq, PartialEq)]
pub enum IterationResult<T> {
    Done(T),
    Stopped,
}

impl<T> IterationResult<T> {
    #[inline]
    pub const fn is_done(&self) -> bool {
        matches!(*self, Self::Done(_))
    }

    #[inline]
    pub const fn is_stopped(&self) -> bool {
        matches!(*self, Self::Stopped)
    }

    #[inline]
    pub fn unwrap(self) -> T {
        match self {
            Self::Done(t) => t,
            Self::Stopped => panic!("called `IterationResult::unwrap()` on an `Stop` value"),
        }
    }
}

#[macro_export]
macro_rules! unwrap_or_stop {
    ($value:expr) => {
        match $value {
            IterationResult::Stopped => {
                return Ok(IterationResult::Stopped);
            }
            IterationResult::Done(inner) => inner,
        }
    };
}

#[macro_export]
macro_rules! unwrap_or_done {
    ($option_expr:expr, $done_val:expr) => {
        match $option_expr {
            Some(data) => data,
            None => return Ok(IterationResult::Done($done_val)),
        }
    };
}

pub struct IterationGuard {
    counter: u32,
    max_iterations: u32,
}

impl IterationGuard {
    pub fn new(max_iterations: u32) -> Self {
        IterationGuard {
            counter: 0,
            max_iterations,
        }
    }

    pub fn next_iteration(&mut self) -> IterationResult<()> {
        self.counter += 1;
        if self.counter >= self.max_iterations {
            return IterationResult::Stopped;
        }
        IterationResult::Done(())
    }

    pub fn n_iterations(&self) -> u32 {
        self.counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::IterationResult::{Done, Stopped};

    #[test]
    fn iteration_gurad() {
        let mut iteration_guard = IterationGuard::new(2);

        assert_eq!(iteration_guard.n_iterations(), 0);

        let res = iteration_guard.next_iteration();
        assert!(res == Done(()));
        assert!(res.is_done());
        assert!(!res.is_stopped());

        assert_eq!(iteration_guard.n_iterations(), 1);

        let res = iteration_guard.next_iteration();
        assert!(res == Stopped);
        assert!(res.is_stopped());
        assert!(!res.is_done());

        assert_eq!(iteration_guard.n_iterations(), 2);
    }
}
