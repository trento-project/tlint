// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

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