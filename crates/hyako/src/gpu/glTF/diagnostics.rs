use hyakou_core::types::import_diagnostic::{
    ImportDiagnostic, ImportMeshContext, ImportNodeContext,
};

pub(super) fn collect_document_diagnostics(
    gltf: &gltf::Gltf,
    asset_label: &str,
) -> Vec<ImportDiagnostic> {
    let mut diagnostics = Vec::new();

    for animation in gltf.animations() {
        let animation_name = animation.name().unwrap_or("unnamed");
        diagnostics.push(ImportDiagnostic::warning(
            "animation",
            format!(
                "This glTF contains animation data {} `{animation_name}` for asset `{asset_label}`. Hyakou currently imports static meshes, but does not import glTF animations.",
                animation.index()
            ),
            None,
            None,
        ));
    }

    for extension in gltf.extensions_required() {
        diagnostics.push(ImportDiagnostic::warning(
            "required extension",
            format!(
                "This glTF requires extension `{extension}` for asset `{asset_label}`. Hyakou currently imports supported base glTF data, but does not import this extension's behavior."
            ),
            None,
            None,
        ));
    }

    diagnostics
}

pub(super) fn collect_node_diagnostics(
    gltf_node: &gltf::Node<'_>,
    diagnostics: &mut Vec<ImportDiagnostic>,
    asset_label: &str,
) {
    let node_context =
        ImportNodeContext::new(gltf_node.index(), gltf_node.name().map(str::to_owned));

    if gltf_node.camera().is_some() {
        diagnostics.push(unimported_node_feature(
            asset_label,
            "camera",
            "camera data",
            "the node transform",
            "glTF cameras",
            &node_context,
            None,
        ));
    }

    if gltf_node.skin().is_some() {
        diagnostics.push(unimported_node_feature(
            asset_label,
            "skin",
            "skin data",
            "the node transform and mesh",
            "skeletal skinning",
            &node_context,
            None,
        ));
    }

    if gltf_node.weights().is_some() {
        diagnostics.push(unimported_node_feature(
            asset_label,
            "node morph target weights",
            "morph target weights",
            "the node transform",
            "morph target weights",
            &node_context,
            None,
        ));
    }

    if let Some(mesh) = gltf_node.mesh() {
        let mesh_context = ImportMeshContext::new(mesh.index(), mesh.name().map(str::to_owned));

        if mesh.weights().is_some() {
            diagnostics.push(unimported_node_feature(
                asset_label,
                "mesh morph target weights",
                "morph target weights",
                "the base mesh",
                "morph target weights",
                &node_context,
                Some(mesh_context.clone()),
            ));
        }

        for primitive in mesh.primitives() {
            if primitive.morph_targets().next().is_some() {
                diagnostics.push(unimported_node_feature(
                    asset_label,
                    "primitive morph targets",
                    "morph target data",
                    "the base mesh",
                    "morph targets",
                    &node_context,
                    Some(mesh_context.clone()),
                ));
            }
        }
    }
}

fn unimported_node_feature(
    asset_label: &str,
    feature: &str,
    contained_data: &str,
    imported_data: &str,
    unimported_data: &str,
    node_context: &ImportNodeContext,
    mesh_context: Option<ImportMeshContext>,
) -> ImportDiagnostic {
    let context = format_node_mesh_context(node_context, mesh_context.as_ref());

    ImportDiagnostic::warning(
        feature,
        format!(
            "This glTF contains {contained_data} for asset `{asset_label}`{context}. Hyakou currently imports {imported_data}, but does not import {unimported_data}."
        ),
        Some(node_context.clone()),
        mesh_context,
    )
}

fn format_node_mesh_context(
    node_context: &ImportNodeContext,
    mesh_context: Option<&ImportMeshContext>,
) -> String {
    let mut context = format!(
        ", node {}{}",
        node_context.index,
        optional_name(node_context.name.as_deref())
    );

    if let Some(mesh_context) = mesh_context {
        context.push_str(&format!(
            ", mesh {}{}",
            mesh_context.index,
            optional_name(mesh_context.name.as_deref())
        ));
    }

    context
}

fn optional_name(name: Option<&str>) -> String {
    name.map(|name| format!(" `{name}`")).unwrap_or_default()
}
