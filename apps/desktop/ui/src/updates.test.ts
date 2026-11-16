import { beforeEach, describe, expect, it, vi } from "vitest";

const { check, relaunch } = vi.hoisted(() => ({
  check: vi.fn(),
  relaunch: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-updater", () => ({ check }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch }));

import { checkForUpdates } from "./updates";

describe("checkForUpdates", () => {
  beforeEach(() => {
    check.mockReset();
    relaunch.mockReset();
  });

  it("reports when no newer release exists", async () => {
    check.mockResolvedValue(null);
    await expect(checkForUpdates()).resolves.toBe("FilmLogger is up to date.");
    expect(relaunch).not.toHaveBeenCalled();
  });

  it("downloads, installs, and relaunches for a newer release", async () => {
    const downloadAndInstall = vi.fn().mockResolvedValue(undefined);
    check.mockResolvedValue({ version: "0.2.0", downloadAndInstall });
    await expect(checkForUpdates()).resolves.toContain("0.2.0");
    expect(downloadAndInstall).toHaveBeenCalledOnce();
    expect(relaunch).toHaveBeenCalledOnce();
  });
});
