(() => {
  const attached = new WeakSet();

  const attach = (form) => {
    if (attached.has(form)) {
      return;
    }
    const feedback = form.querySelector("[data-feedback]");
    if (!(feedback instanceof HTMLElement)) {
      throw new Error("quiz form is missing [data-feedback]");
    }
    feedback.setAttribute("aria-live", "polite");
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      const answer = new FormData(form).get("answer");
      const correct = form.dataset.answer;
      feedback.textContent =
        answer === correct
          ? "Correct. Explain why the other choice is weaker."
          : "Not yet. Re-read the edited-object and evidence boundaries.";
    });
    attached.add(form);
  };

  const init = () => {
    document.querySelectorAll("[data-quiz]").forEach((form) => {
      if (form instanceof HTMLFormElement) {
        attach(form);
      }
    });
  };

  window.HarpQuiz = { init };
  init();
})();
