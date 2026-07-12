use crate::axml::node::XmlNode;

use crate::manifest::model::{Activity, AndroidManifest, Application};

pub fn parse_manifest(root: &XmlNode) -> AndroidManifest {
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
                            app.label = Some(attr.value.clone());
                        }
                        "debuggable" => {
                            app.debuggable = attr.value == "true";
                        }
                        "allowBackup" => {
                            app.allow_backup = attr.value == "true";
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
                                    activity.name = attr.value.clone();
                                }
                                "exported" => {
                                    activity.exported = Some(attr.value == "true");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axml::resolve::ResolvedAttribute;

    fn make_attr(name: &str, value: &str) -> ResolvedAttribute {
        ResolvedAttribute {
            namespace: None,
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    fn make_node(name: &str, attributes: Vec<ResolvedAttribute>) -> XmlNode {
        XmlNode::new(name.to_string(), attributes)
    }

    #[test]
    fn parse_manifest_extracts_package_name() {
        let root = make_node("manifest", vec![
            make_attr("package", "com.example.app"),
        ]);
        let manifest = parse_manifest(&root);
        assert_eq!(manifest.package, Some("com.example.app".to_string()));
    }

    #[test]
    fn parse_manifest_empty_root_produces_defaults() {
        let root = make_node("manifest", vec![]);
        let manifest = parse_manifest(&root);
        assert_eq!(manifest.package, None);
        assert_eq!(manifest.min_sdk, None);
        assert_eq!(manifest.target_sdk, None);
        assert!(manifest.permissions.is_empty());
        assert!(manifest.application.is_none());
    }

    #[test]
    fn parse_manifest_extracts_uses_sdk() {
        let mut root = make_node("manifest", vec![]);
        let uses_sdk = make_node("uses-sdk", vec![
            make_attr("minSdkVersion", "21"),
            make_attr("targetSdkVersion", "33"),
        ]);
        root.children.push(uses_sdk);
        let manifest = parse_manifest(&root);
        assert_eq!(manifest.min_sdk, Some(21));
        assert_eq!(manifest.target_sdk, Some(33));
    }

    #[test]
    fn parse_manifest_uses_sdk_with_non_numeric_values() {
        let mut root = make_node("manifest", vec![]);
        let uses_sdk = make_node("uses-sdk", vec![
            make_attr("minSdkVersion", "invalid"),
            make_attr("targetSdkVersion", "also_invalid"),
        ]);
        root.children.push(uses_sdk);
        let manifest = parse_manifest(&root);
        assert_eq!(manifest.min_sdk, None);
        assert_eq!(manifest.target_sdk, None);
    }

    #[test]
    fn parse_manifest_extracts_permissions() {
        let mut root = make_node("manifest", vec![]);
        let perm1 = make_node("uses-permission", vec![
            make_attr("name", "android.permission.INTERNET"),
        ]);
        let perm2 = make_node("uses-permission", vec![
            make_attr("name", "android.permission.CAMERA"),
        ]);
        root.children.push(perm1);
        root.children.push(perm2);
        let manifest = parse_manifest(&root);
        assert_eq!(manifest.permissions.len(), 2);
        assert!(manifest.permissions.contains(&"android.permission.INTERNET".to_string()));
        assert!(manifest.permissions.contains(&"android.permission.CAMERA".to_string()));
    }

    #[test]
    fn parse_manifest_application_flags() {
        let mut root = make_node("manifest", vec![]);
        let app = make_node("application", vec![
            make_attr("label", "@string/app_name"),
            make_attr("debuggable", "true"),
            make_attr("allowBackup", "false"),
        ]);
        root.children.push(app);
        let manifest = parse_manifest(&root);
        assert!(manifest.application.is_some());
        let app = manifest.application.unwrap();
        assert_eq!(app.label, Some("@string/app_name".to_string()));
        assert!(app.debuggable);
        assert!(!app.allow_backup);
    }

    #[test]
    fn parse_manifest_activities() {
        let mut root = make_node("manifest", vec![]);
        let mut app = make_node("application", vec![]);
        let activity1 = make_node("activity", vec![
            make_attr("name", "com.example.MainActivity"),
            make_attr("exported", "true"),
        ]);
        let activity2 = make_node("activity", vec![
            make_attr("name", "com.example.SecondActivity"),
        ]);
        app.children.push(activity1);
        app.children.push(activity2);
        root.children.push(app);
        let manifest = parse_manifest(&root);
        assert!(manifest.application.is_some());
        let app = manifest.application.unwrap();
        assert_eq!(app.activities.len(), 2);
        assert_eq!(app.activities[0].name, "com.example.MainActivity");
        assert_eq!(app.activities[0].exported, Some(true));
        assert_eq!(app.activities[1].name, "com.example.SecondActivity");
        assert_eq!(app.activities[1].exported, None);
    }

    #[test]
    fn parse_manifest_application_debuggable_false() {
        let mut root = make_node("manifest", vec![]);
        let app = make_node("application", vec![
            make_attr("debuggable", "false"),
        ]);
        root.children.push(app);
        let manifest = parse_manifest(&root);
        let app = manifest.application.unwrap();
        assert!(!app.debuggable);
    }

    #[test]
    fn parse_manifest_activity_exported_false() {
        let mut root = make_node("manifest", vec![]);
        let mut app = make_node("application", vec![]);
        let activity = make_node("activity", vec![
            make_attr("name", "com.example.MainActivity"),
            make_attr("exported", "false"),
        ]);
        app.children.push(activity);
        root.children.push(app);
        let manifest = parse_manifest(&root);
        let app = manifest.application.unwrap();
        assert_eq!(app.activities[0].exported, Some(false));
    }

    #[test]
    fn parse_manifest_full_example() {
        let mut root = make_node("manifest", vec![
            make_attr("package", "com.example.myapp"),
        ]);

        let uses_sdk = make_node("uses-sdk", vec![
            make_attr("minSdkVersion", "21"),
            make_attr("targetSdkVersion", "33"),
        ]);
        root.children.push(uses_sdk);

        let perm = make_node("uses-permission", vec![
            make_attr("name", "android.permission.INTERNET"),
        ]);
        root.children.push(perm);

        let mut app = make_node("application", vec![
            make_attr("debuggable", "true"),
            make_attr("allowBackup", "true"),
        ]);
        let activity = make_node("activity", vec![
            make_attr("name", "com.example.MainActivity"),
            make_attr("exported", "true"),
        ]);
        app.children.push(activity);
        root.children.push(app);

        let manifest = parse_manifest(&root);
        assert_eq!(manifest.package, Some("com.example.myapp".to_string()));
        assert_eq!(manifest.min_sdk, Some(21));
        assert_eq!(manifest.target_sdk, Some(33));
        assert_eq!(manifest.permissions.len(), 1);
        assert!(manifest.application.is_some());
        let app = manifest.application.unwrap();
        assert!(app.debuggable);
        assert!(app.allow_backup);
        assert_eq!(app.activities.len(), 1);
        assert_eq!(app.activities[0].name, "com.example.MainActivity");
        assert_eq!(app.activities[0].exported, Some(true));
    }
}
