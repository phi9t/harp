import { useEffect, useRef } from "react";

import { canonicalChapters } from "../content/canonical";
import type { CanonicalDocument } from "../content/types";
import { renderCanonicalMath } from "./math";

function secondLevelHeadings(html: string): string[] {
  const parsed = new DOMParser().parseFromString(html, "text/html");
  return Array.from(parsed.querySelectorAll("h2"))
    .map((heading) => heading.textContent?.trim() ?? "")
    .filter((heading) => heading.length > 0);
}

export function CanonicalDocumentView({
  document,
  sectionNavigationLabel,
  sectionId = null,
}: {
  document: CanonicalDocument;
  sectionNavigationLabel?: string;
  sectionId?: string | null;
}) {
  const articleRef = useRef<HTMLElement>(null);
  const sections =
    sectionNavigationLabel === undefined
      ? []
      : secondLevelHeadings(document.html);

  useEffect(() => {
    const article = articleRef.current;
    if (!article) {
      return;
    }
    renderCanonicalMath(article);
    if (!sectionId) {
      return;
    }
    const heading = Array.from(
      article.querySelectorAll<HTMLElement>("[id]"),
    ).find((candidate) => candidate.id === sectionId);
    if (!heading) {
      return;
    }
    heading.tabIndex = -1;
    heading.focus({ preventScroll: true });
    heading.scrollIntoView?.({ block: "start" });
  }, [document.html_sha256, sectionId]);

  const scrollToSection = (section: string): void => {
    const article = articleRef.current;
    const heading = Array.from(article?.querySelectorAll("h2") ?? [])
      .find((candidate) => candidate.textContent?.trim() === section);
    if (!(heading instanceof HTMLElement)) {
      return;
    }
    heading.tabIndex = -1;
    heading.focus({ preventScroll: true });
    heading.scrollIntoView?.({ block: "start" });
  };

  return (
    <div className={sections.length > 0 ? "technical-document" : undefined}>
      {sections.length > 0 && sectionNavigationLabel !== undefined ? (
        <nav
          aria-label={sectionNavigationLabel}
          className="technical-section-nav"
        >
          <p className="eyebrow">In this reading</p>
          <ol>
            {sections.map((section, index) => (
              <li key={section}>
                <button type="button" onClick={() => scrollToSection(section)}>
                  <span>{String(index + 1).padStart(2, "0")}</span>
                  {section}
                </button>
              </li>
            ))}
          </ol>
        </nav>
      ) : null}
      <article
        ref={articleRef}
        aria-label={document.title}
        className="canonical-chapter panel"
        data-html-sha256={document.html_sha256}
        data-markdown-sha256={document.markdown_sha256}
      >
        <div
          className="canonical-markdown"
          dangerouslySetInnerHTML={{ __html: document.html }}
        />
        <footer className="canonical-receipt">
          <span>Canonical Markdown</span>
          <code>{document.canonical_markdown_path}</code>
          <span>SHA-256</span>
          <code>{document.markdown_sha256}</code>
        </footer>
      </article>
    </div>
  );
}

export function ChapterReader({
  document,
  onSelect,
}: {
  document: CanonicalDocument;
  onSelect: (document: CanonicalDocument) => void;
}) {
  return (
    <div className="chapter-reader">
      <aside className="chapter-rail">
        <p className="eyebrow">Technical spine</p>
        <nav aria-label="Technical chapters">
          <ol>
            {canonicalChapters.map((candidate, index) => (
              <li key={candidate.concept_id}>
                <button
                  type="button"
                  aria-current={
                    candidate.concept_id === document.concept_id
                      ? "page"
                      : undefined
                  }
                  onClick={() => onSelect(candidate)}
                >
                  <span>{String(index + 1).padStart(2, "0")}</span>
                  {candidate.title}
                </button>
              </li>
            ))}
          </ol>
        </nav>
        <p>
          Seventy-five retained concepts have one primary explanatory home.
          Source and operational records stay collapsed.
        </p>
      </aside>
      <CanonicalDocumentView document={document} />
    </div>
  );
}
