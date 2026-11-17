import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { DEFAULT_SOUND_PATHS, SoundManager } from "./sound.svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

interface MockAudioInstance {
  src: string;
  preload: string;
  currentTime: number;
  load: ReturnType<typeof vi.fn>;
  play: ReturnType<typeof vi.fn>;
}

describe("SoundManager", () => {
  const mockedInvoke = vi.mocked(invoke);
  let createdAudios: MockAudioInstance[] = [];
  let playBehavior: (src: string) => Promise<void>;

  beforeEach(() => {
    createdAudios = [];
    playBehavior = async () => {};
    mockedInvoke.mockReset();

    class MockAudio {
      src: string;
      preload = "";
      currentTime = 5;
      load = vi.fn();
      play = vi.fn(() => playBehavior(this.src));

      constructor(src: string) {
        this.src = src;
        createdAudios.push(this);
      }
    }

    vi.stubGlobal("Audio", MockAudio);
    vi.stubGlobal("AudioContext", undefined);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("uses default sound paths when no custom paths are configured", async () => {
    const manager = new SoundManager();
    await manager.refreshFromSettings({
      sound_server_up: null,
      sound_server_down: "   ",
      sound_notification: "",
    });

    expect(mockedInvoke).not.toHaveBeenCalled();

    await manager.play("server_up");
    await manager.play("server_down");
    await manager.play("notification");

    expect(createdAudios.map((a) => a.src)).toEqual([
      DEFAULT_SOUND_PATHS.server_up,
      DEFAULT_SOUND_PATHS.server_down,
      DEFAULT_SOUND_PATHS.notification,
    ]);
    expect(createdAudios.every((a) => a.currentTime === 0)).toBe(true);
    expect(createdAudios.every((a) => a.play.mock.calls.length === 1)).toBe(
      true,
    );
  });

  it("fetches settings via get_app_settings_cmd when refreshFromSettings is called without arguments", async () => {
    mockedInvoke.mockImplementation(async (cmd) => {
      if (cmd === "get_app_settings_cmd") {
        return {
          sound_server_up: "C:/sounds/up.mp3",
          sound_server_down: null,
          sound_notification: null,
        };
      }
      if (cmd === "load_custom_sound_cmd") {
        return "data:audio/mpeg;base64,SUQz";
      }
      return null;
    });

    const manager = new SoundManager();
    await manager.refreshFromSettings();

    expect(mockedInvoke).toHaveBeenCalledWith("get_app_settings_cmd");
    expect(mockedInvoke).toHaveBeenCalledWith("load_custom_sound_cmd", {
      path: "C:/sounds/up.mp3",
    });

    await manager.play("server_up");
    const customUpAudio = createdAudios.find(
      (a) => a.src === "data:audio/mpeg;base64,SUQz",
    );
    expect(customUpAudio).toBeDefined();
    expect(customUpAudio?.play).toHaveBeenCalledTimes(1);
  });

  it("falls back to default sound path when load_custom_sound_cmd returns null", async () => {
    mockedInvoke.mockResolvedValue(null);

    const manager = new SoundManager();
    await manager.refreshFromSettings({
      sound_notification: "C:/missing/notify.mp3",
    });

    expect(mockedInvoke).toHaveBeenCalledWith("load_custom_sound_cmd", {
      path: "C:/missing/notify.mp3",
    });

    await manager.play("notification");
    const notificationAudio = createdAudios.find(
      (a) => a.src === DEFAULT_SOUND_PATHS.notification,
    );
    expect(notificationAudio).toBeDefined();
    expect(notificationAudio?.play).toHaveBeenCalledTimes(1);
  });

  it("falls back to default sound path when custom audio playback fails", async () => {
    const customDataUrl = "data:audio/mpeg;base64,CORRUPT";
    mockedInvoke.mockResolvedValue(customDataUrl);

    playBehavior = async (src) => {
      if (src === customDataUrl) {
        throw new Error("NotSupportedError");
      }
    };

    const manager = new SoundManager();
    await manager.refreshFromSettings({
      sound_server_down: "C:/sounds/corrupt.mp3",
    });

    await manager.play("server_down");

    const customAudio = createdAudios.find((a) => a.src === customDataUrl);
    const fallbackAudio = createdAudios.find(
      (a) => a.src === DEFAULT_SOUND_PATHS.server_down,
    );
    expect(customAudio?.play).toHaveBeenCalledTimes(1);
    expect(fallbackAudio?.play).toHaveBeenCalledTimes(1);
  });

  it("pre-decodes and plays sounds via Web Audio API AudioContext when available", async () => {
    const startedBuffers: AudioBuffer[] = [];
    const resumeMock = vi.fn(async () => {});
    const decodeMock = vi.fn(async (buf: ArrayBuffer) => ({
      byteLength: buf.byteLength,
    } as unknown as AudioBuffer));

    class MockAudioContext {
      state = "suspended";
      destination = {};
      resume = resumeMock;
      decodeAudioData = decodeMock;
      createBufferSource() {
        return {
          buffer: null as AudioBuffer | null,
          connect: vi.fn(),
          start: vi.fn(function (this: { buffer: AudioBuffer | null }) {
            if (this.buffer) startedBuffers.push(this.buffer);
          }),
        };
      }
    }

    vi.stubGlobal("AudioContext", MockAudioContext);
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => ({
        ok: true,
        arrayBuffer: async () => new Uint8Array([1, 2, 3, 4]).buffer,
      })),
    );

    mockedInvoke.mockResolvedValue("data:audio/mpeg;base64,SUQz");

    const manager = new SoundManager();
    await manager.refreshFromSettings({
      sound_notification: "C:/sounds/notify.mp3",
    });
    expect(decodeMock).toHaveBeenCalled();

    await manager.play("notification");
    expect(resumeMock).toHaveBeenCalledTimes(1);
    expect(startedBuffers).toHaveLength(1);
  });

  it("falls back to default buffer on preload when custom Web Audio decode fails and propagates default failures", async () => {
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
    const decodeMock = vi.fn(async (buf: ArrayBuffer) => {
      if (buf.byteLength === 3) {
        throw new Error("corrupt custom audio");
      }
      return { byteLength: buf.byteLength } as unknown as AudioBuffer;
    });

    class MockAudioContext {
      state = "running";
      destination = {};
      resume = vi.fn(async () => {});
      decodeAudioData = decodeMock;
      createBufferSource() {
        return {
          buffer: null as AudioBuffer | null,
          connect: vi.fn(),
          start: vi.fn(),
        };
      }
    }

    vi.stubGlobal("AudioContext", MockAudioContext);
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => ({
        ok: true,
        arrayBuffer: async () => new Uint8Array([1, 2, 3, 4]).buffer,
      })),
    );

    mockedInvoke.mockResolvedValue("data:audio/mpeg;base64,SUQz");

    const manager = new SoundManager();
    await manager.refreshFromSettings({
      sound_notification: "C:/sounds/broken.mp3",
    });
    expect(warnSpy).toHaveBeenCalledTimes(1);
    expect(decodeMock).toHaveBeenCalledTimes(4);
    warnSpy.mockRestore();

    decodeMock.mockRejectedValue(new Error("broken default audio"));
    const failingManager = new SoundManager();
    await expect(failingManager.refreshFromSettings({})).rejects.toThrow(
      "broken default audio",
    );
  });

  it("propagates playback error when default sound playback fails", async () => {
    playBehavior = async () => {
      throw new Error("AudioContext blocked");
    };

    const manager = new SoundManager();
    await expect(manager.play("server_up")).rejects.toThrow(
      "AudioContext blocked",
    );
  });

  it("is a no-op when Audio and AudioContext are undefined", async () => {
    vi.stubGlobal("Audio", undefined);
    vi.stubGlobal("AudioContext", undefined);
    const manager = new SoundManager();
    await expect(manager.play("notification")).resolves.toBeUndefined();
  });
});
