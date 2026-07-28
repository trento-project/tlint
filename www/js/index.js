// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

import { basicSetup, EditorView } from "codemirror"
import { yaml } from "@codemirror/lang-yaml"
import { Compartment } from "@codemirror/state"

import Lint from "./lint"
import example from "./example"

const DRAFT_KEY = "tlint:draft";
const SPEC_URL = "https://www.trento-project.io/docs/wanda/specification.html#_anatomy_of_a_check";

const replaceContent = (view, content) => {
    view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: content }
    });
};

// LocalStorage helpers, with try/catch to avoid errors in private browsing mode
const readDraft = () => { try { return localStorage.getItem(DRAFT_KEY) || ""; } catch { return ""; } };
const writeDraft = (content) => { try { localStorage.setItem(DRAFT_KEY, content); } catch { } };
const clearDraft = () => { try { localStorage.removeItem(DRAFT_KEY); } catch { } };

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
    const editable = new Compartment();
    expandSpec.href = SPEC_URL;

    const editor = document.getElementById("editor");
    const code = new EditorView({
        doc: readDraft(),
        extensions: [
            basicSetup,
            yaml(),
            editable.of(EditorView.editable.of(true)),
            EditorView.updateListener.of((update) => {
                if (update.docChanged) {
                    writeDraft(update.state.doc.toString());
                }
            }),
        ],
        parent: editor
    })

    submit.addEventListener("click", async (event) => {
        code.dispatch({ effects: editable.reconfigure(EditorView.editable.of(false)) });
        submit.disabled = true;
        const result = document.getElementById("result");
        result.className = "pending";
        result.textContent = "Linting...";
        const { result: isValid, messages } = await lib.lint(code.state.doc.toString());
        result.textContent = messages.join("\n");
        result.className = isValid ? "ok" : "error";
        code.dispatch({ effects: editable.reconfigure(EditorView.editable.of(true)) });
        submit.disabled = false;
    });

    reset.addEventListener("click", async (event) => {
        replaceContent(code, "");
        clearDraft();
        submit.disabled = false;
        const result = document.getElementById("result");
        result.className = "";
        result.textContent = "";
    });

    loadExample.addEventListener("click", async (event) => {
        replaceContent(code, example);
        writeDraft(example);
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
