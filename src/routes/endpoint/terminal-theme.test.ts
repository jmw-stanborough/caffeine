import { describe, expect, it } from "vitest";
import { getTerminalTheme } from "./terminal-theme";

describe("terminal-theme", () => {
  it("returns gruvbox dark theme when isDark is true", () => {
    const theme = getTerminalTheme(true);
    expect(theme.background).toBe("#1d2021");
    expect(theme.foreground).toBe("#ebdbb2");
    expect(theme.cursor).toBe("#fabd2f");
  });

  it("returns gruvbox light theme when isDark is false", () => {
    const theme = getTerminalTheme(false);
    expect(theme.background).toBe("#fbf1c7");
    expect(theme.foreground).toBe("#3c3836");
    expect(theme.cursor).toBe("#3c3836");
  });
});
