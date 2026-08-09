(() => {
  const attachedQueries = new WeakSet();
  const attachedRandomizers = new WeakSet();
  const normalize = (value) => value.trim().toLowerCase();

  const initSourceFilter = () => {
    const query = document.querySelector("[data-source-query]");
    const count = document.querySelector("[data-result-count]");
    const cards = [...document.querySelectorAll("[data-source-card]")];
    if (
      !(query instanceof HTMLInputElement) ||
      !(count instanceof HTMLElement) ||
      attachedQueries.has(query)
    ) {
      return;
    }
    count.setAttribute("aria-live", "polite");

    const apply = () => {
      const needle = normalize(query.value);
      let visible = 0;
      cards.forEach((card) => {
        if (!(card instanceof HTMLElement)) {
          return;
        }
        const haystack = normalize(
          `${card.dataset.sourceId ?? ""} ${card.dataset.section ?? ""} ` +
            `${card.dataset.family ?? ""} ${card.textContent ?? ""}`,
        );
        const show = needle === "" || haystack.includes(needle);
        card.hidden = !show;
        visible += Number(show);
      });
      count.textContent = `${visible} of ${cards.length} sources`;
    };

    query.addEventListener("input", apply);
    apply();
    attachedQueries.add(query);
  };

  const sourceOptions = (list) =>
    [...list.options].filter((option) => option.value.trim() !== "");

  const pickSource = (options, randomValue = Math.random()) => {
    if (options.length === 0) {
      return null;
    }
    const index = Math.min(
      options.length - 1,
      Math.floor(randomValue * options.length),
    );
    return options[index];
  };

  const initRandomSource = () => {
    const button = document.querySelector("[data-random-source]");
    const output = document.querySelector("[data-random-source-output]");
    const list = document.querySelector("[data-random-source-list]");
    if (
      !(button instanceof HTMLButtonElement) ||
      !(output instanceof HTMLElement) ||
      !(list instanceof HTMLSelectElement) ||
      attachedRandomizers.has(button)
    ) {
      return;
    }

    const options = sourceOptions(list);
    button.addEventListener("click", () => {
      const selected = pickSource(options);
      if (selected === null) {
        output.textContent = "No source cards are available.";
        return;
      }
      const link = document.createElement("a");
      link.href = selected.value;
      link.textContent = selected.textContent.trim();
      output.replaceChildren("Review ", link, " after answering from memory.");
    });
    attachedRandomizers.add(button);
  };

  const init = () => {
    initSourceFilter();
    initRandomSource();
  };

  window.HarpCoverage = { init, pickSource };
  init();
})();
