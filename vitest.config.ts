import { defineWorkersConfig } from "@cloudflare/vitest-pool-workers/config";

export default defineWorkersConfig({
  test: {
    poolOptions: {
      workers: {
        wrangler: { configPath: "./wrangler.jsonc" },
        miniflare: {
          bindings: {
            ENVIRONMENT: "test",
            ALLOWED_ORIGINS: "http://localhost:5173,http://localhost:5174",
            ALLOWED_PATHS: "my-post",
          },
        },
      },
    },
  },
});
