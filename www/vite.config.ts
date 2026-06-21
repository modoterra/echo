import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import {
  buildDocsSearchAsset,
  buildDocsSearchRecords,
  type DocsSemanticAsset,
} from "./src/docs/search";

const docsSearchIndexFileName = "docs-search-index.json";
const docsSemanticIndexFileName = "docs-semantic-index.json";
const shouldBuildSemanticIndex = process.env.DOCS_EMBEDDINGS === "true";

function docsSearchIndexPlugin(): Plugin {
  return {
    name: "docs-search-index",
    configureServer(server) {
      server.middlewares.use((request, response, next) => {
        if (request.url !== `/${docsSearchIndexFileName}`) {
          next();
          return;
        }

        response.setHeader("Content-Type", "application/json");
        response.end(JSON.stringify(buildDocsSearchAsset()));
      });
    },
    async generateBundle() {
      this.emitFile({
        type: "asset",
        fileName: docsSearchIndexFileName,
        source: JSON.stringify(buildDocsSearchAsset()),
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

  return {
    dimensions: 384,
    model: "xmlml6v2",
    records,
  };
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [docsSearchIndexPlugin(), react(), tailwindcss()],
});
