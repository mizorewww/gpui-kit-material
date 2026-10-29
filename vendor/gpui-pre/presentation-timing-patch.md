# Presentation timing contract

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package.

`profiler::CpuFrameTiming` carries `presentation_wait` and
`presentation_waits`. Platform backends call
`profiler::record_presentation_wait` for time blocked inside presentation (for
example `CAMetalLayer nextDrawable`). The wait is attributed to the innermost
open CPU frame scope on the calling thread and only while tracing is enabled.
Scope boundaries and `duration()` are unchanged: consumers keep the raw elapsed
time and decide whether blocked waiting counts as CPU work.

`profiler::GpuExecution`, `record_gpu_execution` and `take_gpu_executions` keep
a bounded (65536 entries, overflow counted), thread-safe store of driver GPU
timestamps reported by platform completion handlers while tracing is enabled.
GPU time is evidence separate from CPU frame scopes.
