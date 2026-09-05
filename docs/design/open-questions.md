# 未決事項

Status: Current

v0.5の実装挙動に影響する未決事項はない。採択済みの判断は[決定記録](decisions.md)を正とする。

浮動小数点演算が生成するNaNのsignとpayload、およびoperandからのpayload伝播は、programから観測する
bit reinterpretationを言語が持たないため意図的に未指定である。これは[D009](decisions.md#d009-floatは-ieee-754-2019-の固定profileとする)に定める。
