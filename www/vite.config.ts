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

const docsSearchIndexFileName = "indices/search.json";
const docsSemanticIndexFileName = "indices/semantic.json";
const shouldBuildSemanticIndex = process.env.DOCS_EMBEDDINGS === "true";

function docsSearchIndexPlugin(): Plugin {
  let devSemanticAsset: Promise<DocsSemanticAsset> | null = null;

  return {
    name: "docs-search-index",
    configureServer(server) {
      server.middlewares.use(async (request, response, next) => {
        if (request.url === `/${docsSearchIndexFileName}`) {
          response.setHeader("Content-Type", "application/json");
          response.end(JSON.stringify(buildChecksummedDocsSearchAsset()));
          return;
        }

        if (request.url === `/${docsSemanticIndexFileName}`) {
          try {
            devSemanticAsset ??= buildDocsSemanticAsset();
            response.setHeader("Content-Type", "application/json");
            response.end(JSON.stringify(await devSemanticAsset));
          } catch (error) {
            devSemanticAsset = null;
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
    async generateBundle() {
      this.emitFile({
        type: "asset",
        fileName: docsSearchIndexFileName,
        source: JSON.stringify(buildChecksummedDocsSearchAsset()),
      });

      if (shouldBuildSemanticIndex) {
        this.emitFile({
          type: "asset",
          fileName: docsSemanticIndexFileName,
          source: JSON.stringify(await buildDocsSemanticAsset()),
        });
      }
    },
  };
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

// https://vite.dev/config/
export default defineConfig({
  plugins: [docsSearchIndexPlugin(), react(), tailwindcss()],
});
