use bluer::agent::{RequestConfirmation, RequestPasskey};
use std::sync::Arc;

/// Creates a minimal auto-accept pairing agent with `KeyboardDisplay` capability.
///
/// The agent automatically accepts all passkey and confirmation requests.
/// This is suitable for testing; a production app should prompt the user.
fn create_agent() -> bluer::agent::Agent {
    bluer::agent::Agent {
        request_default: true,
        // Provide a fixed passkey for KeyboardOnly peripherals.
        request_passkey: Some(Box::new(|_req: RequestPasskey| {
            Box::pin(async move { Ok(123456u32) })
        })),
        // Auto-accept numeric comparison confirmations.
        request_confirmation: Some(Box::new(|_req: RequestConfirmation| {
            Box::pin(async move { Ok(()) })
        })),
        // Display passkey — no-op (just log if needed).
        display_passkey: Some(Box::new(|req| {
            Box::pin(async move {
                eprintln!(
                    "Pairing passkey for {}: {:06} ({} digits entered)",
                    req.device, req.passkey, req.entered
                );
                Ok(())
            })
        })),
        ..Default::default()
    }
}

/// A wrapper for a registered agent handle that can be shared across threads.
pub type AgentHandle = Arc<tokio::sync::Mutex<Option<bluer::agent::AgentHandle>>>;

/// Register our auto-accept agent with `BlueZ` and return the handle.
pub async fn register_agent(session: &bluer::Session) -> Result<AgentHandle, String> {
    let agent = create_agent();
    let handle = session
        .register_agent(agent)
        .await
        .map_err(|e| format!("Failed to register pairing agent: {e}"))?;
    Ok(Arc::new(tokio::sync::Mutex::new(Some(handle))))
}
