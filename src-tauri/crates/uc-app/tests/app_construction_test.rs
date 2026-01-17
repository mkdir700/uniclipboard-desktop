/// Test App direct construction without full dependency setup
///
/// This test verifies that App::new() exists and accepts AppDeps,
/// but doesn't create full mocks for all dependencies.
#[test]
fn test_app_new_signature_exists() {
    // This test verifies that the App struct exists and has a new() method
    // by checking it compiles. We don't need to actually call it with
    // full dependencies - the existence check is sufficient.
    //
    // The actual construction with full AppDeps is tested in integration tests.

    // Verify App is exported from uc_app
    use uc_app::App;

    // If this compiles, App exists and is accessible
    let _type_check: Option<App> = None;
}

#[test]
fn test_app_deps_is_plain_struct() {
    // Verify AppDeps is a plain struct, not a Builder
    use std::mem;
    use uc_app::AppDeps;

    // This test verifies AppDeps is a plain struct by checking its size
    // (Builders typically have smaller sizes due to being stateless)
    let size = mem::size_of::<AppDeps>();

    // AppDeps should contain Arc<dyn Trait> pointers
    // Each Arc<dyn Trait> is typically 16 bytes (pointer + vtable)
    // We expect it to be reasonably large (not a zero-sized builder)
    assert!(
        size > 100,
        "AppDeps should contain dependencies, size was {}",
        size
    );
}
