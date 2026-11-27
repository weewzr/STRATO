//! The comparison of a native reply with the reply of the WebAssembly module (test H).
//!
//! The rules, in the order the comparison applies them. Every leaf of the two replies is compared.
//!
//! 1. The per-value rule ([`PARITY_RTOL`]): `|a - b| <= rtol * max(|a|, |b|)` for every number;
//!    array lengths and object keys are identical; strings, booleans and nulls are equal. The
//!    values of the independent Sturm-Liouville series (see [`is_series_derived`]) get the
//!    series' own absolute `accuracy_floor` of that case in addition. Every physical value
//!    (concentrations, inventories, release fractions, generated and decayed amounts,
//!    diffusivities, fluxes, bounds) is judged by this rule and by no other.
//! 2. The residual rule (decision 76(a)), by field identity ([`residual_identity`]): a balance
//!    RESIDUAL is a rounding-level difference of large numbers, so its relative difference between
//!    two maths libraries is not meaningful. It passes when it lies within its own derived bound
//!    on both platforms and the bound field itself obeys rule 1. The rule never applies to a
//!    field that is not in the list of [`residual_identity`].
//! 3. The flag rule (decision 76(a)), by field identity ([`flag_identity`]): a drawn-or-withheld
//!    flag (and the reason code of a withheld value) at the margin of the drawing mask may differ
//!    between the platforms only where the error estimates of the two platforms agree by rule 1
//!    and lie on either side of the threshold of the mask. The estimates are read from the
//!    fields `cover_relative_difference[]` and `relative_error_estimate[]` and the threshold from
//!    `mask_relative_tolerance` of the same object; a reply without them gives no evidence, and a
//!    differing flag then fails.
//! 4. The reason-swap rule (second integration, fix of the verifier's finding on
//!    `analytical_stage_5_constant`; a ruling to be confirmed by the coordinator), by field
//!    identity ([`reason_swap_identity`]): the reason code of a value that BOTH platforms withhold
//!    may differ between exactly the two codes `COVER_SENSITIVITY` and `ERROR_ESTIMATE_UNDEFINED`.
//!    Both are tests of the same mask, applied in a fixed order, at a value whose released amount
//!    is at rounding level: the first test fails when the reduced-cover series differs by more
//!    than the threshold, the second where the released amount is not positive; which of them
//!    fires first depends on the sign of rounding noise. The value is `null` on both platforms,
//!    so no physical value and no drawn-or-withheld decision is involved. A code against `null`
//!    (drawn on one platform) never passes this rule, nor does any other pair of codes.

use crate::tolerances::PARITY_RTOL;
use serde_json::Value;
use std::collections::BTreeMap;

/// The estimates the flag rule reads, in the object that holds the flag arrays.
pub const FLAG_EVIDENCE: [&str; 2] = ["cover_relative_difference", "relative_error_estimate"];
/// The threshold field of the flag rule.
pub const FLAG_THRESHOLD: &str = "mask_relative_tolerance";

/// What happened to one pattern of failing leaves.
#[derive(Default, Debug, Clone)]
pub struct Failure {
    /// Number of failing leaves.
    pub count: usize,
    /// The first failing pair (native, WebAssembly).
    pub first: Option<(String, String)>,
    /// Worst relative difference.
    pub worst_rel: f64,
}

/// The result for one case.
#[derive(Default, Debug, Clone)]
pub struct Tally {
    /// The case name; it says which function produced the reply.
    pub case: String,
    /// Leaves compared.
    pub compared: usize,
    /// Leaves bit-identical or equal.
    pub identical: usize,
    /// Worst relative difference over all numbers.
    pub worst_rel: f64,
    /// Failing leaves by pattern (array positions shown as `[]`).
    pub failures: BTreeMap<String, Failure>,
    /// Leaves whose field identity is covered by the residual rule.
    pub residual_covered: usize,
    /// Of those, the leaves that differ beyond rule 1 and pass by the residual rule.
    pub residual_excused: usize,
    /// Leaves whose field identity is covered by the flag rule.
    pub flag_covered: usize,
    /// Of those, the leaves that differ and pass by the flag rule.
    pub flag_excused: usize,
    /// Leaves whose field identity is covered by the reason-swap rule (the reason codes).
    pub swap_covered: usize,
    /// Of those, the leaves that differ and pass by the reason-swap rule.
    pub swap_excused: usize,
}

/// The kinds of residual field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidualKind {
    /// The residual at the last row; bound: the sibling `residual_bound_mol`.
    Final,
    /// The largest `|residual|` over the rows; bound: the largest bound over the rows.
    MaxAbs,
    /// The residual at one row; bound: the bound of the same row.
    Row,
    /// The residual of one row over `M0 + generated`; bound: the bound of the row over the same.
    RowRelative,
    /// The largest relative residual over the rows; bound: the largest bound over `M0 + generated`.
    MaxRelative,
}

/// The identity of a residual field: the patterns (array positions as `[]`) that end with one of
/// the names below. A pattern that is not listed is never covered by the residual rule.
pub fn residual_identity(pattern: &str) -> Option<ResidualKind> {
    const FIELDS: [(&str, ResidualKind); 7] = [
        (".balance.final.residual_mol", ResidualKind::Final),
        (".balance.max_abs_residual_mol", ResidualKind::MaxAbs),
        (
            ".diagnostics.max_abs_balance_residual_mol",
            ResidualKind::MaxAbs,
        ),
        (".balance_residual_mol[]", ResidualKind::Row),
        (".balance_residual_relative[]", ResidualKind::RowRelative),
        (
            ".balance.max_abs_residual_relative_to_total",
            ResidualKind::MaxRelative,
        ),
        (
            ".diagnostics.max_abs_balance_residual_relative",
            ResidualKind::MaxRelative,
        ),
    ];
    FIELDS
        .iter()
        .find(|(name, _)| pattern.ends_with(name))
        .map(|(_, k)| *k)
}

/// The identity of a flag field.
pub fn flag_identity(pattern: &str) -> bool {
    [
        ".drawn[]",
        ".drawn_count",
        ".withheld_reason_codes[]",
        ".withheld_reason_codes_of_total[]",
    ]
    .iter()
    .any(|name| pattern.ends_with(name))
}

/// The identity of a field of the reason-swap rule: the reason codes of withheld values.
pub fn reason_swap_identity(pattern: &str) -> bool {
    [
        ".withheld_reason_codes[]",
        ".withheld_reason_codes_of_total[]",
    ]
    .iter()
    .any(|name| pattern.ends_with(name))
}

/// The one pair of codes the reason-swap rule admits, in either order.
pub const SWAPPABLE_REASONS: [&str; 2] = ["COVER_SENSITIVITY", "ERROR_ESTIMATE_UNDEFINED"];

/// The reason-swap rule for one leaf: two different codes (text, so withheld on both platforms)
/// that are the two of [`SWAPPABLE_REASONS`].
pub fn reason_swap_passes(a: &Value, b: &Value) -> bool {
    match (a.as_str(), b.as_str()) {
        (Some(x), Some(y)) => {
            x != y && SWAPPABLE_REASONS.contains(&x) && SWAPPABLE_REASONS.contains(&y)
        }
        _ => false,
    }
}

/// Fields that come from the independent Sturm-Liouville series, or from a difference with it.
/// The label only classifies a failure for the report; it never excuses one.
/// A `compare` reply holds two runs, `.baseline` and `.modified`, each with its own series.
pub fn is_series_derived(case: &str, pattern: &str) -> bool {
    let p = pattern
        .strip_prefix(".baseline")
        .or_else(|| pattern.strip_prefix(".modified"))
        .unwrap_or(pattern);
    // The top-level series fields are the series only in the reply of the `analytical` function
    // (its cases are named `analytical_*`). In the replies of `run`, `compare` and `run_stage`
    // the top-level `release_fraction[]` is the BTCS model series and stays on the bare rule.
    // `run_scenario` returns the series under `.method_b`; `run_stage` only says whether it applies.
    let analytical_reply = case.starts_with("analytical");
    p.starts_with(".analytical.")
        || p.starts_with(".diagnostics.analytical_comparison")
        || p.starts_with(".method_b.release_fraction")
        || p.starts_with(".method_b.missing_weight")
        || p.starts_with(".method_b.withheld_reason_codes")
        || (analytical_reply
            && (p.starts_with(".release_fraction[]")
                || p.starts_with(".release_fraction_of_total[]")
                || p.starts_with(".missing_weight")
                || p.starts_with(".withheld_reason_codes")))
}

/// The series accuracy floor of the native reply that a leaf belongs to: that of the run itself, or
/// for a `compare` reply that of the run (`.baseline` or `.modified`) the leaf lies in.
fn floor_at(native: &Value, path: &str) -> f64 {
    let run = if path.starts_with(".baseline") {
        &native["baseline"]
    } else if path.starts_with(".modified") {
        &native["modified"]
    } else {
        native
    };
    run["analytical"]["accuracy_floor"]
        .as_f64()
        .or_else(|| run["method_b"]["accuracy_floor"].as_f64())
        .or_else(|| run["accuracy_floor"].as_f64())
        .unwrap_or(0.0)
}

/// The pattern of a path: the array positions replaced by `[]`.
pub fn pattern(path: &str) -> String {
    let mut out = String::new();
    let mut in_index = false;
    for ch in path.chars() {
        match ch {
            '[' => {
                in_index = true;
                out.push_str("[]");
            }
            ']' => in_index = false,
            c if !in_index => out.push(c),
            _ => {}
        }
    }
    out
}

/// The value at a path such as `.series.balance_residual_mol[3]` (a leading dot, `.key` and
/// `[index]` steps).
pub fn get<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = root;
    let mut rest = path;
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('.') {
            let end = r.find(['.', '[']).unwrap_or(r.len());
            cur = cur.get(&r[..end])?;
            rest = &r[end..];
        } else {
            let r = rest.strip_prefix('[')?;
            let end = r.find(']')?;
            cur = cur.get(r[..end].parse::<usize>().ok()?)?;
            rest = &r[end + 1..];
        }
    }
    Some(cur)
}

fn number(root: &Value, path: &str) -> Option<f64> {
    get(root, path).and_then(Value::as_f64)
}

/// Rule 1 without the series allowance: `|a - b| <= rtol * max(|a|, |b|)`.
pub fn per_value(a: f64, b: f64) -> bool {
    a == b || (a - b).abs() <= PARITY_RTOL * a.abs().max(b.abs())
}

/// The part of a path before the last occurrence of `marker`, or `None`.
fn before<'a>(path: &'a str, marker: &str) -> Option<&'a str> {
    path.rfind(marker).map(|i| &path[..i])
}

/// The derived bound that goes with a residual leaf of one reply, and the amount that the bound is
/// divided by for a relative residual: `(bound, divisor)`. `None` where the reply lacks what the
/// kind needs, and the residual rule then does not apply.
fn bound_of(reply: &Value, path: &str, kind: ResidualKind) -> Option<f64> {
    // The object that holds the bound arrays: the path before the field, and the same with
    // `.series`, because the stage reply keeps the arrays under `.series`.
    let container = |marker: &str| -> Option<&str> { before(path, marker) };
    let arrays = |base: &str, name: &str| -> Option<Vec<f64>> {
        [format!("{base}.{name}"), format!("{base}.series.{name}")]
            .iter()
            .find_map(|p| {
                get(reply, p).and_then(Value::as_array).map(|a| {
                    a.iter()
                        .map(|v| v.as_f64().unwrap_or(f64::NAN))
                        .collect::<Vec<f64>>()
                })
            })
    };
    let totals = |base: &str| -> Option<Vec<f64>> {
        let start = number(reply, &format!("{base}.balance.start_inventory_mol"))?;
        let gen = arrays(base, "generated_mol")?;
        Some(gen.iter().map(|g| start + g).collect())
    };
    match kind {
        ResidualKind::Final => {
            let base = container(".residual_mol")?;
            number(reply, &format!("{base}.residual_bound_mol"))
        }
        ResidualKind::MaxAbs => {
            let base = container(".balance.max_abs_residual_mol")
                .or_else(|| container(".diagnostics.max_abs_balance_residual_mol"))?;
            let b = arrays(base, "balance_residual_bound_mol")?;
            b.into_iter().reduce(f64::max)
        }
        ResidualKind::Row => {
            let i = path.rfind('[')?;
            let base = &path[..i];
            let bound_path = base.replace("balance_residual_mol", "balance_residual_bound_mol");
            number(reply, &format!("{bound_path}{}", &path[i..]))
        }
        ResidualKind::RowRelative => {
            let i = path.rfind('[')?;
            let idx: usize = path[i + 1..path.len() - 1].parse().ok()?;
            let base = &path[..i];
            let container = base.strip_suffix(".balance_residual_relative")?;
            let bound_path = format!("{container}.balance_residual_bound_mol{}", &path[i..]);
            let bound = number(reply, &bound_path)?;
            let top = container.strip_suffix(".series").unwrap_or(container);
            Some(bound / totals(top)?.get(idx).copied()?)
        }
        ResidualKind::MaxRelative => {
            let base = container(".balance.max_abs_residual_relative_to_total")
                .or_else(|| container(".diagnostics.max_abs_balance_residual_relative"))?;
            let bounds = arrays(base, "balance_residual_bound_mol")?;
            let totals = totals(base)?;
            bounds
                .iter()
                .zip(totals.iter())
                .map(|(b, t)| b / t)
                .reduce(f64::max)
        }
    }
}

/// The residual rule for one leaf: both residuals within their own bound, the bounds equal by rule 1.
fn residual_passes(native: &Value, wasm: &Value, path: &str, kind: ResidualKind) -> bool {
    let (Some(rn), Some(rw)) = (number(native, path), number(wasm, path)) else {
        return false;
    };
    let (Some(bn), Some(bw)) = (bound_of(native, path, kind), bound_of(wasm, path, kind)) else {
        return false;
    };
    rn.abs() <= bn && rw.abs() <= bw && per_value(bn, bw)
}

/// The flag rule for one position: the container of the flag arrays must carry the estimates of
/// both replies, and for at least one estimate the two values must agree by rule 1 and lie on
/// either side of the threshold (one at most, the other above it).
fn flag_passes(native: &Value, wasm: &Value, path: &str) -> bool {
    if let Some(base) = path.strip_suffix(".drawn_count") {
        // The count differs only by the positions of the flag array that differ, each of which
        // must pass the rule itself.
        let flags = |v: &Value| {
            get(v, &format!("{base}.drawn"))
                .and_then(Value::as_array)
                .cloned()
        };
        let (Some(a), Some(b)) = (flags(native), flags(wasm)) else {
            return false;
        };
        let differing: Vec<usize> = (0..a.len().min(b.len()))
            .filter(|&i| a[i] != b[i])
            .collect();
        let count = |v: &[Value]| v.iter().filter(|x| x.as_bool() == Some(true)).count();
        return a.len() == b.len()
            && !differing.is_empty()
            && count(&a).abs_diff(count(&b)) <= differing.len()
            && differing
                .iter()
                .all(|i| flag_passes(native, wasm, &format!("{base}.drawn[{i}]")));
    }
    let (container, index) = match path.rfind('[') {
        Some(i) => (
            &path[..i],
            path[i + 1..].trim_end_matches(']').parse::<usize>().ok(),
        ),
        None => (path, None),
    };
    let Some(base) = container
        .rfind('.')
        .map(|i| &container[..i])
        .filter(|_| index.is_some())
    else {
        return false; // a count has no position: nothing to read
    };
    let index = index.unwrap_or(0);
    let (Some(tn), Some(tw)) = (
        number(native, &format!("{base}.{FLAG_THRESHOLD}")),
        number(wasm, &format!("{base}.{FLAG_THRESHOLD}")),
    ) else {
        return false;
    };
    if tn != tw {
        return false;
    }
    FLAG_EVIDENCE.iter().any(|name| {
        let at = |v: &Value| number(v, &format!("{base}.{name}[{index}]"));
        match (at(native), at(wasm)) {
            (Some(a), Some(b)) => per_value(a, b) && (a <= tn) != (b <= tn),
            _ => false,
        }
    })
}

fn fail(t: &mut Tally, path: &str, a: String, b: String, rel: f64) {
    let f = t.failures.entry(pattern(path)).or_default();
    f.count += 1;
    f.worst_rel = f.worst_rel.max(rel);
    f.first.get_or_insert((a, b));
}

/// The absolute allowance added to the per-value rule for one leaf: the series accuracy floor of
/// the case for series-derived values, except for the two numbers that define the floor.
fn extra_allowance(case: &str, path: &str, native: &Value) -> f64 {
    let pat = pattern(path);
    if is_series_derived(case, &pat)
        && !pat.ends_with(".accuracy_floor")
        && !pat.ends_with(".min_flux_ratio")
    {
        floor_at(native, path)
    } else {
        0.0
    }
}

fn walk(path: &str, a: &Value, b: &Value, native: &Value, wasm: &Value, t: &mut Tally) {
    let pat = pattern(path);
    let residual = residual_identity(&pat);
    let flag = flag_identity(&pat);
    if residual.is_some() {
        t.residual_covered += 1;
    }
    if flag {
        t.flag_covered += 1;
    }
    let swap = reason_swap_identity(&pat);
    if swap {
        t.swap_covered += 1;
    }
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (
                x.as_f64().unwrap_or(f64::NAN),
                y.as_f64().unwrap_or(f64::NAN),
            );
            t.compared += 1;
            if x.to_bits() == y.to_bits() || x == y {
                t.identical += 1;
                return;
            }
            let rel = (x - y).abs() / x.abs().max(y.abs());
            t.worst_rel = t.worst_rel.max(rel);
            let allowed =
                PARITY_RTOL * x.abs().max(y.abs()) + extra_allowance(&t.case, path, native);
            if (x - y).abs() <= allowed {
                return;
            }
            if let Some(kind) = residual {
                if residual_passes(native, wasm, path, kind) {
                    t.residual_excused += 1;
                    return;
                }
            }
            if flag && flag_passes(native, wasm, path) {
                t.flag_excused += 1;
                return;
            }
            fail(t, path, format!("{x:e}"), format!("{y:e}"), rel);
        }
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() {
                fail(
                    t,
                    &format!("{path}.length"),
                    x.len().to_string(),
                    y.len().to_string(),
                    f64::INFINITY,
                );
            }
            for (i, (p, q)) in x.iter().zip(y).enumerate() {
                walk(&format!("{path}[{i}]"), p, q, native, wasm, t);
            }
        }
        (Value::Object(x), Value::Object(y)) => {
            for k in x.keys().chain(y.keys().filter(|k| !x.contains_key(*k))) {
                match (x.get(k), y.get(k)) {
                    (Some(p), Some(q)) => walk(&format!("{path}.{k}"), p, q, native, wasm, t),
                    _ => fail(
                        t,
                        &format!("{path}.{k} (key)"),
                        "present/absent".into(),
                        "absent/present".into(),
                        f64::INFINITY,
                    ),
                }
            }
        }
        (p, q) => {
            t.compared += 1;
            if p == q {
                t.identical += 1;
            } else if flag && flag_passes(native, wasm, path) {
                t.flag_excused += 1;
            } else if swap && reason_swap_passes(p, q) {
                t.swap_excused += 1;
            } else {
                fail(t, path, p.to_string(), q.to_string(), f64::INFINITY);
            }
        }
    }
}

/// Compares the native reply of a case with the reply of the WebAssembly module.
pub fn compare_case(case: &str, native: &Value, wasm: &Value) -> Tally {
    let mut t = Tally {
        case: case.to_string(),
        ..Tally::default()
    };
    walk("", native, wasm, native, wasm, &mut t);
    t
}

/// Whether a failing pattern belongs to the series (for the report only).
pub fn kind_of(case: &str, pattern: &str) -> &'static str {
    if is_series_derived(case, pattern) {
        "series-derived"
    } else {
        "BTCS"
    }
}
