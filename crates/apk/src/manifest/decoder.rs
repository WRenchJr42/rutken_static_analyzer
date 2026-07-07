use crate::axml::node::XmlNode;

use crate::manifest::model::{
    AndroidManifest,
    Application,
    Activity,
};


pub fn decode_manifest(root: &XmlNode) -> AndroidManifest {
    let mut manifest = AndroidManifest {
        package: None,
        min_sdk: None,
        target_sdk: None,
        permissions: Vec::new(),
        application: None,
    };

    for attr in &root.attributes {
        if attr.name == "package" {
            manifest.package = Some(attr.value.clone());
        }
    }


    for child in &root.children {
        match child.name.as_str() {
            "uses-sdk" => {
                for attr in &child.attributes {
                    match attr.name.as_str() {
                        "minSdkVersion" => {
                            manifest.min_sdk = attr.value.parse().ok();
                        }
                        "targetSdkVersion" => {
                            manifest.target_sdk = attr.value.parse().ok();
                        }

                        _ => {}

                    }
                }
            }


            "uses-permission" => {
                for attr in &child.attributes {
                    if attr.name == "name" {
                        manifest.permissions.push(attr.value.clone());
                    }
                }
            }

            "application" => {
                let mut app = Application {
                    label: None,
                    debuggable: false,
                    allow_backup: false,
                    activities: Vec::new(),
                };

                for attr in &child.attributes {
                    match attr.name.as_str() {
                        "label" => {
                            app.label =
                                Some(attr.value.clone());
                        }
                        "debuggable" => {
                            app.debuggable =
                                attr.value == "true";
                        }
                        "allowBackup" => {
                            app.allow_backup =
                                attr.value == "true";
                        }

                        _ => {}

                    }
                }

                for node in &child.children {
                    if node.name == "activity" {
                        let mut activity = Activity {
                            name: String::new(),
                            exported: None,
                        };

                        for attr in &node.attributes {
                            match attr.name.as_str() {
                                "name" => {
                                    activity.name =
                                        attr.value.clone();
                                }
                                "exported" => {
                                    activity.exported =
                                        Some(
                                            attr.value == "true"
                                        );
                                }

                                _ => {}
                            }
                        }
                        app.activities.push(activity);
                    }
                }
                manifest.application = Some(app);
            }
            _ => {}
        }
    }
    manifest
}
