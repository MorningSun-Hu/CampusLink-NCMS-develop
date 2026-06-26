use anyhow::Result;
use tracing::{info, warn};

pub struct Locker {
    super_password: String,
    attempts: u32,
    max_attempts: u32,
}

impl Locker {
    pub fn new(super_password: &str) -> Self {
        Self {
            super_password: super_password.to_string(),
            attempts: 0,
            max_attempts: 5,
        }
    }

    pub fn try_unlock(&mut self, input: &str) -> UnlockResult {
        if input == self.super_password {
            info!("Screen unlocked successfully - audit: valid password entered");
            UnlockResult::Success
        } else {
            self.attempts += 1;
            warn!(
                "Failed unlock attempt {}/{}",
                self.attempts, self.max_attempts
            );
            if self.attempts >= self.max_attempts {
                info!("Lock cooldown activated - {} failed attempts", self.attempts);
                UnlockResult::Cooldown {
                    remaining_seconds: 30,
                }
            } else {
                UnlockResult::Failure {
                    attempts: self.attempts,
                    max_attempts: self.max_attempts,
                }
            }
        }
    }

    pub fn reset_attempts(&mut self) {
        self.attempts = 0;
    }
}

pub enum UnlockResult {
    Success,
    Failure { attempts: u32, max_attempts: u32 },
    Cooldown { remaining_seconds: u64 },
}
