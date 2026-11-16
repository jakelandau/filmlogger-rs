import { describe, expect, it } from "vitest";
import { formatFlags } from "./format-flags";
import type { Format, ScreenClass } from "./types";

function theatre(screen: ScreenClass, flags = { three_d: false, hfr: false, dbox: false }): Format {
  return {
    theatre: {
      projection: { dci: "4k_laser" },
      screen,
      ...flags,
    },
  };
}

const labels = (format: Format | null) => formatFlags(format).map((flag) => flag.label);

describe("formatFlags", () => {
  it("reports explicit theatre presentation flags", () => {
    expect(labels(theatre("plf", { three_d: true, hfr: true, dbox: false }))).toEqual(["3D", "HFR"]);
  });

  it("treats D-BOX as motion seating", () => {
    expect(labels(theatre("regular", { three_d: false, hfr: false, dbox: true }))).toEqual(["Motion"]);
  });

  it("treats 4DX and Ultra 4DX as motion seating and 3D", () => {
    expect(labels(theatre("4dx"))).toEqual(["3D", "Motion"]);
    expect(labels(theatre("ultra_4dx"))).toEqual(["3D", "Motion"]);
  });

  it("does not report flags for home viewings", () => {
    expect(labels({ home: { medium: "bluray" } })).toEqual([]);
  });
});
