import { defineConfig } from "astro/config";

export default defineConfig({
  site: process.env.SITE_URL ?? "http://localhost",
  base: process.env.BASE_URL ?? "/",
  output: "static",
  build: {
    format: "directory",
  },
});
