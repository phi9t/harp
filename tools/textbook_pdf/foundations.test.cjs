const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { renderForTest } = require('./build.cjs');

for (const name of [
  '01_objects_and_representations',
  '02_vector_spaces_and_subspaces',
  '03_linear_maps_and_exact_structure',
  '04_coordinates_and_duality',
]) {
  test(`foundations reading edition renders mathematics and six solution references: ${name}`, () => {
    const relative = `knowledge/crouzeix_textbook/part_01_linear_structure/${name}.md`;
    const source = fs.readFileSync(path.join(__dirname, '../..', relative), 'utf8');
    const html = renderForTest(source, relative);
    assert.match(html, /<math[ >]/, 'the actual chapter must render mathematics');
    assert.doesNotMatch(html, /class="katex-error"/);
    for (let exercise = 1; exercise <= 6; exercise++) {
      assert.ok(html.includes(`exercise_${String(exercise).padStart(2, '0')}_solution`),
        `${name} must expose the named formal solution for exercise ${exercise}`);
    }
    assert.match(html, /Chapter0[1-4]\.lean/, 'readers must be able to reach the Lean code');
  });
}
