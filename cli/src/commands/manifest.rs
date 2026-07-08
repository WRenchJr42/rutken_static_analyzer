use ir::{ApkIR, Manifest};

pub fn render(ir: &ApkIR) -> String {
    let mut output = String::new();
    let Some(manifest) = &ir.manifest else {
        output.push_str("<manifest>\n</manifest>\n");
        return output;
    };

    render_manifest(manifest, &mut output);
    output
}

fn render_manifest(manifest: &Manifest, output: &mut String) {
    output.push_str("<manifest");
    if let Some(package) = &manifest.package {
        output.push_str(&format!(" package=\"{}\"", package));
    }
    output.push_str(">\n");

    for permission in &manifest.permissions {
        output.push_str(&format!(
            "  <uses-permission android:name=\"{}\" />\n",
            permission
        ));
    }

    if let Some(app) = &manifest.application {
        output.push_str("  <application");
        if let Some(label) = &app.label {
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
            for activity in &app.activities {
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
}
