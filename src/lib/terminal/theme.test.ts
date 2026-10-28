import { describe, expect, it } from "vitest";
import { getTerminalTheme } from "./theme";

describe("terminal theme", () => {
  it("returns dark theme when dark mode is enabled", () => {
    const theme = getTerminalTheme(true);
    expect(theme.background).toBe("#1d2021");
    expect(theme.foreground).toBe("#ebdbb2");
    expect(theme.cursor).toBe("#fabd2f");
  });

  it("returns light theme when light mode is enabled", () => {
    const theme = getTerminalTheme(false);
    expect(theme.background).toBe("#fbf1c7");
    expect(theme.foreground).toBe("#3c3836");
    expect(theme.cursor).toBe("#3c3836");
  });
});
