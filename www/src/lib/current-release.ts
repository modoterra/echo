/**
 * Current published Echo toolchain prerelease.
 *
 * GitHub `/releases/latest` only resolves a non-prerelease and 404s while every
 * tag is still an alpha. Keep this file, `/install`, README, and docs/install.md
 * aligned when a new tag is published. List only assets on that tag.
 */
export const currentPrereleaseTag = "v0.0.1-alpha.12";

export const currentPrereleaseAssets = [
  {
    artifact: "linux-x86_64",
    archive: "xo-linux-x86_64.tar.gz",
    host: "Linux x86_64",
  },
  {
    artifact: "macos-arm64",
    archive: "xo-macos-arm64.tar.gz",
    host: "macOS arm64",
  },
] as const;

export const currentPrereleaseUrl = `https://github.com/modoterra/echo/releases/tag/${currentPrereleaseTag}`;

export const releasesIndexUrl = "https://github.com/modoterra/echo/releases";
