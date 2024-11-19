import { expose } from 'comlink';

const wasm = import('tlint');

export default class WasmWorker {
  lint(content) {
    return new Promise(async (resolve) => {
      const lib = await wasm;
      const result = lib.lint(content);
      resolve(result);
    });
  }
}

expose(WasmWorker);