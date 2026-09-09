# Exclusive publisher verification fixtures

- Owner: `SIGNOFF-REPAIR.11.4.3.1.4`; REPAIR-0037.
- Baseline: 8542331; only crates/reasonbraid-server/tests/publisher.rs changes among Rust sources.
- Raw evidence: target/publisher-fixture-controls. All focused test, lint, process, source, residue and book results are consumed.

## Reproduction and correction

Extract the exact old repo_dir helper into a standalone Rust probe; remove only
the unused server import and append a readiness/witness/input-wait main. Compile
with CARGO_MANIFEST_DIR pointing to an exclusively created fixture manifest root
under target/publisher-fixture-controls/baseline-c_6uq6_7. The probe retains the
original relative path expression and counter/removal operations. No historical
publisher directory is touched.

Start the first process, verify its witness, and keep it waiting on owned stdin.
Start a second process using the same isolated manifest. Both select pub-0; the
second removes the first witness while the first owner remains alive. Each exits
zero after its input is released and its result consumed. baseline-verification.json
records all four observed properties and both exits; this is actual cross-process
deletion by the old helper, not merely source inference.

The replacement verifies each target/publisher-tests parent component with
symlink_metadata and device identity, creates a UUIDv7-named directory exclusively
with mode 0700, and stores its device/inode identity. Creation collision refuses;
there is no preexisting-entry deletion. finish checks that original directory
identity, removes only its owned tree and checks absence. Tests call finish after
all gix readers and repository handles leave scope. Drop performs no deletion,
so panics, unfinished tests or unconfirmed cleanup retain diagnostics. This is
cooperating local test isolation, not a hostile same-user filesystem race sandbox.
Linux and macOS are the explicit filesystem-control platforms.

Compile the exact replacement helper against an existing project-local uuid
library, with another exclusive fixture manifest root under
target/publisher-fixture-controls/candidate-2wph4dvt. Two simultaneous processes
create different private same-volume directories and retain both witnesses.
Consume the first process: only its directory disappears while the second process
and witness remain intact. Consume the second and confirm its directory is absent.
Both exits are zero. candidate-helper-verification.json records each assertion;
helper source/compiler logs and root/library identities remain alongside it.

The tracked tests preserve both existing publication/fetch-back/CAS/immutability
behaviors and add collision preservation, unfinished-evidence retention and
replaced-directory cleanup refusal. The retention/refusal controls place their
simulated incomplete child under another exclusively owned successful control;
that outer owner cleans its own data after all assertions. They never clean real
failed or historical evidence merely because its test ended.

Historical target/publisher-tests/pub-83115 is preserved. historical-residue.json
records nineteen files with sizes/hashes before final comparison. Production
publisher code, dependencies and publication semantics are unchanged. Broader
publication atomicity and fixture/tool lifecycle review retain their existing
repair owners; this bounded prerequisite closes only its tested fixture contract.

## Verification completion

- `python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-server --test publisher -- --nocapture`: five tests pass, zero ignored/failed, runtime 0.03s, rc=0. candidate.log/.exit preserve the output.
- `python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-server --test publisher --all-features -- -D warnings`: strict focused lint passes, rc=0; clippy.log/.exit retain the result. Workspace formatting passes after formatting the edited test source.
- Launch two copies of the actual publisher-8761e8a234b8367e test executable concurrently: each passes all five controls, rc=0. All ten created directories are distinct and absent afterward. concurrent-0.log, concurrent-1.log and concurrent-verification.json preserve the results. The two-process helper control separately establishes simultaneous live fixture owners and surviving second-owner data after first-owner cleanup.
- All nineteen historical pub-83115 file names, sizes and SHA-256 values match the prior census. The five initial test directories are also absent. Production publisher.rs, server Cargo.toml, Cargo.lock and README are byte-identical to 8542331.
- make book, seven rendered contract markers, final source/exit/residue checks and git diff --check pass, rc=0. final-verification.json retains identities and consumed results. No full CI or push occurs in this leaf.

## Native compiler wait, kept separate from test behavior

Compilation took 5m 17s without intervention; at elapsed 4:38 rustc PID 29306 had
accumulated only 0:02.44 CPU. A one-second installed sample invocation completed
naturally after prolonged symbol processing, rc=0, and its result is consumed.
The compiler-worker's 802 observed stacks are in proc-macro load_dylib through
dyld dlopen/mapSegments to __fcntl; the main thread waits in pthread_join.
compiler-wait.sample and native-wait.json retain this evidence. The later fd/thread
probes returned no such process because it had already finished. Strict lint
included waiting for the same Cargo build lock and completed in 5m 56s.

This locates the sampled native wait; it does not identify the exact fcntl request,
module being loaded, OS security decision, device fault or ultimate delay cause.
No OS setting, signature, attribute or compiler input was changed, and no process
was killed: the sampler had finished before the bounded-stop inspection. Existing
SIGNOFF-REPAIR.11.2 owns further native-loader investigation. The publisher tests
and concurrent copies execute successfully once built; their fixture defect is
reproduced and repaired independently of this host observation.

The next checkpoint prerequisite is browser profile/process lifetime .11.4.3.1.5,
then safe compiler-artifact disposition .6 and the full checkpoint .2.
