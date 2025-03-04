import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tsconfigPaths from "vite-tsconfig-paths";

const base_api = "http://127.0.0.1:3000";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), tsconfigPaths()],
  server: {
    proxy: {
      "/accountnames": base_api,
      "/accounttransactions": base_api,
      "/accountbalances": base_api,
    },
  },
});
