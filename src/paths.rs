use std::path::{Path, PathBuf};

pub fn project_root() -> PathBuf {
    if let Ok(root) = std::env::var("QUBA_PROJECT_ROOT") {
        return PathBuf::from(root);
    }

    if let Some(shared) = local_share_root() {
        return shared;
    }

    let flatpak_root = PathBuf::from("/app/share/quba-gtk");
    if flatpak_root.join("quba-viewer-1.5.0").exists() {
        return flatpak_root;
    }

    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("quba-viewer-1.5.0").exists() {
            return cwd;
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if parent.join("quba-viewer-1.5.0").exists() {
                return parent.to_path_buf();
            }
            if let Some(grandparent) = parent.parent() {
                if grandparent.join("quba-viewer-1.5.0").exists() {
                    return grandparent.to_path_buf();
                }
            }
        }
    }

    PathBuf::from(".")
}

fn local_share_root() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    let candidate = PathBuf::from(home).join(".local/share/quba-gtk");
    if candidate.join("quba-viewer-1.5.0").exists() && helper_exists_in(&candidate) {
        Some(candidate)
    } else {
        None
    }
}

pub fn helper_script() -> PathBuf {
    let root = project_root();
    let bundled = root.join("dist").join("quba-render-helper.bundle.cjs");
    if bundled.exists() {
        return bundled;
    }
    root.join("scripts").join("quba-render-helper.cjs")
}

pub fn node_binary() -> PathBuf {
    if let Ok(path) = std::env::var("QUBA_NODE_BINARY") {
        return PathBuf::from(path);
    }
    PathBuf::from("node")
}

pub fn legacy_root() -> PathBuf {
    project_root().join("quba-viewer-1.5.0")
}

pub fn has_legacy_assets() -> bool {
    legacy_root().join("app").join("xslt").exists()
}

pub fn display_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

fn helper_exists_in(root: &Path) -> bool {
    root.join("dist").join("quba-render-helper.bundle.cjs").exists()
        || root.join("scripts").join("quba-render-helper.cjs").exists()
}
