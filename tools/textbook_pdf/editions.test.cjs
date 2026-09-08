const test = require('node:test');
const assert = require('node:assert/strict');
const { readingEditions } = require('./editions.cjs');

test('reading editions preserve Chapter 1 and add the actual Harp chapter', () => {
  const chapters=Array.from({length:36},(_,i)=>({id:`chapter-${i+1}`}));
  const editions=readingEditions(chapters);
  assert.deepEqual(editions.map(e=>e.name),[
    'crouzeix-foundations', 'chapter-01-objects-and-representations',
    'chapter-36-harp-finite-horizon-proof',
  ]);
  assert.equal(editions[0].list,chapters);
  assert.equal(editions[0].full,true);
  assert.equal(editions[1].title,'Objects and Representations');
  assert.deepEqual(editions[1].list,[chapters[0]]);
  assert.equal(editions[2].title,'The Harp finite-horizon proof');
  assert.match(editions[2].subtitle,/Chapter 36/);
  assert.deepEqual(editions[2].list,[chapters[35]]);
  assert.equal(editions[2].full,false);
});

test('incomplete edition input cannot silently substitute another chapter',()=>{
  assert.throws(()=>readingEditions(Array(35).fill({})),/36 chapters/);
});

test('Harp supplements are readable in the full and Chapter 36 editions only',()=>{
  const chapters=Array.from({length:36},(_,i)=>({id:`chapter-${i+1}`}));
  const supplements=[{id:'harp_mathematical_audit'},{id:'harp_finite_horizon_remainder'}];
  const editions=readingEditions(chapters,supplements);
  assert.deepEqual(editions[0].supplements,supplements);
  assert.deepEqual(editions[1].supplements,[]);
  assert.deepEqual(editions[2].supplements,supplements);
  assert.equal(editions[0].list.length,36);
  assert.equal(editions[2].list.length,1);
});
