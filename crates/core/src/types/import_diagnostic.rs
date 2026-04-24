#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDiagnostic {
    pub severity: ImportDiagnosticSeverity,
    pub feature: String,
    pub message: String,
    pub node: Option<ImportNodeContext>,
    pub mesh: Option<ImportMeshContext>,
}

impl ImportDiagnostic {
    pub fn warning(
        feature: impl Into<String>,
        message: impl Into<String>,
        node: Option<ImportNodeContext>,
        mesh: Option<ImportMeshContext>,
    ) -> Self {
        Self {
            severity: ImportDiagnosticSeverity::Warning,
            feature: feature.into(),
            message: message.into(),
            node,
            mesh,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportDiagnosticSeverity {
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportNodeContext {
    pub index: usize,
    pub name: Option<String>,
}

impl ImportNodeContext {
    pub fn new(index: usize, name: Option<String>) -> Self {
        Self { index, name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportMeshContext {
    pub index: usize,
    pub name: Option<String>,
}

impl ImportMeshContext {
    pub fn new(index: usize, name: Option<String>) -> Self {
        Self { index, name }
    }
}
