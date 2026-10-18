The workspace is a Blade IDL/codegen task. It uses a local
`cpp3rdlib/protobuf` package whose `bin/protoc` wrapper calls the capsule's
host `/usr/bin/protoc`.

This task uses a `gen_rule` fallback instead of Blade `proto_library`: in the
capsule, the host protobuf runtime libraries require newer `GLIBCXX_3.4.29` and
`GLIBC_2.34` symbols than the Blade gcc 8.3 link environment supplies. The
fallback still runs `protoc` and then generates the small C++ header consumed by
the app.

The schema now includes a `shard` field, and the app must use generated schema
metadata for that field. Repair the Blade wiring so:

```sh
blade-offline build //app:proto_app
```

builds, and the app prints:

```text
PROTO_RESULT id=1 shard=2 name=shard
```

Do not replace the app with hard-coded output; keep the schema-driven generated
header in the build path.
