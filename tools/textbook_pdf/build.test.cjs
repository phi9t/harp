const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { renderForTest, safeTarget, editionNote } = require('./build.cjs');

test('multiline inline mathematics stays one mathematical expression', () => {
  const html=renderForTest('The identity $\\operatorname{tr}(AB)=\\sum_{i,j}A_{ij}B_{ji}\n+=\\operatorname{tr}(BA)$ holds. Also $A=U\\Sigma V^*$ holds.');
  assert.equal((html.match(/class="katex"/g)||[]).length,2);
  assert.equal((html.match(/<em>/g)||[]).length,0);
  assert.match(html,/encoding="application\/x-tex"/);
});

test('both display styles render and code fences retain literal dollar signs', () => {
  const html=renderForTest('$$x=y$$\n\n$$\na=b\n$$\n\n```text\n$x$\n```');
  assert.equal((html.match(/class="katex-display"/g)||[]).length,2);
  assert.match(html,/<code class="language-text">\$x\$/);
});

test('repeated headings have distinct targets', () => {
  const html=renderForTest('#### Proof\n\nFirst.\n\n#### Proof\n\nSecond.');
  assert.match(html,/id="fixture-proof"/);
  assert.match(html,/id="fixture-proof-2"/);
});

test('editorial roles retain theorem mathematics and Lean correspondence', () => {
  const html=renderForTest('### CFT-01-003 — Coordinate action\n\n**Theorem.** $x=y$.\n\nLean correspondence: `coordinate_action` states the identity.');
  assert.match(html,/<span class="theorem-id">CFT-01-003<\/span>Coordinate action/);
  assert.match(html,/<p class="statement"><strong>Theorem\.<\/strong>/);
  assert.match(html,/class="katex"/);
  assert.match(html,/<p class="evidence-note">Lean correspondence: <code>coordinate_action<\/code> states the identity\./);
});

test('root-qualified wikilinks find extensionless Markdown sources', () => {
  const html=renderForTest('[[tools/textbook_pdf/README|Sources]]');
  assert.match(html,/sources\/tools\/textbook_pdf\/README\.md/);
  assert.doesNotMatch(html,/#edition|unavailable-ref/);
});

test('edition provenance does not claim unpublished local work or a missing chapter',()=>{
  for(const full of [false,true]) {
    const html=editionNote(full);
    assert.match(html,/candidate checkout/);
    assert.doesNotMatch(html,/Chapter 13 uses|recent coordinate workshop|locally compiled Lean additions|replacement is unfinished/);
  }
});

test('unavailable references do not redirect to unrelated edition notes', () => {
  const html=renderForTest('[[knowledge/crouzeix_textbook/nonexistent-fixture|Missing]]');
  assert.match(html,/#unavailable-ref-/);
  assert.doesNotMatch(html,/#edition/);
});

test('generated outputs reject symlinks and path escape', () => {
  const temporary=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'harp-pdf-target-test-')));
  const generated=path.join(temporary,'generated');fs.mkdirSync(generated);
  fs.symlinkSync(path.join(temporary,'outside'),path.join(generated,'linked'));
  assert.throws(()=>safeTarget(path.join(generated,'linked','file.pdf'),generated),/Symlinked/);
  assert.throws(()=>safeTarget(path.join(temporary,'outside.pdf'),generated),/escapes/);
  assert.equal(safeTarget(path.join(generated,'safe.pdf'),generated),path.join(generated,'safe.pdf'));
  fs.unlinkSync(path.join(generated,'linked'));fs.rmdirSync(generated);fs.rmdirSync(temporary);
});
