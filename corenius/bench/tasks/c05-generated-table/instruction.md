Fix the status lookup table so it matches `data/status_schema.json`.

`lookup_status(code)` must return the schema label, retry delay in milliseconds, and terminal flag for each known code. Unknown codes must return `Unknown`, retry delay `0`, and terminal `true`. `retryable_codes()` must return the schema codes, in schema order, whose retry delay is greater than zero and whose terminal flag is false.

The repository includes `tools/generate_status_table.py` for updating the generated table from the checked-in schema. The project uses CMake and Ninja. Do not remove the tests.
