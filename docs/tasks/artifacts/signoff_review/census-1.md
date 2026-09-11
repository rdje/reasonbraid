# Source census — part 1

Owner: `SIGNOFF-REPAIR.1`. Baseline: `9c2d2ba`. Status of all records: pending reproduction or explicit refutation. Repair contracts: `docs/tasks/SIGNOFF-REPAIR.md`.

## R-6-27-1

- Repair candidates: `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

Source-review candidates only; runtime reproduction and task-tree ownership MUST follow full-read prerequisite. No edits or tests run.

## R-6-27-2

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.10.1`, `SIGNOFF-REPAIR.10.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

adapter/certification.rs: certifies one Trigger with two common checks + one trigger-specific check but conformance box says six invariants passed. Complete branch accepts FailedKnown terminal as completion. drain unbounded. allowlist ladder needs report verdict/six-box/identity/capability/artifact binding audit.

## R-6-27-3

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.10.1`.
- State: open source-review record; runtime pending.

adapter codex.rs/claude.rs: byte-index stderr tail can split UTF-8 and panic; read_line + conditional len<8192 does not actually bound one long line; child map retains every child; terminal event returns without waiting child/drain. Nonzero subprocess exit/killed child treated definitive remote FailedKnown, needs ambiguity contract audit. Usage mapping against providers unverified.

## R-6-27-4

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

core/budget.rs BudgetDimensions::add uses unchecked u64 + while documentation claims total/fallible. node/supervisor LocalBudget::settle uses subtract(...).unwrap_or_default, hides underflow; signed i64 usage casts to u64; holds in-memory reset on restart.

## R-6-27-5

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.7.3`.
- State: open source-review record; runtime pending.

browse worker: no_sandbox; no user-data-dir override; browser lookup merely exists not provenance/hash pin. request.url unused, step URLs not validated here. Logs only first-party observed async and unbounded; parent_digest is digest of derived text. Need verify caller containment assumptions.

## R-6-27-6

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

extract worker: input whole-read before ceiling; max_chunks/cumulative output only checked after extraction; JS scan follows no indirect PDF references; RSS passed to Atom parser; archive detection extension-only; temp_dir tests locality depends external env.

## R-6-27-7

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.5`, `SIGNOFF-REPAIR.6.1`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

MCP read tools in reasonbraid-mcp/src/lib.rs: list_inbox/get_policy_bundle ignore principal entirely; policy_bundle ignores tenant. get_thread classify grants Network to any other tenant then returns full state, no ThreadInspect/grant/boundary authorization. Module claims same auth as HTTP. Must reproduce and own fix.

## R-6-27-8

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node/channel.rs rotate installs fresh identity only in memory, rb-node cert/key on disk not updated; server-issued key transported HTTP dev. from_hex byte slicing UTF8 panic; fencing token/epoch read with separate locks permits mixed generation. Need cross-check server.

## R-6-27-9

- Repair candidates: `SIGNOFF-REPAIR.3.5`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.10.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node/supervisor completes journal with raw usage only; report chunks returned in RAM. Worker then emits result in separate transaction. Crash between completion and event persistence loses output and terminal retry gate skips. Worker retry refusal reports DEAD LETTER for every terminal including completed (false quarantine), report_dead_letter send never acknowledges local outgoing row. OutcomeUnknown propagates fatal WorkerError, so retry policy may never run except restart. Deadlines constructed but neither supervisor nor adapter enforce. Need tool-backed reproduction after fullread.

## R-31-32-1

- Repair candidates: `SIGNOFF-REPAIR.3.1`, `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.5`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

api.rs replay_command/quarantine_command/inspect_node_inbox/prune_node_inbox authorize req.tenant_id but query only req.node_id, never bind node ownership to tenant (revoke_node DOES explicit ownership). Cross-tenant admin can likely inspect/quarantine/prune/replay arbitrary node commands; replay refreshes victim command with attacker tenant epoch. Runtime pending.

## R-31-32-2

- Repair candidates: `SIGNOFF-REPAIR.3.2`, `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.7.1`, `SIGNOFF-REPAIR.10.1`.
- State: open source-review record; runtime pending.

api.rs register_resolver authorizes caller tenant admin then writes shared resolver registry without tenant scope: additional shared-registry authority endpoint beyond region/adapters. Must include scope census before design.

## R-31-32-3

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.6.1`, `SIGNOFF-REPAIR.7.1`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

api.rs resolve_resource only checks principal enrolled, reads resource by global id, accepts credential binding via shared broker; no owner/authz on acquisition. Snapshot submission errors deliberately ignored R0/R5, book durable evidence claim needs examine.

## R-31-32-4

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.8.2`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

api.rs R3 preflight validates initial URL only then Chromium worker executes normal redirects/subrequests without pinning/routing enforcement; potential SSRF actual browsing. R2 std::env::temp_dir writes require project-local runtime env.

## R-31-32-5

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.5`.
- State: open source-review record; runtime pending.

api.rs issue_node_enroll_token says authorized human but no human restriction; ttl_seconds arbitrary i64 Duration::seconds plus now could panic/overflow; old expired unconsumed unique token blocks reissue unless unused indexexpiry behavior separately examined.

## R-33-35-1

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.8.2`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.9.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

api.rs broad workflow/evaluation/routing/policy/publication/deployment/correction/review/snapshot APIs gate only enrolled principal; many mutation services receive no principal or tenant. Need full service/migration review to confirm unbound input owning_authority-grant impersonation and cross-tenant/global admin writes.

## R-33-35-2

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.9.3`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

api.rs publish_publication accepts arbitrary repo_path directly writes git via publisher::publish under enrollment-only authorization. Declared operator surface comment claims .5 tightened but .5 deployed already. Path/locality/authority restriction absent here.

## R-33-35-3

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

api.rs expire_due_snapshots accepts arbitrary client clock at then expires GLOBAL snapshots; any enrolled principal can tombstone global snapshots by ID or future clock; no owner/admin gate. Need runtime.

## R-33-35-4

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.2`.
- State: open source-review record; runtime pending.

api.rs create_thread_auto uses fixed idempotency key auto_{role}_{tenant}, despite comment server-assigned creation key, so a role can only ever create one unique automatic thread; later body conflicts. Does not carry topics/confidentiality/budget into create body, uses separate max_spend query across all tenant/time scopes not selected authz grant. optional omitted budget bypasses.

## R-33-35-5

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.9.1`.
- State: open source-review record; runtime pending.

api.rs directory_match loads ALL node_presence without tenant column and applies one reader_class to all candidates. Tenant admin Full therefore sees remote tenants' full profiles; any enrolled Tenant search sees remote tenant fields. Filters used for rank use full hidden fields. No per-candidate federation classification. directory_presence narrower propertenantfilter but leaks stable raw role/node IDs as network pseudonyms and state even hidden profile (?) policy investigate.

## R-33-35-6

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

api.rs recruitment respond_to_call_core Role variant sufficient for nonparticipation, no enrollment lookup, permits cross-tenant roles participation with eligibility only, no federation agreement gate; opening call doesn't verify actual thread tenant at endpoint (service pending). close_call allows initiator even revoked grant/boundary, min_participants validated before re-filtering current eligible pool, could close undersized panel. Counts quotas checked outside tx race. Offer inserts after call creation non-atomic.

## R-36-39-1

- Repair candidates: `SIGNOFF-REPAIR.3.1`, `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.5.2`.
- State: open source-review record; runtime pending.

CRITICAL api.rs revoke_grant and revoke_boundary call authority::revoke_* which UPDATE and COMMIT victim grant/boundary BEFORE comparing returned tenant_id with authorized req.tenant_id. Wrong-tenant caller gets404 AFTER actual revocation+victim epoch bump. Authority.rs pages39 proves committed mutation ordering. Need isolated live PostgreSQL RED reproduction and repair leaf ahead feature frontier.

## R-36-39-2

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.8.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

authority::authorize_in_tx loads active boundary by target tenant independent of grant.boundary_id; evaluate+grant_exceeds_boundary never require bound boundary identity. Revoking old boundary then creating new active boundary can reactivate old grants. create_grant subset lacks boundary/tenantidentity equality. Queries load latest active grant only (not any applicable), newer limited grant masks valid grant.

## R-36-39-3

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.4`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.7.1`, `SIGNOFF-REPAIR.8.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

authority::evaluate match ResourceTarget::Tenant accepts ANY selector incl Threads before tenant check. Thread-limited tenant_admin may control entire tenant; ThreadInspect tenantlist leaks allthreadtitles outside grantselector. Delegation dualcheck never checks grant.delegable, boundary.delegable/maxdepth (authz delegate only otherexisting subject). Source docs ADR009 inspect beforefix.

## R-36-39-4

- Repair candidates: `SIGNOFF-REPAIR.3.3`.
- State: open source-review record; runtime pending.

authority authz read queries not locked; separate authorize() commits before admin mutation allows revocation TOCTOU. Even same transaction authorize_in_tx readcommitted no rowlock allows revoked before actualcommit. Need consistent revocation/mutation serialization.

## R-36-39-5

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

api.rs imported card identity/enrollment/receipt commits BEFORE profiles::write_profile; laterfail leaves orphan role, retry displaylabel uniqueness causes500; imported profile provenance upgrades arbitrary cardclaims bypass put_profile selfassertion restriction. No idempotency on identicalcard import, ordinary export timestamp changesdigest.

## R-36-39-6

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.4`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.8.2`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

api::run_thread_command hash excludes target thread_id and authority_context; tenant-wide idempotency key sameprincipaloperationbody reused otherthread returns priorresult instead of mismatch. Authorization after replay means revoked caller gets cached accepted body (intentionaldocs originalresult may readleak assess). Domain errors rollback audit contrary functiondoc every rejectionstored. create_thread writesrouting_resolution before authorize.

## R-36-39-7

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.3`.
- State: open source-review record; runtime pending.

api::apply_node_result_in_tx settlement uses payload.reservation_id arbitrary node-controlled, not reservation bound stored work. Can settle another tenant/thread reservation, selfreportedusage accepted; nousage on refusedresults leaveshelduntilTTL and hides spend. Incarnation linkage uses CURRENTincarnation notdispatch one. Unknown/malformed workevents returnOk stored receipts silently withoutdomain effect.

## R-36-39-8

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.10.1`.
- State: open source-review record; runtime pending.

browse::run_browse waits child exit BEFORE reading piped stdout; output exceeding OS pipe buffer deadlocks untildeadlinekill. Workerstdin writeerror can leakchild; killingonlyworker notchromium subprocess descendants. Same extractionspawner likely inspect next.

## R-36-39-9

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.9.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

rb-server starts migrations prior rejecting invalidsecretstoreprofile (bootclaimfailsclosed but mutatesDB). No configurableauthentication beyond devheaders; defaultsloopback but --hostallowsnonlocal withoutprofilegate. All operator grants under dev header trust must document deployment limits, not claim fullInternetready.

## R-40-42-1

- Repair candidates: `SIGNOFF-REPAIR.4.5`.
- State: open source-review record; runtime pending.

budget.rs reservation ceiling SELECT has no FOR UPDATE/thread/tenant binding; assumes single-writer but API concurrent and per-thread locks do not serialize different ceilings against tenant breaker. Breaker SELECT no lock. Hard bounds race; expiry frees unresolved reservations even remote stillrunning then late settlement overruns. Settlement statusconditionalUPDATE but ignores rows_affected, returnsfalsewinningSettlement. All sum dims overflow as earlier.

## R-40-42-2

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.10.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

ca.rs from_hex validates even bytes but Unicode slicingpanics; issue_node_leaf CertificateParams::new user host_claim expect panic if malformed host. ca expiry one year no renewal boot load checksnone, leaf validity mayexceed CA validity.

## R-40-42-3

- Repair candidates: `SIGNOFF-REPAIR.3.5`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

claims.rs submission.author/verifier taken verbatim notactor binding; anyenrolled writesforgedassessment underanotherauthor. Tombstoned snapshot object stillavailableforcitation; no stale/quarantinepolicygate; anyDBerrmappedSnapshotMissing. Replay ignores differentexcerpt/rationale. Derivation kind notvalidated, dbfailuremappedParentMissing. Need policyclaim clarifyweakcitationexistence notentailment alreadyG5withdrawn.

## R-40-42-4

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.9.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

corrections.rs authority_holds checksonly suppliedgrant idactive/unexpired, notvalidfrom, caller subject, action, selector, boundary or publication scope. Endpointonlyenrolled; arbitrary caller can impersonate anyknownactivegrant. Corrections onlyrecords maynotaffectpublished behavior; docsneedalign.

## R-40-42-5

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.9.3`.
- State: open source-review record; runtime pending.

deployments.rs same unbound owning_authority grantcheck noaction/boundary/caller. assign validatesdigestshape only notactualpublicationprojectiondigest/ref; receipt simplyoverwritesobserved state with node-unbound clientdata, nohistory/audit, can reportapplied despitewrongdigest. Canarywavei64noorderedexecution here records-onlyhonesty.

## R-40-42-6

- Repair candidates: `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

dependence.rs missing attributes give explanation 'no two panel members share... attribute varies across panel', falsely asserts variationwhenallunknown. Duplicateroleids countsameprincipal twice. Morepreciseunknowncoverage needed.

## R-40-42-7

- Repair candidates: `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.8.2`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

evaluation.rs registercorpus checksdigestshape only, nohashbindingto casesJSON; trials arbitrarycaseIDs/arms no corpus/profile membership, dupecasescollapseassignment, usize cast beforemod cross32bitnotstable. calibrations registeredrunsnotcheckedcorpus/workflow, caller-providedBriernotcomputed. evaluate_gate skipsmissing/non-numeric cases andpassesempty scores; no0..1boundonmeasured; falsegreenrequiresred/greeentest. ErrorenumduplicatesusedforallDBfailuresmisdiagnosesdependencyoutage.

## R-40-42-8

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

extraction.rs same confirmed pipedstdoutdeadlock asbrowse: waitexit thenreadstdout, anylargevalidoutputblocks. synchronouspollsleepinsideasyncAPIworkblocksTokioruntimeworker; killcleanup incompleteerrorpaths. tests tinyfeedonly and skipifworkerabsent.

## R-43-1

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.7.1`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

fetcher.rs fetch_authenticated attaches SAME Authorization header on EVERY redirect regardless origin (manual redirect path doesn't scrub unlike clientauto). Broker binding has no tenant/origin/audience policy, so any enrolled resolver user who knows binding can fetch attacker URL withsecret. Criticalsourcecredentialdisclosure runtimewithlocaltestserver authorized fixtures pending.

## R-43-2

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.7.3`.
- State: open source-review record; runtime pending.

fetcher reused directly R2 pipeline butsniff_kind permits only text/HTML, rejectsapplication/pdf/application/zip/application/atom+xml; R2 media acquisitionlikelyneverworks for advertisedtypes exceptmisdeclaredtext. Need runtime R2 end-to-end nottinyworkeronly.

## R-43-3

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

fetcher harden_url attempts rejectalternativenumeric onlyHost::Domain AFTER URLnormalizesnumbersintoIPv4; e.gpublicnumeric normalizesandpassescontrarycontract. Redirect u8 hopcounter canoverflow max_redirects255; headerdefaultportnormalizationcanbypass customports list (productionhttps443default fine). Deflate decoder usesrawdeflate vsHTTPzlibwrapped standard verifyRFCprimarywhenfix. Headertextlabel trustsbytes arbitrarybinary. preflightDNShasnooutertimeceiling (R3).

## R-43-4

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.4`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

federation accept function comments acceptREMOTEproposal but updates own priorproposedrow only; bothsides eachmustproposeacceptown row, docswireexplainneedsalign. Changing pairandscope resetsownstatusonly; oppositeprioracceptautomaticallyparticipatesnewterms butintersectionboolsmakeslimitedscope, no signedversion/policyexpiry. Revocation epoch unchanged forfederation changes so cachedpermissions maylinger iflaterdeliverycaches.

## R-44-45-1

- Repair candidates: `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

git.rs ClassifiedGitHttp auto follows redirects and only classified DNS hook; IP-literal redirected destinations bypassDNS and all URL hardening, allows privateIP SSRF/downgradeHTTP/arbitraryport. No per-hop preflight unlikeclaimedmodule. Tests injectfiletransport, so don'texerciseactualHTTP redirects. BrokerR5separatecritical alreadylogged.

## R-44-45-2

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

git.rs max_time never referenced beyondlimits definition; no acquiretimeout orcancel, blockingclient no explicitbudget, receiveinterrupt AtomicBool neverflips. ODBbytes/object ceiling measured onlyAFTERfullfetch permitsunboundeddisk/decompression. BoundedPostBody::write unboundedVecdespite name andDrop sendsblockingnetwork witherrorslost. Needboundedtransportworker cancellation.

## R-44-45-3

- Repair candidates: `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

git.rs API creates acquisitiontempODB success and only returnsreceipt, dropspathwithoutcleanup/persist snapshot -> leaksprojectdata; acquirestdtemp forbiddenlocalitydefault. git_digest skipsunreadablefiles/read_direrr andhashes concatenatedrawfileswithnoname/length framing givesfalsecomplete; dir_size alsoignoreserrors. collect_files follows symlinkdirectory cycles ifuntrustedfs thoughgixcreatedODBnotremotecheckout.

## R-44-45-4

- Repair candidates: `SIGNOFF-REPAIR.3.4`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

git.rs annotatedtag Ref::Peeled excluded bymatch, try_into_commit nopeel; advertised tag supportbroken. shortbranch/tag ambiguousfirstwins. rawref.. slashgrammarpartial acceptsrefs/.. butgixrejectslater. LFS check arbitrary 'version ht' withinfirst64 falsepositiveplaintext notvalidLFSpointer. gix init_bare defaultglobalconfig mayhonor userenv/cacheoffvolume dependencystoresneedverify beforetests.

## R-46-1

- Repair candidates: `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

lifecycle.rs record_decision and record_approval INSERT child and UPDATE parent separateautocommit operations, no lock/statuscondition, can strand child or regressstatusconcurrency. Electorate/quorum entirely clientprovided nonemptyarray only not snapshotactualparticipants/rulevalidation; 'frozen electorate at action time'overclaim.

## R-46-2

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

lifecycle::record_approval checksinput.approver matchesinput.grant_id subject butendpointdoesnotbindinput.approver toauthenticatedprincipal. noaction/validfrom/boundary/tenantselectorchecks; authorityproof spoofunderany knownactivegrant. registerproposal checks threadusingRLSonly without explicittenantSQL -> privilegedconnection couldbypassRLS; inspect rls/testconfig later.

## R-46-3

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.2`.
- State: open source-review record; runtime pending.

matching.rs eligible follows expression.scope forcapability/interest/confidentiality, but min_concurrency directlycandidate.concurrency regardlessfieldvisibility and permits suspended/unavailable if caller presence_statesrequests. Actualscope mustclamp per candidate atAPI; networkprivacy leaksource confirmed35 even eligibilityusesfilter. Rankpending next.
