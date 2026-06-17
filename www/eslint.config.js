import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";
import { defineConfig, globalIgnores } from "eslint/config";
import { namingConvention } from "eslint-plugin-naming-convention";

export default defineConfig([
  globalIgnores(["dist"]),
  {
    files: ["**/*.{ts,tsx}"],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      reactHooks.configs.flat.recommended,
      reactRefresh.configs.vite,
      namingConvention,
    ],
    languageOptions: {
      globals: globals.browser,
    },
    rules: {
      "naming-convention/naming-convention": [
        "error",
        {
          selector: "file",
          // Allow lowercase kebab-case for TSX files (e.g., app.tsx, logo.tsx, gradient-background.tsx)
          customRegex: "^[a-z][a-z0-9]*(\\-[a-z0-9]+)*\\.(tsx|ts)$",
          /*
           * Note: We're enforcing lowercase kebab-case for TSX files
           * Examples: app.tsx, logo.tsx, gradient-background.tsx
           * Avoids: App.tsx, myFile.tsx (PascalCase, camelCase)
           */
        },
      ],
    },
  },
]);
