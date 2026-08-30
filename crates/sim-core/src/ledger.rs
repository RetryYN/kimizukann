use kimizukann_sim_types::{Fixed, Pool, ReasonCode};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerRecord {
    pub tick: u32,
    pub region_id: u8,
    pub lineage: u8,
    pub reason: ReasonCode,
    pub from_pool: Pool,
    pub to_pool: Pool,
    pub amount: Fixed,
}

pub fn fold_region_records(rows: &mut Vec<LedgerRecord>) {
    rows.sort_by_key(|r| {
        (
            r.tick,
            r.region_id,
            r.lineage,
            r.reason as u8,
            r.from_pool as u8,
            r.to_pool as u8,
        )
    });
    let mut out: Vec<LedgerRecord> = Vec::with_capacity(rows.len());
    for row in rows.drain(..) {
        if let Some(last) = out.last_mut() {
            if last.tick == row.tick
                && last.region_id == row.region_id
                && last.lineage == row.lineage
                && last.reason == row.reason
                && last.from_pool == row.from_pool
                && last.to_pool == row.to_pool
            {
                last.amount += row.amount;
                continue;
            }
        }
        out.push(row);
    }
    *rows = out;
}
