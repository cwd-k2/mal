pub(super) fn buffer_alias() -> &'static str {
    // Buffer object fields and element storage are distinct allocations. Their TBAA types preserve
    // that boundary across inlining. Runtime slot writes do not carry this metadata, so growth still
    // invalidates an active data pointer.
    "!0 = !{!\"Simple C/C++ TBAA\"}\n\
     !1 = !{!\"omnipotent char\", !0, i64 0}\n\
     !2 = !{!\"mal buffer element storage\", !1, i64 0}\n\
     !3 = !{!2, !2, i64 0}\n\
     !4 = distinct !{!4, !\"mal buffer object allocation\"}\n\
     !5 = distinct !{!5, !4, !\"mal buffer object metadata\"}\n\
     !6 = !{!5}\n\
     !7 = !{!\"mal buffer object field\", !1, i64 0}\n\
     !8 = !{!7, !7, i64 0}"
}
