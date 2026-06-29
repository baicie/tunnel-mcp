import { describe, expect, it } from "vitest";
import { maskSecret } from "./mask";

describe("maskSecret", () => {
  it("returns undefined for empty secret", () => {
    expect(maskSecret()).toBeUndefined();
    expect(maskSecret("")).toBeUndefined();
  });

  it("masks short secret", () => {
    expect(maskSecret("abc")).toBe("••••");
  });

  it("keeps prefix and suffix for long secret", () => {
    expect(maskSecret("sk-1234567890abcd")).toBe("sk-1••••abcd");
  });
});
