import { createHash } from "node:crypto";
import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import {
  buildDocsSearchAsset,
  buildDocsSearchRecords,
  type DocsSearchAsset,
  type DocsSemanticAsset,
} from "./src/docs/search";

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
        if (
          request.url === `/${docsSearchIndexDevFileName}` ||
          request.url === `/${searchIndexFileName}`
        ) {
          const asset = getDocsSearchAsset();
          response.setHeader("Content-Type", "application/json");
          response.end(JSON.stringify(asset));
          return;
        }

        if (
          request.url === `/${docsSemanticIndexDevFileName}` ||
          request.url === `/${semanticIndexFileName}`
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
                error:
                  error instanceof Error
                    ? error.message
                    : "Semantic index failed to build",
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
        semanticAsset ??= buildDocsSemanticAsset();
        const loadedSemanticAsset = await semanticAsset;
        semanticIndexFileName ||= docsIndexFileName(
          "semantic",
          loadedSemanticAsset.checksum,
        );
      }

      return [
        `export const docsSearchIndexUrl = ${JSON.stringify(
          searchIndexFileName
            ? `/${searchIndexFileName}`
            : `/${docsSearchIndexDevFileName}`,
        )};`,
        `export const docsSemanticIndexUrl = ${JSON.stringify(
          semanticIndexFileName
            ? `/${semanticIndexFileName}`
            : `/${docsSemanticIndexDevFileName}`,
        )};`,
      ].join("\n");
    },
    buildStart() {
      getDocsSearchAsset();

      if (shouldBuildSemanticIndex) {
        semanticAsset = buildDocsSemanticAsset().then((asset) => {
          semanticIndexFileName = docsIndexFileName(
            "semantic",
            asset.checksum,
          );

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
        semanticIndexFileName ||= docsIndexFileName(
          "semantic",
          loadedSemanticAsset.checksum,
        );

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

function buildChecksummedDocsSearchAsset(): DocsSearchAsset {
  return withChecksum(buildDocsSearchAsset());
}

async function buildDocsSemanticAsset(): Promise<DocsSemanticAsset> {
  const { env, pipeline } = await import("@huggingface/transformers");
  env.localModelPath = "./public/models/";
  env.allowLocalModels = true;
  env.allowRemoteModels = false;

  const extractor = (await pipeline(
    "feature-extraction",
    "xmlml6v2",
    { dtype: "q8" },
  )) as unknown as {
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
    dimensions: 384,
    model: "xmlml6v2",
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

// https://vite.dev/config/
export default defineConfig({
  plugins: [docsSearchIndexPlugin(), react(), tailwindcss()],
});
