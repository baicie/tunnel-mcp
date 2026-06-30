import { describe, expect, it } from "vitest";
import type { SaveWorkspaceProfileInput, WorkspaceProfile } from "../../src/lib/workspace/types";

describe("workspace types", () => {
  it("can construct a SaveWorkspaceProfileInput and serialize via JSON shape", () => {
    const input: SaveWorkspaceProfileInput = {
      id: undefined,
      name: "demo",
      rootPath: "/tmp/demo",
      permissionScopes: [
        {
          id: "scope-1",
          kind: "filesystem",
          pattern: "/tmp/demo/**",
          access: "readwrite",
          requireApproval: true,
        },
      ],
    };

    const profile: WorkspaceProfile = {
      ...input,
      id: "wp-1",
      createdAt: 0,
      updatedAt: 0,
    };

    const raw = JSON.parse(JSON.stringify(profile)) as Record<string, unknown>;
    expect(raw.id).toBe("wp-1");
    expect(raw.name).toBe("demo");
    expect(raw.rootPath).toBe("/tmp/demo");
    expect(Array.isArray(raw.permissionScopes)).toBe(true);
    expect((raw.permissionScopes as unknown[]).length).toBe(1);
  });
});
