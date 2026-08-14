/** Browser bindings for the `echo_wasm` check host (`just wasm`). */

export type CheckDiagnostic = {
  severity: "error" | "warning" | "note" | string;
  code?: string;
  message: string;
  path: string;
  line: number;
  column: number;
  end_line: number;
  end_column: number;
};

export type CheckResult = {
  ok: boolean;
  diagnostics: CheckDiagnostic[];
};

export type FormatResult = {
  ok: boolean;
  text?: string;
  diagnostics: CheckDiagnostic[];
};

export type EchoCheckApi = {
  check: (source: string) => CheckResult;
  format: (source: string) => FormatResult;
  stdFileCount: number;
};

type WasmModule = {
  default: () => Promise<unknown>;
  check: (source: string) => string;
  format: (source: string) => string;
  stdFileCount: () => number;
};

let pending: Promise<EchoCheckApi> | null = null;

export class EchoWasmMissingError extends Error {
  constructor() {
    super("The in-browser checker is not built. From a checkout run `just wasm`, then reload.");
    this.name = "EchoWasmMissingError";
  }
}

export function loadEchoCheck(): Promise<EchoCheckApi> {
  pending ??= importEchoWasm();
  return pending;
}

async function importEchoWasm(): Promise<EchoCheckApi> {
  let mod: WasmModule;
  try {
    const specifier = "/echo-wasm/echo_wasm.js";
    mod = (await import(/* @vite-ignore */ specifier)) as WasmModule;
  } catch (error) {
    throw rewriteLoadError(error);
  }

  try {
    await mod.default();
  } catch (error) {
    throw rewriteLoadError(error);
  }

  return {
    check(source: string) {
      return JSON.parse(mod.check(source)) as CheckResult;
    },
    format(source: string) {
      const result = JSON.parse(mod.format(source)) as FormatResult;
      result.diagnostics ??= [];
      return result;
    },
    stdFileCount: mod.stdFileCount(),
  };
}

function rewriteLoadError(error: unknown): Error {
  if (error instanceof EchoWasmMissingError) {
    return error;
  }
  console.error("echo_wasm load failed", error);
  return new EchoWasmMissingError();
}
