# Probe Fixtures

The current probes build their minimal fixtures at runtime:

- temporary buffers for buffer, marker, narrowing, Org, and FIM probes
- a temporary process for the asynchronous process-output probe
- installed Doom and Straight files for source/topology probes

Future package probes that need repositories, kernels, or terminal frames should
add deterministic fixtures here rather than using the user's live project state.
