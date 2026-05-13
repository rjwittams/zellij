use crate::{output::KittyImageData, panes::kitty_asset_store::KittyAssetData, ClientId};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    io::{self, Write},
    path::PathBuf,
};
#[cfg(unix)]
use std::{ffi::CString, ptr};
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

#[derive(Clone, Debug)]
struct PendingRegularFileRead {
    image_id: u32,
    generation: u64,
    requested_acknowledgement: bool,
}

#[derive(Clone, Debug)]
struct TrackedTemporaryOutputFile {
    path: PathBuf,
    last_referenced_render_generation: u64,
}

#[derive(Clone, Debug)]
struct TrackedSharedMemoryOutput {
    name: String,
    last_referenced_render_generation: u64,
}

#[derive(Clone, Debug, Default)]
pub struct KittyOutputMediaCache {
    media_dir: Option<PathBuf>,
    render_generation: u64,
    one_shot_media_sequence: u64,
    files: HashMap<(u32, u64), CachedKittyOutputFile>,
    temporary_files: VecDeque<TrackedTemporaryOutputFile>,
    shared_memory_objects: VecDeque<TrackedSharedMemoryOutput>,
    pending_regular_file_reads: HashMap<ClientId, VecDeque<PendingRegularFileRead>>,
}

impl KittyOutputMediaCache {
    pub fn disabled() -> Self {
        Self {
            media_dir: None,
            render_generation: 0,
            one_shot_media_sequence: 0,
            files: HashMap::new(),
            temporary_files: VecDeque::new(),
            shared_memory_objects: VecDeque::new(),
            pending_regular_file_reads: HashMap::new(),
        }
    }

    pub fn new_for_session(session_name: &str) -> Self {
        Self::new(session_image_media_dir(session_name))
    }

    pub fn new(media_dir: PathBuf) -> Self {
        Self {
            media_dir: Some(media_dir),
            render_generation: 0,
            one_shot_media_sequence: 0,
            files: HashMap::new(),
            temporary_files: VecDeque::new(),
            shared_memory_objects: VecDeque::new(),
            pending_regular_file_reads: HashMap::new(),
        }
    }

    pub fn ensure_regular_file(
        &mut self,
        image_id: u32,
        generation: u64,
        image_data: &KittyImageData,
    ) -> io::Result<PathBuf> {
        let mut asset_data = KittyAssetData::Image(image_data.clone());
        self.ensure_regular_file_for_asset(image_id, generation, &mut asset_data)
    }

    pub fn ensure_regular_file_for_asset(
        &mut self,
        image_id: u32,
        generation: u64,
        asset_data: &mut KittyAssetData,
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
        let write_result = (|| -> io::Result<()> {
            let mut file = fs::File::create(&path)?;
            write_kitty_asset_data(&mut file, asset_data)?;
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

    pub fn create_temporary_file_for_asset(
        &mut self,
        client_id: ClientId,
        image_id: u32,
        generation: u64,
        asset_data: &mut KittyAssetData,
    ) -> io::Result<PathBuf> {
        let media_dir = self.media_dir.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "kitty output media disabled")
        })?;
        fs::create_dir_all(media_dir)?;
        set_private_media_permissions(media_dir);

        let sequence = self.one_shot_media_sequence;
        self.one_shot_media_sequence = self.one_shot_media_sequence.saturating_add(1);
        let path = media_dir.join(format!(
            "tty-graphics-protocol-c{}-i{}-g{}-r{}-{}.kitty-image",
            client_id, image_id, generation, self.render_generation, sequence
        ));
        let write_result = (|| -> io::Result<()> {
            let mut file = fs::File::create(&path)?;
            write_kitty_asset_data(&mut file, asset_data)?;
            Ok(())
        })();
        if let Err(error) = write_result {
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        self.temporary_files.push_back(TrackedTemporaryOutputFile {
            path: path.clone(),
            last_referenced_render_generation: self.render_generation,
        });
        Ok(path)
    }

    pub fn create_shared_memory_for_asset(
        &mut self,
        _client_id: ClientId,
        _image_id: u32,
        _generation: u64,
        asset_data: &mut KittyAssetData,
    ) -> io::Result<String> {
        let image_data = asset_data.image_data().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid kitty image data")
        })?;
        let sequence = self.one_shot_media_sequence;
        self.one_shot_media_sequence = self.one_shot_media_sequence.saturating_add(1);
        let name = format!("/zk{:x}{:x}", std::process::id(), sequence);
        write_shared_memory_payload(&name, kitty_image_bytes(&image_data))?;
        self.shared_memory_objects
            .push_back(TrackedSharedMemoryOutput {
                name: name.clone(),
                last_referenced_render_generation: self.render_generation,
            });
        Ok(name)
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
            .values()
            .flat_map(|pending_reads| {
                pending_reads
                    .iter()
                    .map(|pending_read| (pending_read.image_id, pending_read.generation))
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
        self.temporary_files.retain(|tracked_file| {
            let was_recently_referenced =
                tracked_file.last_referenced_render_generation >= recently_referenced_cutoff;
            if !was_recently_referenced {
                let _ = fs::remove_file(&tracked_file.path);
            }
            was_recently_referenced && tracked_file.path.exists()
        });
        self.shared_memory_objects.retain(|tracked_object| {
            let was_recently_referenced =
                tracked_object.last_referenced_render_generation >= recently_referenced_cutoff;
            if !was_recently_referenced {
                let _ = unlink_shared_memory_payload(&tracked_object.name);
            }
            was_recently_referenced
        });
    }

    pub fn mark_pending_regular_file_read(
        &mut self,
        client_id: ClientId,
        image_id: u32,
        generation: u64,
        requested_acknowledgement: bool,
    ) {
        self.pending_regular_file_reads
            .entry(client_id)
            .or_default()
            .push_back(PendingRegularFileRead {
                image_id,
                generation,
                requested_acknowledgement,
            });
    }

    pub fn acknowledge_regular_file_read(&mut self, client_id: ClientId, image_id: u32) {
        match self.pending_regular_file_reads.entry(client_id) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let pending_reads = entry.get_mut();
                if let Some(watermark_index) = pending_reads.iter().position(|pending_read| {
                    pending_read.image_id == image_id && pending_read.requested_acknowledgement
                }) {
                    pending_reads.drain(..=watermark_index);
                }
                if entry.get().is_empty() {
                    entry.remove();
                }
            },
            std::collections::hash_map::Entry::Vacant(_) => {},
        }
    }

    pub fn remove_client(&mut self, client_id: ClientId) {
        self.pending_regular_file_reads.remove(&client_id);
    }

    pub fn cleanup_tracked_one_shot_media(&mut self) {
        for tracked_file in self.temporary_files.drain(..) {
            let _ = fs::remove_file(tracked_file.path);
        }
        for tracked_object in self.shared_memory_objects.drain(..) {
            let _ = unlink_shared_memory_payload(&tracked_object.name);
        }
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
        if !old_path.exists() {
            self.media_dir = Some(session_image_media_dir(new_session_name));
            self.files.clear();
            self.temporary_files.clear();
            self.shared_memory_objects.clear();
            self.pending_regular_file_reads.clear();
            return;
        }

        match fs::rename(&old_path, &new_path) {
            Ok(()) => {
                for cached_file in self.files.values_mut() {
                    if let Ok(relative_path) = cached_file.path.strip_prefix(&old_path) {
                        cached_file.path = new_path.join(relative_path);
                    }
                }
                for temporary_file in self.temporary_files.iter_mut() {
                    if let Ok(relative_path) = temporary_file.path.strip_prefix(&old_path) {
                        temporary_file.path = new_path.join(relative_path);
                    }
                }
                self.pending_regular_file_reads.clear();
            },
            Err(error) => {
                log::warn!(
                    "failed to rename kitty media dir from {:?} to {:?}: {:?}",
                    old_path,
                    new_path,
                    error
                );
            },
        }
        self.media_dir = Some(session_image_media_dir(new_session_name));
    }

    pub fn cleanup_session_media(session_name: &str) {
        let path = session_media_dir(session_name);
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }
}

#[cfg(unix)]
fn write_shared_memory_payload(name: &str, payload: &[u8]) -> io::Result<()> {
    let c_name = CString::new(name)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid shared memory name"))?;
    let fd = unsafe {
        libc::shm_open(
            c_name.as_ptr(),
            libc::O_CREAT | libc::O_EXCL | libc::O_RDWR,
            0o600,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }

    let write_result = write_shared_memory_payload_to_fd(fd, payload);
    unsafe {
        libc::close(fd);
    }
    if write_result.is_err() {
        let _ = unlink_shared_memory_payload(name);
    }
    write_result
}

#[cfg(unix)]
fn write_shared_memory_payload_to_fd(fd: libc::c_int, payload: &[u8]) -> io::Result<()> {
    let len = payload.len();
    if unsafe { libc::ftruncate(fd, len as libc::off_t) } != 0 {
        return Err(io::Error::last_os_error());
    }
    if len == 0 {
        return Ok(());
    }
    let mapping = unsafe {
        libc::mmap(
            ptr::null_mut(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        )
    };
    if mapping == libc::MAP_FAILED {
        return Err(io::Error::last_os_error());
    }
    unsafe {
        ptr::copy_nonoverlapping(payload.as_ptr(), mapping as *mut u8, len);
        if libc::munmap(mapping, len) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn write_shared_memory_payload(_name: &str, _payload: &[u8]) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "shared memory kitty output is unsupported on this platform",
    ))
}

#[cfg(unix)]
fn unlink_shared_memory_payload(name: &str) -> io::Result<()> {
    let c_name = CString::new(name)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid shared memory name"))?;
    if unsafe { libc::shm_unlink(c_name.as_ptr()) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(not(unix))]
fn unlink_shared_memory_payload(_name: &str) -> io::Result<()> {
    Ok(())
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

fn write_kitty_asset_data(
    mut writer: impl Write,
    asset_data: &mut KittyAssetData,
) -> io::Result<()> {
    let image_data = asset_data
        .image_data()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid kitty image data"))?;
    writer.write_all(kitty_image_bytes(&image_data))
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
