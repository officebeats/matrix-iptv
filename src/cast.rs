//! Chromecast casting support for Matrix IPTV
//! 
//! This module provides functionality for discovering and streaming to Chromecast devices.

#[cfg(not(target_arch = "wasm32"))]
use std::net::IpAddr;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use mdns_sd::{ServiceDaemon, ServiceEvent};
#[cfg(not(target_arch = "wasm32"))]
use rust_cast::{CastDevice as RustCastDevice, ChannelMessage};
#[cfg(not(target_arch = "wasm32"))]
use rust_cast::channels::media::{Media, StreamType};
#[cfg(not(target_arch = "wasm32"))]
use rust_cast::channels::receiver::CastDeviceApp;

/// Represents a discovered Chromecast device
#[derive(Debug, Clone)]
pub struct CastDevice {
    /// Friendly name of the device (e.g., "Living Room TV")
    pub name: String,
    /// IP address of the device
    pub ip: String,
    /// Port number (usually 8009)
    pub port: u16,
    /// Model name if available
    pub model: Option<String>,
}

impl CastDevice {
    /// Create a new CastDevice
    pub fn new(name: String, ip: String, port: u16) -> Self {
        Self {
            name,
            ip,
            port,
            model: None,
        }
    }
}

/// Playback target for streams
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackTarget {
    /// Play locally via mpv
    Local,
    /// Cast to a Chromecast device
    Chromecast(CastDevice),
}

impl PartialEq for CastDevice {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && self.port == other.port
    }
}

impl Default for PlaybackTarget {
    fn default() -> Self {
        PlaybackTarget::Local
    }
}

/// Manager for Chromecast casting operations
#[cfg(not(target_arch = "wasm32"))]
pub struct CastManager {
    /// Currently active cast connection
    connection: Option<ActiveCast>,
}

#[cfg(not(target_arch = "wasm32"))]
struct ActiveCast {
    device: CastDevice,
    // Future: Could hold the CastDevice connection for control
}

#[cfg(not(target_arch = "wasm32"))]
impl CastManager {
    /// Create a new CastManager
    pub fn new() -> Self {
        Self { connection: None }
    }

    /// Discover Chromecast devices on the local network
    /// 
    /// Scans for the specified duration and returns all found devices.
    pub async fn discover_devices(timeout_secs: u64) -> Result<Vec<CastDevice>, anyhow::Error> {
        use tokio::time::sleep;
        
        let mdns = ServiceDaemon::new()
            .map_err(|e| anyhow::anyhow!("Failed to create mDNS daemon: {}", e))?;
        
        // Chromecast uses _googlecast._tcp.local.
        let receiver = mdns
            .browse("_googlecast._tcp.local.")
            .map_err(|e| anyhow::anyhow!("Failed to browse for Chromecast devices: {}", e))?;
        
        let mut devices = Vec::new();
        let timeout = Duration::from_secs(timeout_secs);
        let start = std::time::Instant::now();
        
        // Collect devices for the timeout duration
        while start.elapsed() < timeout {
            match receiver.try_recv() {
                Ok(event) => {
                    if let ServiceEvent::ServiceResolved(info) = event {
                        // Extract device info
                        let name = info.get_property_val_str("fn")
                            .unwrap_or_else(|| info.get_fullname())
                            .to_string();
                        
                        let model = info.get_property_val_str("md")
                            .map(|s| s.to_string());
                        
                        // Get IP addresses
                        for addr in info.get_addresses() {
                            if let IpAddr::V4(ipv4) = addr {
                                let mut device = CastDevice::new(
                                    name.clone(),
                                    ipv4.to_string(),
                                    info.get_port(),
                                );
                                device.model = model.clone();
                                
                                // Avoid duplicates
                                if !devices.iter().any(|d: &CastDevice| d.ip == device.ip) {
                                    devices.push(device);
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    // No message yet, wait a bit
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
        
        // Stop browsing
        let _ = mdns.stop_browse("_googlecast._tcp.local.");
        let _ = mdns.shutdown();
        
        Ok(devices)
    }

    /// Cast a media URL to a Chromecast device
    pub fn cast_to_device(&mut self, device: &CastDevice, url: &str, title: Option<&str>) -> Result<(), anyhow::Error> {
        // Connect to the Chromecast
        let cast_device = RustCastDevice::connect_without_host_verification(
            &device.ip, 
            device.port
        ).map_err(|e| anyhow::anyhow!("Failed to connect to Chromecast '{}': {}", device.name, e))?;
        
        // Connect to receiver
        cast_device.connection.connect("receiver-0")
            .map_err(|e| anyhow::anyhow!("Failed to connect to receiver: {}", e))?;
        
        // Launch the default media receiver app
        let app = cast_device.receiver.launch_app(&CastDeviceApp::DefaultMediaReceiver)
            .map_err(|e| anyhow::anyhow!("Failed to launch media receiver: {}", e))?;
        
        // Connect to the media receiver
        cast_device.connection.connect(&app.transport_id)
            .map_err(|e| anyhow::anyhow!("Failed to connect to media receiver: {}", e))?;
        
        // Create media info
        let media = Media {
            content_id: url.to_string(),
            content_type: "video/mp4".to_string(), // Generic, Chromecast will handle it
            stream_type: StreamType::Live,
            duration: None,
            metadata: None,
        };
        
        // Load and play the media
        cast_device.media.load(
            &app.transport_id,
            &app.session_id,
            &media,
        ).map_err(|e| anyhow::anyhow!("Failed to load media: {}", e))?;
        
        // Store the active connection info
        self.connection = Some(ActiveCast {
            device: device.clone(),
        });
        
        Ok(())
    }

    /// Stop the current cast session
    pub fn stop_cast(&mut self) -> Result<(), anyhow::Error> {
        if let Some(active) = self.connection.take() {
            // Reconnect to stop
            if let Ok(cast_device) = RustCastDevice::connect_without_host_verification(
                &active.device.ip,
                active.device.port
            ) {
                let _ = cast_device.connection.connect("receiver-0");
                let _ = cast_device.receiver.stop_app("receiver-0");
            }
        }
        Ok(())
    }

    /// Check if currently casting
    pub fn is_casting(&self) -> bool {
        self.connection.is_some()
    }

    /// Get the current cast device if any
    pub fn current_device(&self) -> Option<&CastDevice> {
        self.connection.as_ref().map(|c| &c.device)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for CastManager {
    fn default() -> Self {
        Self::new()
    }
}

// WASM stub implementation
#[cfg(target_arch = "wasm32")]
pub struct CastManager;

#[cfg(target_arch = "wasm32")]
impl CastManager {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn discover_devices(_timeout_secs: u64) -> Result<Vec<CastDevice>, anyhow::Error> {
        Err(anyhow::anyhow!("Casting not supported in browser"))
    }
    
    pub fn cast_to_device(&mut self, _device: &CastDevice, _url: &str, _title: Option<&str>) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!("Casting not supported in browser"))
    }
    
    pub fn stop_cast(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }
    
    pub fn is_casting(&self) -> bool {
        false
    }
    
    pub fn current_device(&self) -> Option<&CastDevice> {
        None
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for CastManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cast_device_equality() {
        let d1 = CastDevice::new("TV 1".into(), "192.168.1.100".into(), 8009);
        let d2 = CastDevice::new("TV 1".into(), "192.168.1.100".into(), 8009);
        let d3 = CastDevice::new("TV 2".into(), "192.168.1.101".into(), 8009);
        
        assert_eq!(d1, d2);
        assert_ne!(d1, d3);
    }

    #[test]
    fn test_playback_target_default() {
        let target = PlaybackTarget::default();
        assert_eq!(target, PlaybackTarget::Local);
    }
}
