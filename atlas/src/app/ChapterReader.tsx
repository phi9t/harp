import { useEffect, useMemo, useRef } from "react";

import { canonicalChapters } from "../content/canonical";
import type { CanonicalDocument } from "../content/types";
import { renderCanonicalMath } from "./math";

function secondLevelHeadings(html: string): { id: string; title: string }[] {
  const parsed = new DOMParser().parseFromString(html, "text/html");
  return Array.from(parsed.querySelectorAll("h2"))
    .map((heading) => ({ id: heading.id, title: heading.textContent?.trim() ?? "" }))
    .filter((heading) => heading.title.length > 0);
}

function addressableHtml(html: string): string {
  const parsed = new DOMParser().parseFromString(html, "text/html");
  const used = new Set(Array.from(parsed.querySelectorAll("[id]")).map((node) => node.id));
  for (const heading of parsed.querySelectorAll("h2, h3")) {
    if (heading.id) continue;
    const base = (heading.textContent ?? "").toLowerCase()
      .replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "section";
    let id = base;
    let suffix = 2;
    while (used.has(id)) id = `${base}-${suffix++}`;
    heading.id = id;
    used.add(id);
  }
  return parsed.body.innerHTML;
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
  const html = useMemo(() => addressableHtml(document.html), [document.html]);
  const sections = secondLevelHeadings(html);

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

  const scrollToSection = (id: string): void => {
    const article = articleRef.current;
    const heading = Array.from(article?.querySelectorAll("h2") ?? [])
      .find((candidate) => candidate.id === id);
    if (!(heading instanceof HTMLElement)) {
      return;
    }
    heading.tabIndex = -1;
    heading.focus({ preventScroll: true });
    heading.scrollIntoView?.({ block: "start" });
  };

  return (
    <div className={sections.length > 0 ? "technical-document" : undefined}>
      {sections.length > 0 ? (
        <aside className="technical-section-nav">
          <details open={sectionNavigationLabel !== undefined}>
          <summary>In this reading</summary>
          <nav aria-label={sectionNavigationLabel ?? "Article sections"}>
          <ol>
            {sections.map((section, index) => (
              <li key={section.id}>
                <button type="button" onClick={() => scrollToSection(section.id)}>
                  <span>{String(index + 1).padStart(2, "0")}</span>
                  {section.title}
                </button>
              </li>
            ))}
          </ol>
          </nav>
          </details>
        </aside>
      ) : null}
      <article
        ref={articleRef}
        aria-label={document.title}
        className="canonical-chapter panel"
        data-html-sha256={document.html_sha256}
        data-markdown-sha256={document.markdown_sha256}
      >
        <details className="article-provenance">
          <summary>About this reading and its provenance</summary>
          <p>Document metadata describes this reading, not an independent proof or benchmark verdict.</p>
          <dl className="document-metadata" aria-label="Document metadata">
          <div>
            <dt>Status</dt>
            <dd>{document.metadata.status}</dd>
          </div>
          <div>
            <dt>Confidence</dt>
            <dd>{document.metadata.confidence}</dd>
          </div>
          <div>
            <dt>Kind</dt>
            <dd>{document.metadata.kind}</dd>
          </div>
          {document.metadata.tags.length > 0 ? (
            <div>
              <dt>Tags</dt>
              <dd>{document.metadata.tags.join(", ")}</dd>
            </div>
          ) : null}
          </dl>
        </details>
        <div
          className="canonical-markdown"
          dangerouslySetInnerHTML={{ __html: html }}
        />
        <details className="article-provenance">
          <summary>Canonical Markdown and checksum</summary>
          <footer className="canonical-receipt">
          <span>Canonical Markdown</span>
          <code>{document.canonical_markdown_path}</code>
          <span>SHA-256</span>
          <code>{document.markdown_sha256}</code>
          </footer>
        </details>
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
