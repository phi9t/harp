import { useState } from "react";

import { canonicalCorpus } from "../content/canonical";
import type { CanonicalDocument, CaseId, Lesson } from "../content/types";
import { CanonicalDocumentView } from "./ChapterReader";

function lessonDocument(lesson: Lesson): CanonicalDocument {
  const document = canonicalCorpus.documents.find(
    (candidate) =>
      candidate.canonical_markdown_path === lesson.canonical_markdown_path,
  );
  if (!document) {
    throw new Error(`Validated lesson document disappeared: ${lesson.lesson_id}`);
  }
  return document;
}

function caseTitle(caseId: CaseId): string {
  const diagnosticCase = canonicalCorpus.diagnostics.cases.find(
    (candidate) => candidate.case_id === caseId,
  );
  if (!diagnosticCase) {
    throw new Error(`Validated diagnostic case disappeared: ${caseId}`);
  }
  return diagnosticCase.title;
}

function ruleForCase(caseId: CaseId): string {
  const diagnosticCase = canonicalCorpus.diagnostics.cases.find(
    (candidate) => candidate.case_id === caseId,
  );
  if (!diagnosticCase) {
    throw new Error(`Validated diagnostic case disappeared: ${caseId}`);
  }
  const rule = canonicalCorpus.diagnostics.rules.rules.find(
    (candidate) =>
      candidate.effect.kind === "classification"
      && candidate.effect.classification
        === diagnosticCase.expected_classification,
  );
  if (!rule) {
    throw new Error(`Classification rule disappeared: ${caseId}`);
  }
  return rule.rule_id;
}

export function LessonRunner({
  lessonId,
  onSelectLesson,
  onDiagnose,
}: {
  lessonId: string;
  onSelectLesson: (lessonId: string) => void;
  onDiagnose: (caseId: string) => void;
}) {
  const lesson = canonicalCorpus.lessons.find(
    (candidate) => candidate.lesson_id === lessonId,
  );
  if (!lesson) {
    throw new Error(`Validated lesson disappeared: ${lessonId}`);
  }
  const [prediction, setPrediction] = useState("");
  const [revealed, setRevealed] = useState(false);

  return (
    <div className="lesson-layout">
      <aside className="panel lesson-rail">
        <p className="eyebrow">Comparison lessons</p>
        <nav aria-label="RSI comparison lessons">
          <ol>
            {canonicalCorpus.lessons.map((candidate) => (
              <li key={candidate.lesson_id}>
                <button
                  type="button"
                  aria-current={
                    candidate.lesson_id === lesson.lesson_id ? "page" : undefined
                  }
                  onClick={() => onSelectLesson(candidate.lesson_id)}
                >
                  <span>{String(candidate.order).padStart(2, "0")}</span>
                  {candidate.title}
                </button>
              </li>
            ))}
          </ol>
        </nav>
      </aside>
      <div className="lesson-main">
        <section className="panel lesson-prediction">
          <p className="eyebrow">
            Lesson {String(lesson.order).padStart(2, "0")}
          </p>
          <h1>{lesson.title}</h1>
          <p>
            Predict the editable object, persistent state, evaluator owner,
            generation event, and claim ceiling before revealing the comparison.
          </p>
          <label>
            <span>Your prediction</span>
            <textarea
              rows={6}
              value={prediction}
              onChange={(event) => setPrediction(event.target.value)}
            />
          </label>
          <button
            type="button"
            disabled={prediction.trim() === ""}
            onClick={() => setRevealed(true)}
          >
            Reveal comparison
          </button>
        </section>
        {revealed ? (
          <>
            <section className="panel lesson-case-summary">
              <h2>Deterministic comparison</h2>
              <ul>
                {lesson.case_ids.map((caseId) => {
                  const diagnosticCase = canonicalCorpus.diagnostics.cases.find(
                    (candidate) => candidate.case_id === caseId,
                  );
                  if (!diagnosticCase) {
                    throw new Error(`Lesson case disappeared: ${caseId}`);
                  }
                  return (
                    <li key={caseId}>
                      <strong>{diagnosticCase.title}</strong>
                      <span>
                        {diagnosticCase.expected_classification.replaceAll("-", " ")}
                      </span>
                      <code>{ruleForCase(caseId)}</code>
                    </li>
                  );
                })}
              </ul>
            </section>
            <CanonicalDocumentView document={lessonDocument(lesson)} />
          </>
        ) : null}
      </div>
      <aside className="panel lesson-transfer">
        <p className="eyebrow">Transfer</p>
        <p>Open a fresh source-backed case in the diagnostic workbench.</p>
        <div>
          {lesson.transfer_case_ids.map((caseId) => (
            <button
              type="button"
              key={caseId}
              onClick={() => onDiagnose(caseId)}
            >
              Diagnose {caseTitle(caseId)}
            </button>
          ))}
        </div>
      </aside>
    </div>
  );
}
