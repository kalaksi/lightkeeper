mod common;
// use lightkeeper;
// extern crate qmetaobject;
// use qmetaobject::*;


#[test]
fn test_smoke() {
    // Disabled for now. This doesn't work well for any tests that require the Qt event loop.
    // It seems that it'd be best if qmetaobject-rs added better support for testing.
    // Additionally, returning engine produces lots of warnings on exit. Maybe the engine outlives some properties.

    // let (config_dir, main_config, hosts_config, group_config) = unsafe { common::setup() };
    // let (_, mut engine) = lightkeeper::run(&config_dir, &main_config, &hosts_config, &group_config, true);

    // let result = engine.invoke_method(QByteArray::from("test"), &[]);
    // let text = result.to_qbytearray().to_string();
    // assert_eq!(text, "OK");
}