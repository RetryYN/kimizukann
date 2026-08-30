use kimizukann_sim_types::{Fixed, TerminationLabel, Thresholds, WorldState, FIXED_SCALE};

#[derive(Clone, Debug)]
pub struct TermState {
    pub fixed_streak: u32,
    pub streak_id: Option<u8>,
    pub tick0_ranking: Vec<u8>,
    pub initial_total_biomass: Fixed,
}

pub fn epsilon(initial: Fixed) -> Result<Fixed, String> {
    i64::try_from((initial as i128) * 100 / 1_000_000).map_err(|_| "epsilon overflow".into())
}

pub fn totals(world: &WorldState) -> Result<([Fixed; 8], Fixed), String> {
    let mut acc = [0i128; 8];
    for cell in &world.grid.cells {
        for (i, b) in cell.biomass.iter().enumerate() {
            acc[i] += i128::from(*b);
        }
    }
    let mut out = [0; 8];
    let mut grand = 0i128;
    for i in 0..8 {
        out[i] = i64::try_from(acc[i]).map_err(|_| "totals overflow")?;
        grand += acc[i];
    }
    Ok((out, i64::try_from(grand).map_err(|_| "totals overflow")?))
}

pub fn share_ge(lineage: Fixed, grand: Fixed, thr: Fixed) -> bool {
    grand != 0
        && (i128::from(lineage) * i128::from(FIXED_SCALE) >= i128::from(grand) * i128::from(thr))
}

pub fn ranking(world: &WorldState) -> Result<Vec<u8>, String> {
    let (tot, _) = totals(world)?;
    let mut ids: Vec<u8> = world.lineages.iter().map(|l| l.id).collect();
    ids.sort_by(|&a, &b| tot[b as usize].cmp(&tot[a as usize]).then(a.cmp(&b)));
    Ok(ids)
}

fn dominant70(tot: &[Fixed; 8], grand: Fixed, ids: &[u8]) -> Option<u8> {
    ids.iter()
        .copied()
        .find(|&id| share_ge(tot[id as usize], grand, 700_000))
}

pub fn step_streak(term: &mut TermState, world: &WorldState) -> Result<(), String> {
    let (tot, grand) = totals(world)?;
    let ids: Vec<u8> = world.lineages.iter().map(|l| l.id).collect();
    match dominant70(&tot, grand, &ids) {
        None => {
            term.fixed_streak = 0;
            term.streak_id = None;
        }
        Some(id) if term.streak_id == Some(id) => {
            term.fixed_streak = term.fixed_streak.saturating_add(1);
        }
        Some(id) => {
            term.streak_id = Some(id);
            term.fixed_streak = 1;
        }
    }
    Ok(())
}

pub fn judge(
    world: &WorldState,
    term: &TermState,
    th: &Thresholds,
    after_step: bool,
) -> Result<Option<TerminationLabel>, String> {
    if !after_step {
        return Ok(None);
    }
    let (tot, grand) = totals(world)?;
    if grand < epsilon(term.initial_total_biomass)? {
        return Ok(Some(TerminationLabel::Extinct));
    }
    let ids: Vec<u8> = world.lineages.iter().map(|l| l.id).collect();
    if term.fixed_streak >= th.fixed_ticks && dominant70(&tot, grand, &ids).is_some() {
        return Ok(Some(TerminationLabel::Fixed));
    }
    if world.tick < th.max_ticks {
        return Ok(None);
    }
    let n15 = ids
        .iter()
        .filter(|&&id| share_ge(tot[id as usize], grand, th.coexist_share))
        .count();
    if n15 >= 2 {
        return Ok(Some(TerminationLabel::Coexist));
    }
    let winner = ranking(world)?.into_iter().next();
    if let Some(winner) = winner {
        let rank0 = term
            .tick0_ranking
            .iter()
            .position(|&id| id == winner)
            .map(|p| p + 1)
            .unwrap_or(1);
        if rank0 >= 3 {
            return Ok(Some(TerminationLabel::Reversal));
        }
    }
    Ok(Some(TerminationLabel::TimeLimit))
}
