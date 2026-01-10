import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* config options here */
  reactCompiler: true,
  allowedDevOrigins: ["localhost:3000", "100.65.129.108:3000"],
  productionBrowserSourceMaps: false,
  experimental: {
    workerThreads: false,
    cpus: 2,
  },
};

export default nextConfig;
