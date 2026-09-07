const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { renderForTest } = require('./build.cjs');

test('canonical Harp chapter renders its endpoint and recurrence as mathematics', () => {
  const relative = 'knowledge/crouzeix_textbook/part_06_constant_two_routes/36_harp_finite_horizon_proof.md';
  const source = fs.readFileSync(path.join(__dirname, '../..', relative), 'utf8');
  const html = renderForTest(source, relative);
  const annotations = [...html.matchAll(/<annotation encoding="application\/x-tex">([\s\S]*?)<\/annotation>/g)]
    .map(match => match[1]);
  assert.ok(annotations.some(tex => tex.includes('\\|p(A)\\|\\le 2')),
    'the destination theorem must be typeset, not printed as raw LaTeX');
  for (const number of ['36.1', '36.2']) {
    assert.ok(annotations.some(tex => tex.includes(`\\tag{${number}}`)),
      `equation ${number} must be typeset`);
  }
  assert.ok(annotations.some(tex => tex.includes('\\langle x,y\\rangle=x^*y')),
    'the inline inner-product convention must be typeset');
});

test('canonical audit renders the expanded statement and finite recurrence', () => {
  const relative='knowledge/crouzeix_textbook/harp_mathematical_audit.md';
  const source=fs.readFileSync(path.join(__dirname,'../..',relative),'utf8');
  const html=renderForTest(source,relative);
  const annotations=[...html.matchAll(/<annotation encoding="application\/x-tex">([\s\S]*?)<\/annotation>/g)].map(match=>match[1]);
  assert.ok(annotations.some(tex=>tex.includes('\\|p(A)\\|_{2\\to2}\\le2')));
  assert.ok(annotations.some(tex=>tex.includes('m_{N+1}')&&tex.includes('b_N')));
  assert.ok(annotations.some(tex=>tex.includes('\\operatorname{Re}\\langle v_{N,k},u_k\\rangle')));
  assert.match(html,/HarpStatementAuditTests\.lean/);
  assert.doesNotMatch(html,/class="katex-error"/);
});

test('canonical remainder supplement renders all six tagged equations', () => {
  const relative='knowledge/crouzeix_textbook/harp_finite_horizon_remainder.md';
  const source=fs.readFileSync(path.join(__dirname,'../..',relative),'utf8');
  const html=renderForTest(source,relative);
  const annotations=[...html.matchAll(/<annotation encoding="application\/x-tex">([\s\S]*?)<\/annotation>/g)].map(match=>match[1]);
  for(let i=1;i<=6;i++)assert.ok(annotations.some(tex=>tex.includes(`\\tag{R.${i}}`)));
  assert.match(html,/implicit scalar recurrence estimate/);
  assert.match(html,/not an executable horizon-selection/);
  assert.doesNotMatch(html,/class="katex-error"/);
});
