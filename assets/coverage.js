(() => {
  const normalize = (value) => value.trim().toLowerCase();

  const init = () => {
    const query = document.querySelector("[data-source-query]");
    const count = document.querySelector("[data-result-count]");
    const cards = [...document.querySelectorAll("[data-source-card]")];
    if (!(query instanceof HTMLInputElement) || !(count instanceof HTMLElement)) {
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
  };

  init();
})();
