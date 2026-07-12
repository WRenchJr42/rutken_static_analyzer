use apk::axml::parser::AxmlParser;
use apk::dex::model::build_dex_model;
use apk::errors::ApkError;
use apk::manifest::model as apk_manifest;
use apk::manifest::parser::parse_manifest;
use apk::reader::ApkContainer;
use ir::{Activity, ApkIR, Application, DexFile, Manifest, Metadata};

use crate::error::EngineError;
use crate::lower::dex::convert_dex_model;

pub(crate) struct IrBuilder<'a> {
    container: &'a ApkContainer,
}

impl<'a> IrBuilder<'a> {
    pub(crate) fn new(container: &'a ApkContainer) -> Self {
        Self { container }
    }

    pub(crate) fn build(&self) -> Result<ApkIR, EngineError> {
        Ok(ApkIR {
            metadata: self.build_metadata(),
            manifest: Some(self.build_manifest()?),
            dex_files: self.build_dex_files()?,
            findings: Vec::new(),
        })
    }

    fn build_metadata(&self) -> Metadata {
        Metadata {
            sha256: Some(self.container.sha256.clone()),
            size: Some(self.container.file_size),
            dex_files: self
                .container
                .dex_files
                .iter()
                .map(|dex| dex.name.clone())
                .collect(),
            architectures: self.container.architectures.clone(),
        }
    }

    fn build_manifest(&self) -> Result<Manifest, ApkError> {
        let document = AxmlParser::parse(&self.container.manifest)?;
        let manifest = match document.root {
            Some(root) => parse_manifest(&root),
            None => apk_manifest::AndroidManifest {
                package: None,
                min_sdk: None,
                target_sdk: None,
                permissions: Vec::new(),
                application: None,
            },
        };

        Ok(convert_manifest(manifest))
    }

    fn build_dex_files(&self) -> Result<Vec<DexFile>, ApkError> {
        self.container
            .dex_files
            .iter()
            .map(|dex_file| {
                let model = build_dex_model(dex_file.name.clone(), &dex_file.bytes)?;
                Ok(convert_dex_model(model))
            })
            .collect()
    }
}

fn convert_manifest(manifest: apk_manifest::AndroidManifest) -> Manifest {
    Manifest {
        package: manifest.package,
        min_sdk: manifest.min_sdk,
        target_sdk: manifest.target_sdk,
        permissions: manifest.permissions,
        application: manifest.application.map(convert_application),
    }
}

fn convert_application(application: apk_manifest::Application) -> Application {
    Application {
        label: application.label,
        debuggable: application.debuggable,
        allow_backup: application.allow_backup,
        activities: application
            .activities
            .into_iter()
            .map(convert_activity)
            .collect(),
    }
}

fn convert_activity(activity: apk_manifest::Activity) -> Activity {
    Activity {
        name: activity.name,
        exported: activity.exported,
    }
}
