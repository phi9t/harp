import { canonicalCorpus } from "../content/canonical";
import { formatRoute, researchAreas, type ResearchArea } from "./routes";

export const areaLabels = {
  research: "Research",
  execution: "Execution",
  mathematics: "Mathematics",
  evidence: "Evidence",
} satisfies Record<ResearchArea, string>;

const areaHeadings = {
  research: "RSI research",
  execution: "Durable execution",
  mathematics: "Mathematics and Lean",
  evidence: "Evaluation and evidence",
} satisfies Record<ResearchArea, string>;

function orientation() {
  const source = canonicalCorpus.documents.find(
    (document) => document.canonical_markdown_path === "knowledge/harp_knowledge_home.md",
  );
  if (!source) throw new Error("Validated orientation document disappeared");
  const body = new DOMParser().parseFromString(source.html, "text/html").body;
  const title = body.querySelector("h1")?.textContent;
  const introduction = body.querySelector("h1 + p")?.outerHTML;
  if (!title || !introduction) throw new Error("Orientation needs a title and introduction");
  const sections = Array.from(body.querySelectorAll("h2")).map((heading) => {
    const parts: string[] = [];
    let sibling = heading.nextElementSibling;
    while (sibling && sibling.tagName !== "H2") {
      parts.push(sibling.outerHTML);
      sibling = sibling.nextElementSibling;
    }
    return {
      id: heading.id,
      title: heading.textContent ?? "",
      html: parts.join(""),
      introduction: heading.nextElementSibling?.tagName === "P"
        ? heading.nextElementSibling.outerHTML : "",
    };
  });
  for (const area of researchAreas) {
    if (!sections.some((section) => section.title === areaHeadings[area])) {
      throw new Error(`Orientation is missing the ${area} reading path`);
    }
  }
  return { source, title, introduction, sections };
}

const content = orientation();

export function ResearchHome({ area }: { area?: ResearchArea }) {
  const selected = area ? content.sections.find((section) => section.title === areaHeadings[area]) : undefined;
  if (selected) {
    return (
      <section className="research-area" aria-labelledby="area-title">
        <header className="area-heading">
          <a href="#home">Harp research atlas</a>
          <h1 id="area-title">{selected.title}</h1>
        </header>
        <div className="area-layout">
          <div className="area-reading canonical-markdown"
            dangerouslySetInnerHTML={{ __html: selected.html }} />
          <aside className="evidence-margin" aria-label="Reading context">
            <p className="eyebrow">Read with the evidence</p>
            <p>Follow the sources and limits in each reading.</p>
            <a href="#documents/knowledge?section=reading-and-provenance">
              About this research snapshot
            </a>
            {area === "execution" ? <a href="#workstreams">Open Workstreams snapshot</a> : null}
          </aside>
        </div>
      </section>
    );
  }
  return (
    <div className="research-home">
      <header className="research-intro">
        <p className="eyebrow">Harp / research atlas</p>
        <h1>{content.title}</h1>
        <div className="research-deck"
          dangerouslySetInnerHTML={{ __html: content.introduction }} />
        <a className="reading-start" href="#explore/research">Start with RSI research <span aria-hidden="true">↗</span></a>
      </header>
      <section className="reading-paths" aria-labelledby="paths-title">
        <div className="paths-heading">
          <h2 id="paths-title">Explore the atlas</h2>
          <a href="#knowledge">Search the library</a>
        </div>
        {researchAreas.map((key) => {
          const section = content.sections.find((candidate) => candidate.title === areaHeadings[key]);
          if (!section) throw new Error("Validated reading path disappeared");
          return (
            <section className="reading-path" key={key}>
              <h3><a href={formatRoute({ kind: "area", area: key })}>{section.title}<span aria-hidden="true">↗</span></a></h3>
              <div dangerouslySetInnerHTML={{ __html: section.introduction }} />
            </section>
          );
        })}
      </section>
      <section className="further-reading" aria-labelledby="further-title">
        <h2 id="further-title">Further study</h2>
        <div dangerouslySetInnerHTML={{ __html: content.sections.find(
          (section) => section.title === "Further study",
        )?.html ?? "" }} />
      </section>
    </div>
  );
}
