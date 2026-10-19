gen_rule(
    name="record_codegen",
    srcs=["record.proto"],
    outs=["record_schema.h", "record.desc"],
    outdir="proto/generated",
    cmd="mkdir -p proto/generated && sh cpp3rdlib/protobuf/bin/protoc --descriptor_set_out=proto/generated/record.desc --proto_path=proto $FIRST_SRC && python3 proto_codegen.py $FIRST_SRC proto/generated/record_schema.h",
)
