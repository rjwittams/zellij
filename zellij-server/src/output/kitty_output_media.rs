use crate::{output::KittyImageData, ClientId};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    io::{self, Write},
    path::PathBuf,
};
use zellij_utils::consts::ZELLIJ_SOCK_DIR;

const RECENTLY_REFERENCED_RENDER_GENERATIONS: u64 = 240;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KittyOutputMediaRetention {
    KeepRecentlyReferenced,
    OnlyExplicitlyKept,
}

#[derive(Clone, Debug)]
struct CachedKittyOutputFile {
    path: PathBuf,
    last_referenced_render_generation: u64,
}

#[derive(Clone, Debug, Default)]
pub struct KittyOutputMediaCache {
    media_dir: Option<PathBuf>,
    render_generation: u64,
    files: HashMap<(u32, u64), CachedKittyOutputFile>,
    pending_regular_file_reads: HashMap<(ClientId, u32), VecDeque<u64>>,
}

impl KittyOutputMediaCache {
    pub fn disabled() -> Self {
        Self {
            media_dir: None,
            render_generation: 0,
            files: HashMap::new(),
            pending_regular_file_reads: HashMap::new(),
        }
    }

    pub fn for_session(session_name: &str) -> Self {
        Self::new(session_image_media_dir(session_name))
    }

    pub fn new(media_dir: PathBuf) -> Self {
        Self {
            media_dir: Some(media_dir),
            render_generation: 0,
            files: HashMap::new(),
            pending_regular_file_reads: HashMap::new(),
        }
    }

    pub fn ensure_regular_file(
        &mut self,
        image_id: u32,
        generation: u64,
        image_data: &KittyImageData,
    ) -> io::Result<PathBuf> {
        if let Some(cached_file) = self.files.get_mut(&(image_id, generation)) {
            if cached_file.path.exists() {
                cached_file.last_referenced_render_generation = self.render_generation;
                return Ok(cached_file.path.clone());
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
        self.files.insert(
            (image_id, generation),
            CachedKittyOutputFile {
                path: path.clone(),
                last_referenced_render_generation: self.render_generation,
            },
        );
        Ok(path)
    }

    pub fn retain_files<F>(&mut self, retention: KittyOutputMediaRetention, mut keep: F)
    where
        F: FnMut(u32, u64) -> bool,
    {
        let recently_referenced_cutoff = self
            .render_generation
            .saturating_sub(RECENTLY_REFERENCED_RENDER_GENERATIONS);
        let pending_regular_file_reads: HashSet<(u32, u64)> = self
            .pending_regular_file_reads
            .iter()
            .flat_map(|((_, image_id), generations)| {
                generations
                    .iter()
                    .map(move |generation| (*image_id, *generation))
            })
            .collect();
        self.files.retain(|(image_id, generation), cached_file| {
            let was_recently_referenced = retention
                == KittyOutputMediaRetention::KeepRecentlyReferenced
                && cached_file.last_referenced_render_generation >= recently_referenced_cutoff;
            let has_pending_read = pending_regular_file_reads.contains(&(*image_id, *generation));
            let should_keep =
                keep(*image_id, *generation) || was_recently_referenced || has_pending_read;
            if !should_keep {
                let _ = fs::remove_file(&cached_file.path);
            }
            should_keep
        });
    }

    pub fn mark_pending_regular_file_read(
        &mut self,
        client_id: ClientId,
        image_id: u32,
        generation: u64,
    ) {
        self.pending_regular_file_reads
            .entry((client_id, image_id))
            .or_default()
            .push_back(generation);
    }

    pub fn acknowledge_regular_file_read(&mut self, client_id: ClientId, image_id: u32) {
        match self.pending_regular_file_reads.entry((client_id, image_id)) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().pop_front();
                if entry.get().is_empty() {
                    entry.remove();
                }
            },
            std::collections::hash_map::Entry::Vacant(_) => {},
        }
    }

    pub fn remove_client(&mut self, client_id: ClientId) {
        self.pending_regular_file_reads
            .retain(|(pending_client_id, _), _| *pending_client_id != client_id);
    }

    pub fn advance_render_generation(&mut self) {
        self.render_generation = self.render_generation.saturating_add(1);
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
