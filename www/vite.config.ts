import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync } from "node:fs";
import path from "node:path";
import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import {
  buildDocsSearchAsset,
  buildDocsSearchRecords,
  type DocsSearchAsset,
  type DocsSemanticAsset,
} from "./src/docs/search";
import { renderStaticHomeAndHub } from "./src/docs/site";

const docsSearchIndexDevFileName = "indices/search.json";
const docsSemanticIndexDevFileName = "indices/semantic.json";
const docsSearchIndicesVirtualModuleId = "virtual:docs-search-indices";
const resolvedDocsSearchIndicesVirtualModuleId = `\0${docsSearchIndicesVirtualModuleId}`;
const shouldBuildSemanticIndex = process.env.DOCS_EMBEDDINGS === "true";

function docsSearchIndexPlugin(): Plugin {
  let isDevServer = false;
  let searchAsset: DocsSearchAsset | null = null;
  let semanticAsset: Promise<DocsSemanticAsset> | null = null;
  let searchIndexFileName = "";
  let semanticIndexFileName = "";

  return {
    name: "docs-search-index",
    configResolved(config) {
      isDevServer = config.command === "serve";
    },
    configureServer(server) {
      server.middlewares.use(async (request, response, next) => {
        const requestPath = request.url?.split("?", 1)[0] ?? "";

        if (
          requestPath === `/${docsSearchIndexDevFileName}` ||
          matchesBuiltIndexFile(requestPath, searchIndexFileName)
        ) {
          const asset = getDocsSearchAsset();
          response.setHeader("Content-Type", "application/json");
          response.end(JSON.stringify(asset));
          return;
        }

        if (
          requestPath === `/${docsSemanticIndexDevFileName}` ||
          matchesBuiltIndexFile(requestPath, semanticIndexFileName)
        ) {
          try {
            semanticAsset ??= buildDocsSemanticAsset();
            const asset = await semanticAsset;
            response.setHeader("Content-Type", "application/json");
            response.end(JSON.stringify(asset));
          } catch (error) {
            semanticAsset = null;
            response.statusCode = 500;
            response.setHeader("Content-Type", "application/json");
            response.end(
              JSON.stringify({
                error: error instanceof Error ? error.message : "Semantic index failed to build",
              }),
            );
          }
          return;
        }

        next();
      });
    },
    resolveId(id) {
      if (id === docsSearchIndicesVirtualModuleId) {
        return resolvedDocsSearchIndicesVirtualModuleId;
      }

      return null;
    },
    async load(id) {
      if (id !== resolvedDocsSearchIndicesVirtualModuleId) {
        return null;
      }

      getDocsSearchAsset();

      if (isDevServer || shouldBuildSemanticIndex) {
        try {
          semanticAsset ??= buildDocsSemanticAsset();
          const loadedSemanticAsset = await semanticAsset;
          semanticIndexFileName ||= docsIndexFileName("semantic", loadedSemanticAsset.checksum);
        } catch {
          semanticAsset = null;
          semanticIndexFileName = "";
        }
      }

      return [
        `export const docsSearchIndexUrl = ${JSON.stringify(
          searchIndexFileName ? `/${searchIndexFileName}` : `/${docsSearchIndexDevFileName}`,
        )};`,
        `export const docsSemanticIndexUrl = ${JSON.stringify(
          semanticIndexFileName ? `/${semanticIndexFileName}` : `/${docsSemanticIndexDevFileName}`,
        )};`,
      ].join("\n");
    },
    buildStart() {
      getDocsSearchAsset();

      if (shouldBuildSemanticIndex) {
        semanticAsset = buildDocsSemanticAsset().then((asset) => {
          semanticIndexFileName = docsIndexFileName("semantic", asset.checksum);
          return asset;
        });
      }
    },
    async generateBundle() {
      const loadedSearchAsset = getDocsSearchAsset();

      this.emitFile({
        type: "asset",
        fileName: searchIndexFileName,
        source: JSON.stringify(loadedSearchAsset),
      });

      if (shouldBuildSemanticIndex) {
        semanticAsset ??= buildDocsSemanticAsset();
        const loadedSemanticAsset = await semanticAsset;
        semanticIndexFileName ||= docsIndexFileName("semantic", loadedSemanticAsset.checksum);

        this.emitFile({
          type: "asset",
          fileName: semanticIndexFileName,
          source: JSON.stringify(loadedSemanticAsset),
        });
      }
    },
  };

  function getDocsSearchAsset() {
    searchAsset ??= buildChecksummedDocsSearchAsset();
    searchIndexFileName ||= docsIndexFileName("search", searchAsset.checksum);
    return searchAsset;
  }
}

function matchesBuiltIndexFile(requestPath: string, fileName: string) {
  return fileName !== "" && requestPath === `/${fileName}`;
}

function buildChecksummedDocsSearchAsset(): DocsSearchAsset {
  return withChecksum(buildDocsSearchAsset());
}

async function buildDocsSemanticAsset(): Promise<DocsSemanticAsset> {
  const { env, pipeline } = await import("@huggingface/transformers");
  env.localModelPath = "./public/models/";
  env.allowLocalModels = true;
  env.allowRemoteModels = false;

  const extractor = (await pipeline("feature-extraction", "xmlml6v2", {
    dtype: "q8",
  })) as unknown as {
    (
      text: string,
      options: { pooling: "mean"; normalize: true },
    ): Promise<{
      data: ArrayLike<number>;
    }>;
  };
  const records = [];

  for (const record of buildDocsSearchRecords()) {
    const text = [
      record.title,
      record.category,
      record.summary,
      record.body,
      record.code,
      record.tags,
      record.aliases,
    ].join("\n");
    const output = await extractor(text, { pooling: "mean", normalize: true });
    records.push({
      id: record.id,
      embedding: Array.from(output.data),
    });
  }

  return withChecksum({
    dimensions: 384 as const,
    model: "xmlml6v2" as const,
    records,
  });
}

function withChecksum<T extends object>(asset: T): T & { checksum: string } {
  return {
    ...asset,
    checksum: checksumAsset(asset),
  };
}

function checksumAsset(asset: object) {
  const assetWithoutChecksum: Record<string, unknown> = { ...asset };
  delete assetWithoutChecksum.checksum;

  return createHash("sha256")
    .update(JSON.stringify(assetWithoutChecksum))
    .digest("hex")
    .slice(0, 16);
}

function docsIndexFileName(name: "search" | "semantic", checksum: string) {
  return `indices/${name}.${checksum}.json`;
}

function docsFirstStaticPlugin(): Plugin {
  const fallbackMarker = '<noscript id="docs-first-fallback"></noscript>';

  return {
    name: "docs-first-static",
    transformIndexHtml(html) {
      if (!html.includes(fallbackMarker)) {
        throw new Error("index.html is missing the docs-first noscript marker");
      }
      return html.replace(
        fallbackMarker,
        `<noscript id="docs-first-fallback">${renderStaticHomeAndHub()}</noscript>`,
      );
    },
    writeBundle() {
      const indexPath = path.resolve("dist/index.html");
      const docsDir = path.resolve("dist/docs");
      mkdirSync(docsDir, { recursive: true });
      copyFileSync(indexPath, path.join(docsDir, "index.html"));
    },
  };
}

export default defineConfig({
  assetsInclude: ["**/*.wasm"],
  optimizeDeps: {
    exclude: ["web-tree-sitter"],
  },
  build: {
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes("/node_modules/")) {
            return undefined;
          }

          if (id.includes("/node_modules/react/") || id.includes("/node_modules/react-dom/")) {
            return "vendor-react";
          }

          if (id.includes("/node_modules/@tanstack/react-router/")) {
            return "vendor-router";
          }

          if (id.includes("/node_modules/motion/")) {
            return "vendor-motion";
          }

          if (
            id.includes("/node_modules/@huggingface/transformers/") ||
            id.includes("/node_modules/onnxruntime-web/")
          ) {
            return "vendor-transformers";
          }

          if (id.includes("/node_modules/minisearch/")) {
            return "vendor-search";
          }

          if (id.includes("/node_modules/web-tree-sitter/")) {
            return "vendor-treesitter";
          }

          return "vendor";
        },
      },
    },
  },
  plugins: [docsSearchIndexPlugin(), docsFirstStaticPlugin(), react(), tailwindcss()],
});
