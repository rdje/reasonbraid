# Source census — part 4

Owner: `SIGNOFF-REPAIR.1`. Baseline: `9c2d2ba`. Status of all records: pending reproduction or explicit refutation. Repair contracts: `docs/tasks/SIGNOFF-REPAIR.md`.

## R-69-1

- Repair candidates: `SIGNOFF-REPAIR.3.5`, `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.11.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node_inbox quarantine test actually codifiescross-tenantadminmutation:seed_node+enqueue constantseedtenant thenbootstrap_adminfreshdifferenttenant, quarantineauthfreshadminown tenant modifiesseednode. Existinggreenqualificationsnotisolationproof. Needfixtureownertenants+negativecrosstenant anddurablestateunchanged.

## R-69-2

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node_enrollment unusedtokenpernode uniqueindex mayneverexpire entry; expiredunusedtokenpreventsreissuanceforever no revoke/cancel path observed; replacementtokenworkflowwhenconsumed? Needinspectnode_replacement suite next. incarnation testcomment secondtokenunissuable contradictspartialunusedindexusedrowallowsnext issuance.

## R-70-1

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

node_replacement no-false-safe-retry drill loses original uncertainattemptjournal thenoperatorreplaywithno explicitpossibleduplicate acknowledgment allowsredispatch. Refreshed authorityepochdoesn'testablishremoteeffectabsence. FakeLoseResponse doesn'ttrack sideeffects, finalonecontribution proveslocalfoldnotproviderexecution. Needtrackedrecoveryconsent+separatesettlementoflostreservation, preserveambiguitydurablyserverreceipt.

## R-70-2

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.10.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node_replacement test usesFailBeforeDispatchadapter forstaledecision refusal sofailurecan'tproveauthgate denied ratherthanadapter; shouldcountdispatchcalls and usewouldsucceedpositiveadapter. Loops discardworkererrors; exactreasonboundedretryinsufficient isolateauthrejection.

## R-70-3

- Repair candidates: `SIGNOFF-REPAIR.3.5`, `SIGNOFF-REPAIR.4.3`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node_inbox prune onlytests partialprefix (lastcursor5 remains), noallrowsprunethenreconnect test. Core MAXcursorbugremains.

## R-71-72-1

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node_work result_after_close uses emptyreservationid and checksdomainrejection only, doesn'tassertactualproviderusage+settlementrecord. Resultdupnew event iddifferentoperation samecommandcorrectly foldsone viaidempotency; forgedreservationcrosscommandstilluntested.

## R-71-72-2

- Repair candidates: `SIGNOFF-REPAIR.3.4`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.10.1`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

node_work revocationcached test doesuseCompletingadapterandstaleevidence forrefusal (strongerthanreplacement), but finalcontributioncountonly1 doesn'tshowrevise absent (revisiondistinctevent); mustassertadaptercalls plusno revisions ifstrengthening.

## R-71-72-3

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

outbox_worker stale_worker_cannot_complete confirms COMPLETEfencingonly, callscompletewithnodeliveryandexpectsCompleted; publicdeliver hasnofencing. Runtimeproof shouldstage stale deliver vsnewcurrenttoken, notclaimfulldomainfencingfromcompletionalone.

## R-73-74-1

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

policy integration approved forged content digest(allAs)forallpolicies expectedaccepted; onlydigestshapechecked. Precedencecycle only2nodes tests. Approval mismatchedproof testusesGHOSTgrant notactual otherholder; authenticatedcaller spoof stilluntested.

## R-73-74-2

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

policy publish invalidrepositorytest usesalreadyEffectivepub hence400stagegatebeforepathaccess; test doesn'tvalidate invalidrepo errorbranch despitecomment. Emptyquorumtest good ownproposal decvalid. Schema tests verifytypedpartialconditions butapprovallivenessboundaryauditabsent.

## R-73-74-3

- Repair candidates: `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.9.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

policy lifecycle tests verdicttargetsha256:00 unrelatedpolicyaccepted, /effective acceptsfakeabc123def456Gitobjects anddeploymentfixtureusesit. Thisprovesroutebypassrealpublishverification, not merelyimaginedlack.

## R-75-1

- Repair candidates: `SIGNOFF-REPAIR.4.3`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.9.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

policyreviews suite codifies1waiver=>repeated_waiver, onlyreschedulesBEFOREdone no newtriggerafterdone; existingdefectIDdedupepermanentuntested. Suspensionexpiryhardcoded2026-09-15 and lackfuturevalidation? timebrittles whenproperchecksland.

## R-75-2

- Repair candidates: `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.9.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

policycorrection drift testdesired_digest allAs differsactualprojection yetassignmentaccepts; supports declaredvsverifiedbindinggap.

## R-76-77-1

- Repair candidates: `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

profiles directorymatchtestonlyforeigncandidate has self_asserted whileexprrequiresowner_attested; falseisolationcoverage becauseprovenancegatefiltersforeign before visibilitytenantbug. Needforeignprofile SAMEqualifiedcap+full data, emptycapexpr, multipletenant readers.

## R-76-77-2

- Repair candidates: `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.11.2`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

profiles open_callstorm/subscription tests createcalls for nonexistenthardcodedthread IDs andexpect200; source servicechecksnoexistence/tenantconfirmedcodifiedfixtures.

## R-76-77-3

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.4`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

profiles auto test onlyfirstsuccess followedpolicyrefusals; nosecondvalidthread=>fixedidempotencykeydefectuntested. Directgrantseed withlatestauto-onlyalsoshadowspriorcontribute grant, illustratinglatestgrantselection issue.

## R-76-77-4

- Repair candidates: `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

profiles workflowexecutiontest closesfreshindependent_panel directlyblind_solicit→decide withnoevidenceverdict; currentlydocumentedclosefold intentionaltestbut roadmapfullprofileexecution needshonest assessment vs stagegates.

## R-78-1

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.7.1`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

profiles R0/R1/R2/R3/R5 resolver integration onlyloopbackpreflightrefusals (notvalidwholepipeline), R2mediafilterunsupportedPDFnotcaught; actualegressredirect/subrequestsecretleaksuntested. BrokerfixtureunscopedtokenusescallerresourceURL confirmsneedorigin+tenantboundcredentials.

## R-78-2

- Repair candidates: `SIGNOFF-REPAIR.3.2`, `SIGNOFF-REPAIR.7.1`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.10.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

resolverreg test tenantadmin globallyregisterselfdeclared securityrungs; siteoperatorrepairshouldinclude resolvers/workflow? Coredecision sharedregistrywrite scope at leastregionadapterresolver; workflowversionmutationrequiresseparateownedleaf.

## R-80-82-1

- Repair candidates: `SIGNOFF-REPAIR.3.2`, `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.5`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.6.1`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

Source/test review only, no runtime runs: profiles decision-family close negative supplies client unresolved text, does not prove rejection for durable unresolved challenges omitted by caller. Synthesis test accepts nonexistent synthesizer hpr-sy-human instead of enrolled actual principal; record attribution remains client asserted. Verdict accepts sha256:00 absent any claimed target. Quarantine fixture seeds inbox for nonexistent node_id, so fixing node ownership requires real node fixtures. Publisher CAS test proves stale request error but never retries same failed publication id; immutable ref may already exist and strand retry. Reconciler consistent test explicitly accepts matching immutable7 with effective8, showing effective channel mismatch ignored. RLS fixture DROP OWNED/DROP ROLE fixed global rls_probe and global table deletes from arbitrary DATABASE_URL; add disposable test database guard and project-owned role isolation. Registry tests currently treat newly enrolled human tenant-admin as global region admin; replace with explicit site-operator authority and deny tenant-admin even boundary-live/revoked.
