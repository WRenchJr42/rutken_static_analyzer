use crate::commands::parse_manifest;
use crate::errors::ApkError;
use crate::reader::ApkContainer;
use crate::axml::node::XmlNode;
use crate::axml::resolve::ResolvedAttribute;

pub fn render(container: &ApkContainer) -> Result<String, ApkError> {
    let manifest = parse_manifest(container)?;

    let mut output = String::new();
    output.push_str("<manifest");
    if let Some(package) = manifest.package {
        output.push_str(&format!(" package=\"{}\"", package));
    }
    output.push_str(">\n");

    for permission in manifest.permissions {
        output.push_str(&format!("  <uses-permission android:name=\"{}\" />\n", permission));
    }

    if let Some(app) = manifest.application {
        output.push_str("  <application");
        if let Some(label) = app.label {
            output.push_str(&format!(" android:label=\"{}\"", label));
        }
        if app.debuggable {
            output.push_str(" android:debuggable=\"true\"");
        }
        if app.allow_backup {
            output.push_str(" android:allowBackup=\"true\"");
        }
        if app.activities.is_empty() {
            output.push_str(" />\n");
        } else {
            output.push_str(">\n");
            for activity in app.activities {
                output.push_str(&format!("    <activity android:name=\"{}\"", activity.name));
                if let Some(exported) = activity.exported {
                    output.push_str(&format!(" android:exported=\"{}\"", exported));
                }
                output.push_str(" />\n");
            }
            output.push_str("  </application>\n");
        }
    }

    output.push_str("</manifest>\n");
    Ok(output)
}

#[allow(dead_code)]
fn render_node(_node: &XmlNode, _indent: usize) -> String {
    String::new()
}

#[allow(dead_code)]
fn format_attribute(attribute: &ResolvedAttribute) -> String {
    match &attribute.namespace {
        Some(namespace) => format!("{}:{}=\"{}\"", namespace, attribute.name, attribute.value),
        None => format!("{}=\"{}\"", attribute.name, attribute.value),
    }
}
