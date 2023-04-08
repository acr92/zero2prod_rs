/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    appDir: true,
  },
  rewrites: () => {
    return [{
      source: "/api/:path*",
      destination: "http://127.0.0.1:8000/:path*"
    }]
  }
}

module.exports = nextConfig
