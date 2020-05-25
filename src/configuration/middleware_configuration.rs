use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;

/**
 * Wrapper for the middleware configurations.
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration {
    ///Stack size of the spawned Reader/Sender threads in bytes.
    pub thread_stack_size: usize,

    ///Stack size of the main Middleware thread in bytes.
    pub middleware_thread_stack_size: usize,

    ///Timeout in microseconds used by the Sender thread for reading messages from the channel.
    pub stream_sender_timeout: u64,

    ///Stability calculation flag.
    pub track_causal_stability: bool,

    ///Parameters that set message batching.
    pub batching: Batching,
}

impl Configuration {
    /**
     * Returns the timeout wrapped in a Duration.
     */
    pub fn get_stream_sender_timeout(&self) -> Duration {
        Duration::from_micros(self.stream_sender_timeout)
    }
}

/**
 * Reads the middleware configuration from a TOML file.
 * An error is returned if not successful.
 *
 * # Arguments
 *
 * `configuration_file_path` - path to the TOML configuration file.
 */
pub fn read_configuration_file(
    configuration_file_path: String,
) -> Result<Configuration, Box<dyn Error>> {
    let mut configuration_string = String::new();
    let mut file = File::open(configuration_file_path)?;

    file.read_to_string(&mut configuration_string)?;
    let configuration: Configuration = toml::from_str(&configuration_string)?;

    Ok(configuration)
}

/**
 * Configuration parameters for the Sender threads message batching.
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Batching {
    ///Number of bytes to be buffered before calling Write.
    pub size: u64,

    ///Number of messages to be buffered before calling Write.
    pub message_number: usize,

    ///Lower value of the ACK timeout in microseconds.
    pub lower_timeout: u64,

    ///Upper value of the ACK timeout in microseconds.
    pub upper_timeout: u64,
}

impl Batching {
    /**
     * Returns the timeout wrapped in a Duration.
     */
    pub fn get_lower_timeout(&self) -> Duration {
        Duration::from_micros(self.lower_timeout)
    }

    /**
     * Returns the timeout wrapped in a Duration.
     */
    pub fn get_upper_timeout(&self) -> Duration {
        Duration::from_micros(self.upper_timeout)
    }
}
