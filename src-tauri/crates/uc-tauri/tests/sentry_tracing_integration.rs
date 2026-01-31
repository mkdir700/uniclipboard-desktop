use sentry::Level;
use uc_tauri::bootstrap::tracing::init_tracing_subscriber;

#[test]
fn test_sentry_tracing_integration() {
    std::env::set_var("SENTRY_DSN", "https://public@example.com/1");
    std::env::set_var("RUST_LOG", "info");

    init_tracing_subscriber().expect("Failed to init tracing");

    let events = sentry::test::with_captured_events(|| {
        tracing::error!("This is a test error");
    });

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::Error);
    assert_eq!(events[0].message.as_deref(), Some("This is a test error"));

    let events = sentry::test::with_captured_events(|| {
        tracing::warn!("This is a warning breadcrumb");
        tracing::error!("This is an error with breadcrumb");
    });

    assert_eq!(events.len(), 1);
    let event = &events[0];

    let breadcrumbs: Vec<_> = event
        .breadcrumbs
        .iter()
        .filter(|b| b.message.as_deref() == Some("This is a warning breadcrumb"))
        .collect();

    assert_eq!(breadcrumbs.len(), 1);
    assert_eq!(breadcrumbs[0].level, Level::Warning);
}
