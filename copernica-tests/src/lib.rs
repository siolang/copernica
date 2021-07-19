mod common;
mod unreliable_sequenced;
mod reliable_sequenced;
mod reliable_ordered;
pub use {
    unreliable_sequenced::{unreliable_sequenced_ping_pong},
    reliable_sequenced::{reliable_sequenced_ping_pong},
    reliable_ordered::{reliable_ordered_ping_pong},
};
use {
    anyhow::{Result, anyhow},
    copernica_common::{LogEntry},
    std::sync::mpsc::{Receiver},
    std::collections::HashMap,
};
pub fn process_network(mut expected_behaviour: HashMap<LogEntry, i32>, receiver: Receiver<LogEntry>) -> Result<()> {
    let ref_expected_behaviour = expected_behaviour.clone();
    let mut error: String = "Corrections below:\n".into();
    loop {
        let log_entry = receiver.recv()?;
        match log_entry {
            LogEntry::Register { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::Message { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::FoundResponseUpstream { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::ResponseArrivedDownstream { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::ForwardResponseDownstream { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::ForwardRequestUpstream { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::End => {
                for (key, value) in &expected_behaviour {
                    if value != &0 {
                        if let Some(ref_value) = ref_expected_behaviour.get(key) {
                            error.push_str(&format!("{} {});\n", key, ref_value - value))
                        }
                    }
                }
                if error == "Corrections below:\n".to_string() {
                    break
                } else {
                    return Err(anyhow!("{}", error))
                }
            },
        }
    }
    Ok(())
}
