use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::OnceLock;

use windows::core::{HSTRING, IInspectable, Ref};
use windows::Foundation::{TypedEventHandler, Uri};
use windows::Media::Core::MediaSource;
use windows::Media::Playback::{MediaPlayer, MediaPlayerAudioCategory, MediaPlayerFailedEventArgs};
use windows::Win32::Foundation::{RPC_E_CHANGED_MODE, S_FALSE, S_OK};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

const SOUND_CHANNEL_CAPACITY: usize = 32;
const MAX_ACTIVE_PLAYERS: usize = 8;

enum SoundCommand {
    Play(PathBuf),
    Finished(u64),
}

struct ComApartmentGuard {
    should_uninitialize: bool,
}

impl ComApartmentGuard {
    fn init_mta() -> Result<Self, String> {
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr == S_OK || hr == S_FALSE {
            Ok(Self {
                should_uninitialize: true,
            })
        } else if hr == RPC_E_CHANGED_MODE {
            Ok(Self {
                should_uninitialize: false,
            })
        } else if hr.is_err() {
            Err(format!("CoInitializeEx failed in sound worker: {hr:?}"))
        } else {
            Ok(Self {
                should_uninitialize: true,
            })
        }
    }
}

impl Drop for ComApartmentGuard {
    fn drop(&mut self) {
        if self.should_uninitialize {
            unsafe { CoUninitialize() };
        }
    }
}

static SOUND_WORKER_TX: OnceLock<Option<SyncSender<SoundCommand>>> = OnceLock::new();

pub(crate) fn play_file(path: PathBuf) -> Result<(), String> {
    let sender = SOUND_WORKER_TX
        .get_or_init(spawn_sound_worker)
        .as_ref()
        .ok_or_else(|| "Native Windows audio worker failed to initialize".to_string())?;

    sender
        .try_send(SoundCommand::Play(path))
        .map_err(|err| format!("Failed to queue sound playback on Windows worker: {err}"))
}

fn spawn_sound_worker() -> Option<SyncSender<SoundCommand>> {
    let (tx, rx) = mpsc::sync_channel::<SoundCommand>(SOUND_CHANNEL_CAPACITY);
    let worker_tx = tx.clone();

    match std::thread::Builder::new()
        .name("caffeine-sound".to_string())
        .spawn(move || run_sound_worker(rx, worker_tx))
    {
        Ok(_) => Some(tx),
        Err(err) => {
            eprintln!("Failed to spawn native Windows sound thread: {err}");
            None
        }
    }
}

fn run_sound_worker(rx: Receiver<SoundCommand>, tx: SyncSender<SoundCommand>) {
    // Declare the COM guard before player objects so Rust drops all COM wrappers prior to CoUninitialize.
    let _com_guard = match ComApartmentGuard::init_mta() {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    let mut active_players: HashMap<u64, MediaPlayer> = HashMap::new();
    let mut next_id: u64 = 1;

    while let Ok(command) = rx.recv() {
        match command {
            SoundCommand::Play(path) => {
                if let Err(err) = play_on_winrt(&mut active_players, &mut next_id, &tx, &path) {
                    eprintln!("WinRT audio playback error for '{}': {err}", path.display());
                }
            }
            SoundCommand::Finished(id) => {
                if let Some(player) = active_players.remove(&id) {
                    let _ = player.Close();
                }
            }
        }
    }

    for (_, player) in active_players.drain() {
        let _ = player.Close();
    }
}

fn play_on_winrt(
    active_players: &mut HashMap<u64, MediaPlayer>,
    next_id: &mut u64,
    tx: &SyncSender<SoundCommand>,
    path: &Path,
) -> Result<(), String> {
    if active_players.len() >= MAX_ACTIVE_PLAYERS {
        if let Some(oldest_id) = active_players.keys().min().copied() {
            if let Some(oldest_player) = active_players.remove(&oldest_id) {
                let _ = oldest_player.Close();
            }
        }
    }

    let id = *next_id;
    *next_id = next_id.wrapping_add(1).max(1);

    let file_uri = path_to_file_uri(path)?;
    let uri = Uri::CreateUri(&HSTRING::from(file_uri))
        .map_err(|e| format!("Uri::CreateUri failed: {e}"))?;
    let source = MediaSource::CreateFromUri(&uri)
        .map_err(|e| format!("MediaSource::CreateFromUri failed: {e}"))?;
    let player = MediaPlayer::new().map_err(|e| format!("MediaPlayer::new failed: {e}"))?;

    if let Err(err) = player.SetAudioCategory(MediaPlayerAudioCategory::Alerts) {
        eprintln!("Failed to set MediaPlayer audio category to Alerts: {err}");
    }

    // Prevent System Media Transport Controls overlays from appearing for short alert chimes.
    match player.CommandManager() {
        Ok(command_manager) => {
            if let Err(err) = command_manager.SetIsEnabled(false) {
                eprintln!("Failed to disable MediaPlayer SMTC controls: {err}");
            }
        }
        Err(err) => {
            eprintln!("Failed to access MediaPlayer CommandManager: {err}");
        }
    }

    let tx_ended = tx.clone();
    player
        .MediaEnded(&TypedEventHandler::<MediaPlayer, IInspectable>::new(
            move |_sender: Ref<'_, MediaPlayer>, _args: Ref<'_, IInspectable>| {
                let _ = tx_ended.try_send(SoundCommand::Finished(id));
                Ok(())
            },
        ))
        .map_err(|e| format!("Failed to attach MediaEnded handler: {e}"))?;

    let tx_failed = tx.clone();
    player
        .MediaFailed(&TypedEventHandler::<MediaPlayer, MediaPlayerFailedEventArgs>::new(
            move |_sender: Ref<'_, MediaPlayer>, args: Ref<'_, MediaPlayerFailedEventArgs>| {
                if let Some(info) = args.as_ref() {
                    let error_kind = info.Error().ok();
                    let ext_code = info.ExtendedErrorCode().ok();
                    let message = info.ErrorMessage().ok().map(|s| s.to_string_lossy());
                    eprintln!(
                        "WinRT MediaPlayer failed (kind={error_kind:?}, hresult={ext_code:?}, message={message:?})"
                    );
                }
                let _ = tx_failed.try_send(SoundCommand::Finished(id));
                Ok(())
            },
        ))
        .map_err(|e| format!("Failed to attach MediaFailed handler: {e}"))?;

    if let Err(err) = player.SetSource(&source) {
        let _ = player.Close();
        return Err(format!("MediaPlayer::SetSource failed: {err}"));
    }

    if let Err(err) = player.Play() {
        let _ = player.Close();
        return Err(format!("MediaPlayer::Play failed: {err}"));
    }

    active_players.insert(id, player);
    Ok(())
}

pub(crate) fn path_to_file_uri(path: &Path) -> Result<String, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to resolve current directory for audio path: {e}"))?
            .join(path)
    };

    let normalized = strip_windows_verbatim_prefix(&absolute);
    let url = url::Url::from_file_path(&normalized).map_err(|_| {
        format!(
            "Failed to convert sound path '{}' to a file:// URI",
            normalized.display()
        )
    })?;
    Ok(url.into())
}

fn strip_windows_verbatim_prefix(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    // WinRT Uri::CreateUri rejects \\?\ verbatim paths, so normalize local and UNC paths first.
    if let Some(unc_tail) = raw.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{unc_tail}"))
    } else if let Some(local_tail) = raw.strip_prefix(r"\\?\") {
        PathBuf::from(local_tail)
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_file_uri_handles_spaces_and_verbatim_prefixes() {
        let spaced = path_to_file_uri(Path::new(r"C:\Users\Test User\my sounds\ding #1.mp3"))
            .expect("valid standard path");
        assert_eq!(
            spaced,
            "file:///C:/Users/Test%20User/my%20sounds/ding%20%231.mp3"
        );

        let verbatim_local = path_to_file_uri(Path::new(r"\\?\C:\Temp\alert sound.mp3"))
            .expect("valid verbatim local path");
        assert_eq!(verbatim_local, "file:///C:/Temp/alert%20sound.mp3");

        let verbatim_unc = path_to_file_uri(Path::new(r"\\?\UNC\fileserver\share\beep.mp3"))
            .expect("valid verbatim UNC path");
        assert_eq!(verbatim_unc, "file://fileserver/share/beep.mp3");
    }
}
