pub(super) fn indent(output: &mut String, depth: usize) {
    for _ in 0..depth {
        output.push_str("    ");
    }
}
