`run_workers.sh STATE` starts two workers against the same state directory. At most one worker may enter the critical section. A worker that cannot acquire the lock must report `busy ID` and exit with status `75`; the acquired worker reports `acquired ID`, appends its ID to `STATE/results.txt`, and releases the lock before the CLI exits.

The verifier is race-tolerant: either worker may win. The required outcome for each run is exactly one acquired worker, exactly one busy worker, one result line, and no remaining `STATE/worker.lock` path.
