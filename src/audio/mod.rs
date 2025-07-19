pub mod capture;
pub mod playback;
pub mod processing;

use anyhow::Result;
use tracing::info;

use crate::config::AppConfig;

pub struct AudioManager {
    config: AppConfig,
    running: bool,
}

impl AudioManager {
    pub fn new(config: AppConfig) -> Self {
        info!("Initializing Audio Manager");
        
        Self {
            config,
            running: false,
        }
    }
    
    pub fn start(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }
        
        info!("Starting Audio Manager");
        self.running = true;
        
        Ok(())
    }
    
    pub fn stop(&mut self) {
        if !self.running {
            return;
        }
        
        info!("Stopping Audio Manager");
        self.running = false;
    }
    
    pub fn is_running(&self) -> bool {
        self.running
    }
}
