// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

import {basicSetup, EditorView} from "codemirror"
import {yaml} from "@codemirror/lang-yaml"

import Lint from "./lint"
import example from "./example"

const DRAFT_KEY = "tlint:draft";
const SPEC_URL = "https://www.trento-project.io/docs/wanda/specification.html#_anatomy_of_a_check";

const replaceContent = (view, content) => {
    view.dispatch({
        changes: {from: 0, to: view.state.doc.length, insert: content}
    });
};

Lint.then((lib) => {
    document.getElementById("loading").remove();
    document.getElementById("main").style.display = 'block';

    const submit = document.getElementById("submit");
    const reset = document.getElementById("reset");
    const loadExample = document.getElementById("load-example");
    const openSpec = document.getElementById("open-spec");
    const closeSpec = document.getElementById("close-spec");
    const expandSpec = document.getElementById("expand-spec");
    const specPanel = document.getElementById("spec-panel");
    const specFrame = document.getElementById("spec-frame");
    expandSpec.href = SPEC_URL;

    const editor = document.getElementById("editor");
    const code = new EditorView({
        doc: localStorage.getItem(DRAFT_KEY) || "",
        extensions: [
            basicSetup,
            yaml(),
            EditorView.updateListener.of((update) => {
                if (update.docChanged) {
                    localStorage.setItem(DRAFT_KEY, update.state.doc.toString());
                }
            }),
        ],
        parent: editor
      })

    submit.addEventListener("click", async (event) => {
        code.editable = false;
        submit.disabled = true;
        const result = document.getElementById("result");
        result.className = "pending";
        result.innerHTML = "Linting...";
        const { result: isValid, messages } = await lib.lint(code.state.doc.toString());
        result.innerHTML = messages.join("\n");
        result.className = isValid ? "ok" : "error";
        code.editable = true;
        submit.disabled = false;
    });

    reset.addEventListener("click", async (event) => {
        replaceContent(code, "");
        localStorage.removeItem(DRAFT_KEY);
        submit.disabled = false;
        const result = document.getElementById("result");
        result.className = "";
        result.innerHTML = "";
    });

    loadExample.addEventListener("click", async (event) => {
        replaceContent(code, example);
        localStorage.setItem(DRAFT_KEY, example);
    });

    openSpec.addEventListener("click", () => {
        if (!specFrame.src) {
            specFrame.src = SPEC_URL;
        }
        specPanel.classList.add("open");
        specPanel.setAttribute("aria-hidden", "false");
    });

    closeSpec.addEventListener("click", () => {
        specPanel.classList.remove("open");
        specPanel.setAttribute("aria-hidden", "true");
    });
});