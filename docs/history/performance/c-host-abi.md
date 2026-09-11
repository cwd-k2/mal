# C host ABI測定記録

Status: Historical measurement record

2026-09-09にrepositoryのNix development environment、`x86_64-unknown-linux-gnu`、Clang 21.1.8でC ABI `0x000600`の
`socket-packet` generated headerを測定した。

| Host value | `sizeof` | `_Alignof` |
|---|---:|---:|
| `mal_Ptr_t` | 8 | 8 |
| `mal_Socket_t` | 8 | 8 |
| `mal_Symbol_t` | 48 | 8 |
| `mal_Packet_t` | 56 | 8 |
| `mal_ReceiveResult_t` | 64 | 8 |

この値はABI保証ではなく、representation変更時にby-value aggregateのcostを比較するbaselineである。正確なdeclarationは
対象programのgenerated headerを正とする。correctnessとallocation/share countは`compiler/tests/c_emit/host_interface.rs`、
`symbol.rs`、`performance.rs`のnative fixtureで固定する。
