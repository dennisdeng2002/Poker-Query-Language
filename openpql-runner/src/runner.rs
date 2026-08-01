// TODO: remove!; tmp implementation
#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;

pub struct PQLRunner {}

/// Once failed samples/rejections outnumber `n_trails` by this factor (with
/// a minimum floor), we give up: a genuinely unsatisfiable range/board/WHERE
/// combination fails on effectively every attempt, so it hits this ceiling
/// almost immediately regardless of how high it is set. A merely *rare* but
/// satisfiable combination (e.g. a pocket pair colliding with a same-rank
/// card fixed on the board) can have a success probability well under 50%,
/// so comparing `n_fail` directly to `n_trails` bails out on those long
/// before enough successes could realistically accumulate.
const MAX_FAIL_RATIO: usize = 1000;
const MIN_FAIL_CEILING: usize = 100_000;

/// Runs `n_trails` successful trials on its own clone of the [`Vm`]
/// (sharing `cache` with the other clones).
fn run_trials(
    mut vm: Vm,
    n_trails: usize,
    where_program: Option<&VmProgram>,
    programs: &[VmProgram],
    selectors: &[ast::Selector],
) -> PQLResult<RunnerOutput> {
    let mut rng = rand::rng();
    let mut output = RunnerOutput::new(vm.static_data.game, selectors);
    let max_fails = n_trails.saturating_mul(MAX_FAIL_RATIO).max(MIN_FAIL_CEILING);

    while output.n_succ < n_trails {
        if output.n_fail >= max_fails {
            return Err(((0, 1), VmError::SamplingFailed).into());
        }

        match vm.sample(&mut rng) {
            Some(()) => {
                if let Some(wp) = where_program {
                    let keep =
                        matches!(wp.execute(&mut vm.as_context())?, VmStackValue::Bool(true));

                    if !keep {
                        //TODO: refine this
                        output.n_fail += 1;
                        continue;
                    }
                }

                for (idx, program) in programs.iter().enumerate() {
                    output.push_value(idx, program.execute(&mut vm.as_context())?);
                }
                output.n_succ += 1;
            }
            None => output.n_fail += 1,
        }
    }

    Ok(output)
}

impl PQLRunner {
    // TODO: check max selectors
    // TODO: refactor run logic
    // TODO: remove
    #[allow(clippy::missing_panics_doc)]
    pub fn try_run_stmt(
        stmt: &ast::Stmt<'_>,
        max_trials: Option<usize>,
        n_threads: Option<usize>,
    ) -> PQLResult<RunnerOutput> {
        let mut vm = Vm::from_stmt(stmt)?;

        if let Some(n) = max_trials {
            vm.static_data.n_trails = n;
        }

        let n_trails = vm.static_data.n_trails;

        let where_program = match &stmt.where_clause {
            Some(expr) => Some(vm::compile_where(&mut vm, expr)?),
            None => None,
        };

        let programs = stmt
            .selectors
            .iter()
            .map(|s| vm::compile_selector(&mut vm, s))
            .collect::<PQLResult<Vec<_>>>()?;

        // wasm has no threads: spawning panics at runtime, so clamp to 1
        // and take the direct path below
        let n_threads = if cfg!(target_family = "wasm") {
            1
        } else {
            n_threads
                .or_else(|| thread::available_parallelism().map(usize::from).ok())
                .unwrap_or(1)
                .clamp(1, n_trails.max(1))
        };

        if n_threads == 1 {
            return run_trials(
                vm,
                n_trails,
                where_program.as_ref(),
                &programs,
                &stmt.selectors,
            );
        }

        let outputs = thread::scope(|scope| {
            let vm = &vm;
            let programs = &programs;
            let where_program = where_program.as_ref();
            let selectors = &stmt.selectors;

            (0..n_threads)
                .map(|i| {
                    // distribute the remainder over the first few threads
                    let quota = n_trails / n_threads + usize::from(i < n_trails % n_threads);

                    scope.spawn(move || {
                        run_trials(vm.clone(), quota, where_program, programs, selectors)
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<PQLResult<Vec<_>>>()
        })?;

        Ok(outputs
            .into_iter()
            .reduce(|mut acc, output| {
                acc.merge(output);
                acc
            })
            .unwrap())
    }

    // tmp function
    pub fn run<S: io::Write, T: io::Write>(
        src: &str,
        max_trials: Option<usize>,
        n_threads: Option<usize>,
        stream_out: &mut S,
        stream_err: &mut T,
    ) -> io::Result<()> {
        match parse_pql(src) {
            Ok(stmts) => {
                for (i, stmt) in stmts.iter().enumerate() {
                    if i > 0 {
                        writeln!(stream_out, "{:-<80}", "")?;
                    }

                    match Self::try_run_stmt(stmt, max_trials, n_threads) {
                        Ok(output) => {
                            output.report_to_stream(stmt, stream_out)?;
                            writeln!(stream_out, "{} trials", output.n_succ)?;
                        }
                        Err(err) => {
                            writeln!(stream_err, "{err:?} {}", &src[err.loc.0..err.loc.1])?;
                        }
                    }
                }
            }
            Err(err) => writeln!(stream_err, "{err:?}")?,
        }
        Ok(())
    }
}
