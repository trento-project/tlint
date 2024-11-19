import Lint from "./lint"

Lint.then((lib) => {
    document.getElementById("loading").remove();
    document.getElementById("main").style.display = 'block';

    const submit = document.getElementById("submit");
    submit.addEventListener("click", async (event) => {
        const content = document.getElementById("checkContent").value;
        const { result, message } = await lib.lint(content);
        console.log(message)
        document.getElementById("result").innerHTML = message;
        const color = result ? "green" : "red";
        document.getElementById("result").style.backgroundColor = color;
    });
});