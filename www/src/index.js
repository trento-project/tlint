import Lint from "./lint"

Lint.then((lib) => {
    document.getElementById("loading").remove();
    document.getElementById("main").style.display = 'block';

    const submit = document.getElementById("submit");
    submit.addEventListener("click", async (event) => {
        const textarea = document.getElementById("checkContent");
        textarea.disabled = true;
        submit.disabled = true;
        document.getElementById("result").innerHTML = "Linting...";
        document.getElementById("result").style.backgroundColor = "gray";
        const { result, messages } = await lib.lint(textarea.value);
        document.getElementById("result").innerHTML = messages.join("\n");
        document.getElementById("result").style.backgroundColor = result ? "green" : "red";
        textarea.disabled = false;
        submit.disabled = false;
    });
});