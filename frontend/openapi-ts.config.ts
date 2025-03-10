import { defineConfig } from "@hey-api/openapi-ts";

export default defineConfig({
  input: "http://localhost:5173/api/openapi",
  output: {
    path: "src/client",
    format: "prettier",
  },
  plugins: [
    { name: "@hey-api/client-fetch", runtimeConfigPath: "./src/hey-api.ts" },
  ],
});
