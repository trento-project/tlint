// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

import { wrap } from "comlink";

const instance = new Worker(new URL("./lint.worker", import.meta.url));
const Wrapper = wrap(instance);

const getWorker = async () => {
  const LintWorker = await new Wrapper();
  return LintWorker;
};

export default getWorker();