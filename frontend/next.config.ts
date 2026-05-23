const nextConfig = {
  async rewrites() {
    return [
      {
        source: "/product/:path*",
        destination: "http://localhost:8080/product/:path*",
      },
      {
        source: "/auth/:path*",
        destination: "http://localhost:8080/auth/:path*",
      },
      {
        source: "/pos/:path*",
        destination: "http://localhost:8080/pos/:path*",
      },
      {
        source: "/main/:path*",
        destination: "http://localhost:8080/main/:path*",
      },
    ];
  },
};

export default nextConfig;