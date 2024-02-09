// SPDX-License-Identifier: Apache-2.0

use std::io::Write;

const UDEV_RULE_PATH: &str = "/etc/udev/rules.d/98-sriov-operator.rules";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() != 3 {
        println!("Usage: {} set|unset <interface_name>", argv[0]);
        return Ok(());
    }
    let action = &argv[1];
    let iface_name = &argv[2];

    match action.as_str() {
        "set" => create_udev_rules(iface_name)?,
        "unset" => remove_udev_rules()?,
        _ => {
            println!("Usage: {} set|unset <interface_name>", argv[0]);
            return Ok(());
        }
    }

    reload_udev_rules_and_trigger(iface_name)?;
    get_udev_property(iface_name)?;

    Ok(())
}

fn remove_udev_rules() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::remove_file(UDEV_RULE_PATH)?;
    Ok(())
}

fn create_udev_rules(
    iface_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let udev_rule_content = format!(
        r#"ENV{{INTERFACE}}=="{}", ENV{{NM_SRIOV_UNMANAGED}}="1""#,
        iface_name
    );

    let mut fd = std::fs::File::create(UDEV_RULE_PATH)?;
    fd.write_all(udev_rule_content.as_bytes())?;
    Ok(())
}

fn reload_udev_rules_and_trigger(
    iface_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    std::process::Command::new("udevadm")
        .args(["control", "--reload-rules"])
        .output()?;

    std::process::Command::new("udevadm")
        .args(["trigger", "--action", "change"])
        .arg(format!("/sys/class/net/{iface_name}"))
        .output()?;

    std::process::Command::new("udevadm")
        .arg("settle")
        .output()?;
    Ok(())
}

fn get_udev_property(
    iface_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = libudev::Context::new()?;
    let dev_sys_path = format!("/sys/class/net/{}", iface_name);
    let dev_path = std::path::Path::new(&dev_sys_path);

    let udev_dev = libudev::Device::from_syspath(&context, &dev_path)?;

    println!(
        "NM_SRIOV_UNMANAGED = {:?}",
        udev_dev.property_value("NM_SRIOV_UNMANAGED")
    );
    Ok(())
}
