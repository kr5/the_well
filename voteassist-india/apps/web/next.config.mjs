/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  transpilePackages: ["@voteassist/decision-engine", "@voteassist/i18n", "@voteassist/knowledge"],
};

export default nextConfig;
