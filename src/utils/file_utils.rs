use std::path::Path;

/// File extensions recognised as 3D model formats.
pub const SUPPORTED_3D_EXTENSIONS: &[&str] = &[
    "glb", "gltf", "fbx", "obj", "stl", "ply", "dae", "3ds",
];

/// Check whether a filename has a recognised 3D model extension.
pub fn is_3d_file(name: &str) -> bool {
    name.rsplit('.')
        .next()
        .map(|ext| SUPPORTED_3D_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Check whether the file at `path` has a supported 3D extension.
pub fn is_supported_format(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_3D_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_3d_file() {
        assert!(is_3d_file("model.glb"));
        assert!(is_3d_file("hero.FBX"));
        assert!(!is_3d_file("readme.md"));
        assert!(!is_3d_file("no_extension"));
    }

    #[test]
    fn test_is_supported_format() {
        assert!(is_supported_format(&PathBuf::from("/tmp/scene.gltf")));
        assert!(!is_supported_format(&PathBuf::from("/tmp/image.png")));
    }
}
