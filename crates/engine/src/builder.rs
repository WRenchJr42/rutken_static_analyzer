use apk::axml::parser::AxmlParser;
use apk::dex::instruction as apk_instruction;
use apk::dex::model::{DexModel, build_dex_model};
use apk::errors::ApkError;
use apk::manifest::model as apk_manifest;
use apk::manifest::parser::parse_manifest;
use apk::reader::ApkContainer;
use ir::{
    Activity, ApkIR, Application, BranchKind, Class, DexFile, Instruction, InvokeKind, Manifest,
    Metadata, Method,
};

use crate::error::EngineError;

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

fn convert_dex_model(model: DexModel) -> DexFile {
    DexFile {
        name: model.name,
        strings: model.strings,
        classes: model
            .classes
            .into_iter()
            .map(|class| Class {
                name: class.name,
                methods: class
                    .methods
                    .into_iter()
                    .map(|method| Method {
                        name: method.name,
                        access_flags: method.access_flags,
                        code_offset: (method.code_off != 0).then_some(method.code_off),
                        instructions: method
                            .instructions
                            .into_iter()
                            .map(convert_instruction)
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn convert_instruction(instruction: apk_instruction::Instruction) -> Instruction {
    match instruction {
        apk_instruction::Instruction::Const { register, value } => {
            Instruction::Const { register, value }
        }
        apk_instruction::Instruction::ConstString { register, value } => {
            Instruction::ConstString { register, value }
        }
        apk_instruction::Instruction::Invoke {
            kind,
            method,
            registers,
        } => Instruction::Invoke {
            kind: convert_invoke_kind(kind),
            method,
            registers,
        },
        apk_instruction::Instruction::FieldAccess { field } => Instruction::FieldAccess { field },
        apk_instruction::Instruction::NewInstance { class } => Instruction::NewInstance { class },
        apk_instruction::Instruction::CheckCast { class } => Instruction::CheckCast { class },
        apk_instruction::Instruction::MoveResult { register } => {
            Instruction::MoveResult { register }
        }
        apk_instruction::Instruction::Return => Instruction::Return,
        apk_instruction::Instruction::Throw => Instruction::Throw,
        apk_instruction::Instruction::Nop => Instruction::Nop,
        apk_instruction::Instruction::Payload => Instruction::Payload,
        apk_instruction::Instruction::Branch { kind } => Instruction::Branch {
            kind: convert_branch_kind(kind),
        },
        apk_instruction::Instruction::Unknown { opcode, raw } => {
            Instruction::Unknown { opcode, raw }
        }
    }
}

fn convert_invoke_kind(kind: apk_instruction::InvokeKind) -> InvokeKind {
    match kind {
        apk_instruction::InvokeKind::Static => InvokeKind::Static,
        apk_instruction::InvokeKind::Virtual => InvokeKind::Virtual,
        apk_instruction::InvokeKind::Direct => InvokeKind::Direct,
        apk_instruction::InvokeKind::Super => InvokeKind::Super,
        apk_instruction::InvokeKind::Interface => InvokeKind::Interface,
    }
}

fn convert_branch_kind(kind: apk_instruction::BranchKind) -> BranchKind {
    match kind {
        apk_instruction::BranchKind::Goto => BranchKind::Goto,
        apk_instruction::BranchKind::IfEqz => BranchKind::IfEqz,
        apk_instruction::BranchKind::IfNez => BranchKind::IfNez,
    }
}
