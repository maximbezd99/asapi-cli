import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(({ mode }) => {
  const environment = loadEnv(mode, ".", "ASAPI_");
  return {
    plugins: [react()],
    server: {
      port: 5173,
      proxy: {
        "/api":
          environment.ASAPI_API_ORIGIN ?? "http://127.0.0.1:3000",
      },
    },
  };
});
