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

function decodeDataUrlToArrayBuffer(dataUrl: string): ArrayBuffer {
  const commaIndex = dataUrl.indexOf(",");
  if (commaIndex < 0) {
    throw new Error("Invalid data URL format");
  }
  const base64 = dataUrl.slice(commaIndex + 1);
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

async function fetchSoundBytes(src: string): Promise<ArrayBuffer> {
  if (src.startsWith("data:")) {
    return decodeDataUrlToArrayBuffer(src);
  }
  const response = await fetch(src);
  if (!response.ok) {
    throw new Error(`Failed to fetch sound asset ${src}: ${response.status}`);
  }
  return response.arrayBuffer();
}

export class SoundManager {
  private sources: Record<SoundEffectKey, string> = { ...DEFAULT_SOUND_PATHS };
  private audioContext: AudioContext | null = null;
  private bufferCache: Partial<
    Record<SoundEffectKey, { src: string; buffer: AudioBuffer }>
  > = {};
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
        const dataUrl = trimmed
          ? await invoke<string | null>("load_custom_sound_cmd", {
              path: trimmed,
            })
          : null;
        this.sources[key] =
          dataUrl && dataUrl.length > 0 ? dataUrl : DEFAULT_SOUND_PATHS[key];
        await this.preload(key, this.sources[key]);
      }),
    );
  }

  async play(key: SoundEffectKey): Promise<void> {
    const src = this.sources[key];
    const defaultSrc = DEFAULT_SOUND_PATHS[key];

    if (typeof AudioContext !== "undefined") {
      try {
        await this.playBuffer(key, src);
        return;
      } catch (err) {
        if (src === defaultSrc) {
          throw err;
        }
        // Fall back to the built-in sound if a custom audio source fails to decode or play.
        this.sources[key] = defaultSrc;
        await this.playBuffer(key, defaultSrc);
        return;
      }
    }

    if (typeof Audio === "undefined") return;

    const audio = this.getAudio(key, src);
    try {
      audio.currentTime = 0;
      await audio.play();
    } catch (err) {
      if (src === defaultSrc) {
        throw err;
      }
      this.sources[key] = defaultSrc;
      const fallbackAudio = this.getAudio(key, defaultSrc);
      fallbackAudio.currentTime = 0;
      await fallbackAudio.play();
    }
  }

  private async preload(key: SoundEffectKey, src: string): Promise<void> {
    if (typeof AudioContext !== "undefined") {
      try {
        await this.getDecodedBuffer(key, src);
      } catch {
        // Defer error handling to play() so fallback to defaultSrc can run cleanly.
      }
      return;
    }

    if (typeof Audio !== "undefined") {
      this.getAudio(key, src);
    }
  }

  private getContext(): AudioContext {
    if (!this.audioContext) {
      this.audioContext = new AudioContext();
    }
    return this.audioContext;
  }

  private async getDecodedBuffer(
    key: SoundEffectKey,
    src: string,
  ): Promise<AudioBuffer> {
    const cached = this.bufferCache[key];
    if (cached && cached.src === src) {
      return cached.buffer;
    }

    const ctx = this.getContext();
    const bytes = await fetchSoundBytes(src);
    const buffer = await ctx.decodeAudioData(bytes.slice(0));
    this.bufferCache[key] = { src, buffer };
    return buffer;
  }

  private async playBuffer(key: SoundEffectKey, src: string): Promise<void> {
    const ctx = this.getContext();
    if (ctx.state === "suspended") {
      await ctx.resume();
    }
    const buffer = await this.getDecodedBuffer(key, src);
    const source = ctx.createBufferSource();
    source.buffer = buffer;
    source.connect(ctx.destination);
    source.start(0);
  }

  private getAudio(key: SoundEffectKey, src: string): HTMLAudioElement {
    const cached = this.audioCache[key];
    if (cached && cached.src === src) {
      return cached.audio;
    }

    const audio = new Audio(src);
    audio.preload = "auto";
    audio.load?.();
    this.audioCache[key] = { src, audio };
    return audio;
  }
}

export const soundManager = new SoundManager();
