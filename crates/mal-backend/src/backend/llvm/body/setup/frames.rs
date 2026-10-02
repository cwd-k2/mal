//! The states that can suspend into a control frame, numbered by the tags their frames carry.

use super::*;

/// Frame sites in state order with their tags. Rejects a frame this emitter cannot resume: one reached through another
/// function's region or call mode, an environment-carrying frame outside a common region, or a field without a runtime
/// type. Local control storage needs at least one frame site.
pub(super) fn frame_sites(
    execution: &crate::execution::Program,
    states: &[StateId],
    id: FunctionId,
    common_region: Option<ControlRegionId>,
    types: &Types,
    localizes_control: bool,
) -> Option<(Vec<StateId>, HashMap<StateId, u32>)> {
    let frame_sites = states
        .iter()
        .filter(|site| execution.control_frames.frame(**site).is_some())
        .copied()
        .collect::<Vec<_>>();
    let frame_tags = frame_sites
        .iter()
        .enumerate()
        .map(|(tag, site)| Some((*site, u32::try_from(tag).ok()?)))
        .collect::<Option<HashMap<_, _>>>()?;
    if localizes_control && frame_sites.is_empty() {
        return None;
    }
    if frame_sites.iter().any(|site| {
        let frame = execution
            .control_frames
            .frame(*site)
            .expect("collected frame site");
        (common_region.is_none()
            && (execution.control_calls.mode(*site) != Some(ControlCallMode::DirectRegion(id))
                || frame.carries_environment))
            || common_region
                .is_some_and(|region| execution.control_regions.site_region(*site) != Some(region))
            || frame
                .fields
                .iter()
                .any(|field| types.value(&field.ty).is_none())
    }) {
        return None;
    }
    Some((frame_sites, frame_tags))
}
