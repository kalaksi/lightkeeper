mod common;
use lightkeeper;

extern crate qmetaobject;
use qmetaobject::*;


#[test]
fn test_smoke() {
    let (config_dir, main_config, hosts_config, group_config) = unsafe { common::setup() };
    let (_, mut engine) = lightkeeper::run(&config_dir, &main_config, &hosts_config, &group_config, true);

    let result = engine.invoke_method(QByteArray::from("test"), &[]);
    let text = result.to_qbytearray().to_string();
    assert_eq!(text, "OK");
}