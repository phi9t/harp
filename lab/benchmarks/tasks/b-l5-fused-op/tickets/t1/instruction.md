Add `float bias;` to `idl/params.idl` after `float scale;`.

The generator preserves IDL field order in `gen/params.h`, so the final C layout must be `scale` followed by `bias`.
Do not edit the generator or generated output by hand.
