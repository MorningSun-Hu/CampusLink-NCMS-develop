use anyhow::Result;
use tracing::info;

pub struct LockManager {
    is_locked: bool,
}

impl LockManager {
    pub fn new() -> Self {
        Self { is_locked: false }
    }

    pub async fn lock(&mut self) -> Result<()> {
        if self.is_locked {
            info!("Already locked");
            return Ok(());
        }
        info!("Screen lock engaged");
        self.is_locked = true;
        Ok(())
    }

    pub async fn unlock(&mut self, password: &str, correct_password: &str) -> Result<bool> {
        if !self.is_locked {
            info!("Already unlocked");
            return Ok(true);
        }
        if password == correct_password {
            info!("Screen unlocked successfully");
            self.is_locked = false;
            Ok(true)
        } else {
            info!("Unlock attempt with incorrect password");
            Ok(false)
        }
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }
}

impl Default for LockManager {
    fn default() -> Self {
        Self::new()
    }
}
