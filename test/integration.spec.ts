import { SELF } from "cloudflare:test";
import { describe, expect, it } from "vitest";

describe("Routing", () => {
  it("returns 404 for unknown routes", async () => {
    const response = await SELF.fetch("http://example.com/unknown");
    expect(response.status).toBe(404);
  });

  it("returns 404 when page id segment is missing", async () => {
    const response = await SELF.fetch("http://example.com/api/v1/connect");
    expect(response.status).toBe(404);
  });
});

describe("CORS", () => {
  it("blocks requests from disallowed origins", async () => {
    const response = await SELF.fetch("http://example.com/api/v1/connect/my-post", {
      headers: {
        Origin: "http://evil.com",
        Upgrade: "websocket",
      },
    });

    expect(response.status).toBe(403);
    expect(response.headers.get("Access-Control-Allow-Origin")).toBeNull();
  });

  it("allows requests from configured origins", async () => {
    const response = await SELF.fetch("http://example.com/api/v1/connect/my-post", {
      headers: { Origin: "http://localhost:5173" },
    });

    expect(response.status).not.toBe(403);
  });
});

describe("WebSocket connect", () => {
  it("returns 426 when Upgrade header is absent", async () => {
    const response = await SELF.fetch("http://example.com/api/v1/connect/my-post", {
      headers: { Origin: "http://localhost:5173" },
    });

    expect(response.status).toBe(426);
  });

  it("returns 426 when Upgrade header is not 'websocket'", async () => {
    const response = await SELF.fetch("http://example.com/api/v1/connect/my-post", {
      headers: {
        Origin: "http://localhost:5173",
        Upgrade: "h2c",
      },
    });

    expect(response.status).toBe(426);
  });

  it("returns 403 when path is not allowed", async () => {
    const response = await SELF.fetch("http://example.com/api/v1/connect/secret-page", {
      headers: {
        Origin: "http://localhost:5173",
        Upgrade: "websocket",
      },
    });
    
    expect(response.status).toBe(403);
  });
});
