import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { DEFAULT_SOUND_PATHS, SoundManager } from "./sound.svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

interface MockAudioInstance {
  src: string;
  currentTime: number;
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
      currentTime = 5;
      play = vi.fn(() => playBehavior(this.src));

      constructor(src: string) {
        this.src = src;
        createdAudios.push(this);
      }
    }

    vi.stubGlobal("Audio", MockAudio);
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
    expect(createdAudios[0].src).toBe("data:audio/mpeg;base64,SUQz");
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
    expect(createdAudios[0].src).toBe(DEFAULT_SOUND_PATHS.notification);
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

    expect(createdAudios.map((a) => a.src)).toEqual([
      customDataUrl,
      DEFAULT_SOUND_PATHS.server_down,
    ]);
    expect(createdAudios[1].play).toHaveBeenCalledTimes(1);
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

  it("is a no-op when Audio is undefined", async () => {
    vi.stubGlobal("Audio", undefined);
    const manager = new SoundManager();
    await expect(manager.play("notification")).resolves.toBeUndefined();
  });
});
