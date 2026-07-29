# crep: cargo remote execution protocol

This protocol is in part inspired by the [REAPI (Remote Execution Protocol)](https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto
), used in the bazel build system.

The rargo client (The CLI) and the remote execution service communicate with The crep.