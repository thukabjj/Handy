use crate::error::HandyError;
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// A trait for safely locking synchronization primitives with proper error handling.
///
/// This trait provides a way to handle mutex poisoning gracefully by returning
/// a `HandyError` instead of panicking with `unwrap()`.
pub trait SafeLock<T> {
    /// Safely acquire a lock, returning a `HandyError` if the lock is poisoned.
    fn safe_lock(&self) -> Result<MutexGuard<'_, T>, HandyError>;
}

impl<T> SafeLock<T> for Mutex<T> {
    fn safe_lock(&self) -> Result<MutexGuard<'_, T>, HandyError> {
        self.lock().map_err(|e| {
            HandyError::state("Internal state lock failed")
                .with_details(format!("Mutex poisoned: {}", e))
        })
    }
}

/// A trait for safely locking RwLock primitives with proper error handling.
pub trait SafeRwLock<T> {
    /// Safely acquire a read lock, returning a `HandyError` if the lock is poisoned.
    fn safe_read(&self) -> Result<RwLockReadGuard<'_, T>, HandyError>;

    /// Safely acquire a write lock, returning a `HandyError` if the lock is poisoned.
    fn safe_write(&self) -> Result<RwLockWriteGuard<'_, T>, HandyError>;
}

impl<T> SafeRwLock<T> for RwLock<T> {
    fn safe_read(&self) -> Result<RwLockReadGuard<'_, T>, HandyError> {
        self.read().map_err(|e| {
            HandyError::state("Internal state read lock failed")
                .with_details(format!("RwLock poisoned: {}", e))
        })
    }

    fn safe_write(&self) -> Result<RwLockWriteGuard<'_, T>, HandyError> {
        self.write().map_err(|e| {
            HandyError::state("Internal state write lock failed")
                .with_details(format!("RwLock poisoned: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_safe_lock_success() {
        let mutex = Mutex::new(42);
        let guard = mutex.safe_lock();
        assert!(guard.is_ok());
        assert_eq!(*guard.unwrap(), 42);
    }

    #[test]
    fn test_safe_lock_poisoned() {
        let mutex = Arc::new(Mutex::new(42));
        let mutex_clone = mutex.clone();

        // Poison the mutex by panicking while holding the lock
        let handle = thread::spawn(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("Intentional panic to poison mutex");
        });

        let _ = handle.join(); // Wait for the panic

        // Now try to safely lock
        let result = mutex.safe_lock();
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.category, crate::error::ErrorCategory::State);
        assert!(error.details.is_some());
    }

    #[test]
    fn test_safe_rwlock_read() {
        let rwlock = RwLock::new("test");
        let guard = rwlock.safe_read();
        assert!(guard.is_ok());
        assert_eq!(*guard.unwrap(), "test");
    }

    #[test]
    fn test_safe_rwlock_write() {
        let rwlock = RwLock::new(0);
        {
            let mut guard = rwlock.safe_write().unwrap();
            *guard = 100;
        }
        let guard = rwlock.safe_read().unwrap();
        assert_eq!(*guard, 100);
    }
}
