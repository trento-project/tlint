import {basicSetup, EditorView} from "codemirror"
import {yaml} from "@codemirror/lang-yaml"

import Lint from "./lint"

Lint.then((lib) => {
    document.getElementById("loading").remove();
    document.getElementById("main").style.display = 'block';

    const submit = document.getElementById("submit");
    const reset = document.getElementById("reset");

    const editor = document.getElementById("editor");
    const code = new EditorView({
        doc: "",
        extensions: [basicSetup, yaml()],
        parent: editor
      })

    submit.addEventListener("click", async (event) => {
        code.editable = false;
        submit.disabled = true;
        document.getElementById("result").innerHTML = "Linting...";
        document.getElementById("result").style.backgroundColor = "gray";
        const { result, messages } = await lib.lint(code.state.doc.toString());
        document.getElementById("result").innerHTML = messages.join("\n");
        document.getElementById("result").style.backgroundColor = result ? "green" : "red";
        code.editable = true;
        submit.disabled = false;
    });

    reset.addEventListener("click", async (event) => {
        code.dispatch({
            changes: {from: 0, to: code.state.doc.toString().length, insert:""}
        })
        submit.disabled = false;
        document.getElementById("result").style.removeProperty("background-color");
        document.getElementById("result").innerHTML = "";
    });
});