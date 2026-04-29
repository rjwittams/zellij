use crate::output::KittyImageData;
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::PathBuf,
};
use zellij_utils::consts::ZELLIJ_SOCK_DIR;

#[derive(Clone, Debug, Default)]
pub struct KittyOutputMediaCache {
    media_dir: Option<PathBuf>,
    files: HashMap<(u32, u64), PathBuf>,
}

impl KittyOutputMediaCache {
    pub fn disabled() -> Self {
        Self {
            media_dir: None,
            files: HashMap::new(),
        }
    }

    pub fn for_session(session_name: &str) -> Self {
        Self::new(session_image_media_dir(session_name))
    }

    pub fn new(media_dir: PathBuf) -> Self {
        Self {
            media_dir: Some(media_dir),
            files: HashMap::new(),
        }
    }

    pub fn ensure_regular_file(
        &mut self,
        image_id: u32,
        generation: u64,
        image_data: &KittyImageData,
    ) -> io::Result<PathBuf> {
        if let Some(path) = self.files.get(&(image_id, generation)) {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        let media_dir = self.media_dir.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "kitty output media disabled")
        })?;
        fs::create_dir_all(media_dir)?;
        set_private_media_permissions(media_dir);

        let path = media_dir.join(format!("i{}-g{}.kitty-image", image_id, generation));
        let bytes = kitty_image_bytes(image_data);
        let write_result = (|| -> io::Result<()> {
            let mut file = fs::File::create(&path)?;
            file.write_all(bytes)?;
            Ok(())
        })();
        if let Err(error) = write_result {
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        self.files.insert((image_id, generation), path.clone());
        Ok(path)
    }

    pub fn rename_session(&mut self, old_session_name: &str, new_session_name: &str) {
        if self.media_dir.is_none() {
            return;
        }

        let old_path = session_media_dir(old_session_name);
        let new_path = session_media_dir(new_session_name);
        if old_path.exists() {
            let _ = fs::rename(old_path, new_path);
        }
        self.media_dir = Some(session_image_media_dir(new_session_name));
        self.files.clear();
    }

    pub fn cleanup_session_media(session_name: &str) {
        let path = session_media_dir(session_name);
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }
}

fn session_media_dir(session_name: &str) -> PathBuf {
    ZELLIJ_SOCK_DIR.join("session-media").join(session_name)
}

fn session_image_media_dir(session_name: &str) -> PathBuf {
    session_media_dir(session_name).join("image")
}

fn kitty_image_bytes(image_data: &KittyImageData) -> &[u8] {
    match image_data {
        KittyImageData::Png { data, .. }
        | KittyImageData::Rgb { data, .. }
        | KittyImageData::Rgba { data, .. } => data,
    }
}

fn set_private_media_permissions(media_dir: &std::path::Path) {
    if !media_dir.starts_with(&*ZELLIJ_SOCK_DIR) {
        set_private_permissions(media_dir);
        return;
    }

    let mut ancestors = Vec::new();
    let mut current = Some(media_dir);
    while let Some(path) = current {
        if path == &*ZELLIJ_SOCK_DIR {
            break;
        }
        ancestors.push(path);
        current = path.parent();
    }
    for path in ancestors.into_iter().rev() {
        set_private_permissions(path);
    }
}

#[cfg(unix)]
fn set_private_permissions(path: &std::path::Path) {
    let _ = zellij_utils::shared::set_permissions(path, 0o700);
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &std::path::Path) {}
