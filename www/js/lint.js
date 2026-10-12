// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: GPL-3.0-or-later

import { wrap } from "comlink";

const instance = new Worker(new URL("./lint.worker", import.meta.url));
const Wrapper = wrap(instance);

const getWorker = async () => {
  const LintWorker = await new Wrapper();
  return LintWorker;
};

export default getWorker();
