import katex from "katex";

export function renderCanonicalMath(root: ParentNode): void {
  for (const element of root.querySelectorAll<HTMLElement>(".math[data-tex]")) {
    const tex = element.dataset.tex ?? element.textContent ?? "";
    try {
      katex.render(tex, element, {
        displayMode: element.classList.contains("math-display"),
        output: "mathml",
        throwOnError: true,
        trust: false,
        strict: "error",
      });
      element.dataset.tex = tex;
      element.classList.remove("math-error");
    } catch {
      element.replaceChildren(document.createTextNode(tex));
      element.dataset.tex = tex;
      element.classList.add("math-error");
    }
  }
}
