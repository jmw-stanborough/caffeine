import { invoke } from "@tauri-apps/api/core";

export const DEFAULT_SOUND_PATHS = {
  server_up: "/sound-files/servers/up.mp3",
  server_down: "/sound-files/servers/down.mp3",
  notification: "/sound-files/notification.mp3",
} as const;

export type SoundEffectKey = keyof typeof DEFAULT_SOUND_PATHS;

export interface SoundSettings {
  sound_server_up?: string | null;
  sound_server_down?: string | null;
  sound_notification?: string | null;
}

const SETTINGS_KEY_MAP: Record<SoundEffectKey, keyof SoundSettings> = {
  server_up: "sound_server_up",
  server_down: "sound_server_down",
  notification: "sound_notification",
};

export class SoundManager {
  private sources: Record<SoundEffectKey, string> = { ...DEFAULT_SOUND_PATHS };
  private audioCache: Partial<
    Record<SoundEffectKey, { src: string; audio: HTMLAudioElement }>
  > = {};

  async refreshFromSettings(settings?: SoundSettings): Promise<void> {
    const resolvedSettings =
      settings ?? (await invoke<SoundSettings>("get_app_settings_cmd"));

    const keys = Object.keys(DEFAULT_SOUND_PATHS) as SoundEffectKey[];
    await Promise.all(
      keys.map(async (key) => {
        const rawPath = resolvedSettings[SETTINGS_KEY_MAP[key]];
        const trimmed = rawPath?.trim() ?? "";
        if (!trimmed) {
          this.sources[key] = DEFAULT_SOUND_PATHS[key];
          return;
        }

        const dataUrl = await invoke<string | null>("load_custom_sound_cmd", {
          path: trimmed,
        });
        this.sources[key] =
          dataUrl && dataUrl.length > 0 ? dataUrl : DEFAULT_SOUND_PATHS[key];
      }),
    );
  }

  async play(key: SoundEffectKey): Promise<void> {
    if (typeof Audio === "undefined") return;

    const src = this.sources[key];
    const defaultSrc = DEFAULT_SOUND_PATHS[key];
    const audio = this.getAudio(key, src);

    try {
      audio.currentTime = 0;
      await audio.play();
    } catch (err) {
      if (src === defaultSrc) {
        throw err;
      }
      // Fall back to the built-in sound if a custom audio source fails to decode or play.
      this.sources[key] = defaultSrc;
      const fallbackAudio = this.getAudio(key, defaultSrc);
      fallbackAudio.currentTime = 0;
      await fallbackAudio.play();
    }
  }

  private getAudio(key: SoundEffectKey, src: string): HTMLAudioElement {
    const cached = this.audioCache[key];
    if (cached && cached.src === src) {
      return cached.audio;
    }

    const audio = new Audio(src);
    this.audioCache[key] = { src, audio };
    return audio;
  }
}

export const soundManager = new SoundManager();
