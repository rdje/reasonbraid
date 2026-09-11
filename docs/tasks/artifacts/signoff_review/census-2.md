# Source census — part 2

Owner: `SIGNOFF-REPAIR.1`. Baseline: `9c2d2ba`. Status of all records: pending reproduction or explicit refutation. Repair contracts: `docs/tasks/SIGNOFF-REPAIR.md`.

## R-47-1

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.1`.
- State: open source-review record; runtime pending.

matching.rs rank latency_score reads rawprofile.cost_latency_class regardlessvisibility (notfilteredvisibleobject), explanation always saysmatchespreferred even score0 mismatch. diversity matchesvalue acrossANY attribute categories ratherthansameattribute, missingmine/allNone gets1.0 maxvariation despiteunknownshould0. fullcandidategroup's heaviestowner overlap allsametenant1 suppressproviderdiversity. Weightsunboundedf64 mayoverflowtotalnonfinite serializationnull.

## R-47-2

- Repair candidates: `SIGNOFF-REPAIR.4.3`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.6.1`, `SIGNOFF-REPAIR.6.2`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

mcp_listen.rs dedupwindow next.pushthen.truncate(64) preservesFIRST64 forever anddropsnewdelivery65+, so newestreplay processedrepeatedly. cursorupdates unconditionallycanregress; ignores_last; firstinsertmissingrowraceuniqueerrorno gaplock. Malformeddedup JSON unwrapordefaultsilentlyforgetsseen. Need runtime 65+eventreconnect monotonic/insertionconcurrency tests.

## R-48-49-1

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.5`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.6.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

mcp_write.respond indexes body[tenant_id] arbitrary JSON, scalar/array can panic insteadtypedinvalidcommand; no typeddeserialize before. gate quota beforeidempotentreplay consumesquota each retry (documentedcountsadmittedcalls). join_call gate own tenant butservicecalltarget notbound, proposegateonlyenrolled noauthorityboundary.

## R-48-49-2

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.10.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node_channel handshake certificateproof lacks nonce/timestamp/serverchallenge. Capturedvalidsigned request replayable untilcert expiry, returnsfreshleasetoken and fencesoriginalnode. rotate proof evenmorecritical replayable staticcoverage overcert+node+version returnsNEWPRIVATEKEY; anyonewithcapturedrotateproof can renewidentityindefinitely. Actual rb-server HTTP no mTLSwire (mtls.rs config exists onlynotmain). Runtimeproofreplaytestpending.

## R-48-49-3

- Repair candidates: `SIGNOFF-REPAIR.4.3`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.5.3`.
- State: open source-review record; runtime pending.

node_channel event_id_for_operation query lacks node_id/tenant, so arbitraryregisterednode suppliesvictimoperationid and receivesvictimeventid andfakeadjudication; event receiptglobaleventIDONCONFLICT ignores mismatchednode/payload lettingothernodepoisondedup. Needscope+payloaddigest conflict.

## R-48-49-4

- Repair candidates: `SIGNOFF-REPAIR.3.5`, `SIGNOFF-REPAIR.4.3`, `SIGNOFF-REPAIR.4.4`.
- State: open source-review record; runtime pending.

node_channel current_cursor MAX liveinbox andenqueueMAX+1. Prune acknowledgedallrows resetscurrent0 soexistingnode highcursor journal_lost; newcursorreuse silentlyignored. Monotonichighwatermustseparatedurablecounter notdeletable inbox.

## R-48-49-5

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node_channel ack onlypre-check fencing thenupdate no txreverify; oldsessionack raceshandshake (eventsdoesproperlock). poll similarreadwindow. renew_lease conditionepochonly noexpires/tokencheckinmutation so can reviveexpiredlease afterprecheckdelay. Revokingcertdoesnotinvalidatelease so heartbeats renewforever andevents continue (boundaryauthorizesworkresultsbutchannelothercommandsstill).

## R-48-49-6

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.2`.
- State: open source-review record; runtime pending.

node_channel rotate verifiescertoutsideinsert transaction andrevokenode loops UPDATEcerts canracefreshrotationinsert leavingnewactivecertafterrevocation. Enrollmentissuedtokenbeforeboundaryrevocation stillvalidconsumption noboundarycheckpending50.

## R-48-49-7

- Repair candidates: `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

mediated.rs RX onlyvocabulary no durablepublishedcall/delivery/answerenforcement, requiressecondverifiernotchecked anddefaultoriginal_not_inspectedfalse. Tests titledroundtripseveryshape only3records+1minimalexcerpt notall6. Claimfullphase4feature requiresremainingtasknotfakecompletion.

## R-50-1

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.8.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

node_enroll no boundary/adminissuerauth reevaluation tokenconsumption; afterboundaryrevocation canstillenroll. replacementenroll keeps nodes.host_id old despitefreshhostclaim+responsehost_id, so subsequentrotationrevertsSANtooldhost. Previousincarnationsvalid_to neverclosed, multiplecurrentrows. No revocation/fencing oldlease onreplacement; oldnodelease persistsuntilhandshake.

## R-50-2

- Repair candidates: `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

outbox::deliver takes event_id/outbox_id nolease/fencing andnevervalidatescorrespondence, so stale worker cancommit delivery afternewlease contrary lib.moduleclaim ('stale worker can nevercommit'). complete doesn'trequire deliveryexists allowsackwithoutdeliver. Existingdeliveryissinkonly, notactualnetworkconsumer; expectedfeaturecompletionaudit.

## R-50-3

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.6.1`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

policy.rs register digestonlyshape, notcanonicaldocument; permitsdeclaredexpiredgrant (statusonly), resolve no boundary/validfrom/actions scope, digestdeadcode neververifiedatuse. is_semvercustomdigits1..3 notSemVer despitelabel. Selector malformedscalar defaults wildcard (failopen). lifecycle draft/superseded/deprecated stillapplicable; onlysuspended/retractedexcluded. Dependenciescheckssetmembership ignoresversions/applicableactivity. Precedencecycleonly2-edgecheck commentclaimsDAG; finishresolutionnextpage.

## R-51-1

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

policy resolution detects onlydirect2cycles thenassertsDAG even3+cycle. exceptionsvalidatednamesagainstANYpolicybutnoactualgrant/expiry/authorityscope orclauseeffect; explanationpromisesseven-stepenforcementbutwaiverjustnames. Needprecisesemanticsandruntime.

## R-51-2

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.10.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

profiles VisibilityPolicydoc claims everyfieldself_only butDefault broadNetwork/Tenant; field_visibledoc reversesladder examplethoughimplementationcorrect. filterFull omitsincarnation_id+visibility despitecommentfullprofile; optionalNonebecomesnull contrarywireabsentclaims.

## R-51-3

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.8.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

profiles write_profile current_version SELECT noFORUPDATE, concurrentwrites collide/reject500, attest readthenwrite losesconcurrentchanges. Capabilityexpiry neverevaluated bymatching; negativeconcurrencySome(-1) presenceavailable/replayadmitted; inputvalidationneeded.

## R-51-4

- Repair candidates: `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.9.2`.
- State: open source-review record; runtime pending.

projections acceptsclientprovidedlock independently ofresolvedpolicylist/digests; recordsnoresolution/provenancebasis inDB exceptartifactbytes, load neverreverifiesdigest. Wholepolicypublicationauthority/canonicaldigest evidencepending nextpage.

## R-52-1

- Repair candidates: `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.10.2`.
- State: open source-review record; runtime pending.

publications stage validatesprojectionexists onlynot linkage toproposal policy/approval scope, acceptsarbitrarymanifestdigestshape nohash. mark_effective acceptsanynonemptystrings notvalidobjectIDs, nofetchback/signature/contentproof gate; effective/failed transitions select+update unlockedcondition absent concurrentterminaloverwrite. Proposalstatusneveradvancepublished inreadmodule.

## R-52-2

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.10.2`, `SIGNOFF-REPAIR.11.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

publisher::publish commit includestimenow =>samecontentdifferentseconds producesdifferentcommit contraryidempotentclaim. Writes staging thenimmutable theneffective separate reftransactions; CASfailure leavesimmutable; retryfailsImmutableExists so strandedstagedworkflow unrecoverable withfunction alone. API no durablepreparedcommit id so retry/reconciliation cannotknowexpected. Gitrefscommit beforeDBmarkeffective potentialorphan/uncertain. Onlyfetchbackmanifestblob (notbundle/tree/commit), doesnotcompare declaredmanifest_digest. gixopen acceptsnonbare repo contrarycontract.

## R-52-3

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

quota::check_in_tx selectsquotarow andcounts withoutFORUPDATE, concurrentperprincipal ormulti-threadtenant actions exceedceiling. countfutureevents(noat<=) can affectdeterministicclock tests;unconfigurednoaudit contrarytypecomment. Storesdenialperrequest unbounded canDoSreceiptstore unlessgateoutside.

## R-52-4

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

reconciler purefunction ignores GitState.effective completely. EffectiveDB+matchingimmutable+missing/moved effective returnsConsistent; noDB+onlyeffective likewiseConsistent. Matrixnotexecutor; no startupreconciliationworkerruns inrb-server.

## R-52-5

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.5.3`.
- State: open source-review record; runtime pending.

receipts agreementremote_ref tenantid notdigest-pinnedremoterecord despitemodulepromise. cardinputchecksum selfprovided notsignedoriginauthentication, beforetreattrustedexternalorigin needverify. userauthorizedoperatorimport may declareoriginbuttrustagreementcannotauthenticatecardsolehash.

## R-53-1

- Repair candidates: `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.7.1`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

recruitment record_response comment sayssecondresponseconflict butUPSERTsilentlyoverwrites payloadkind (originalcreated_atunchanged), nohistory. snapshot_panel INSERTthenUPDATEtwoautocommits no currenteligibility/response serialization, noactualthreadinvitations anywheremodule despiteclaimsameflow. Openingcallservice no threadexistence/tenantvalidation (migrationFKpending).

## R-53-2

- Repair candidates: `SIGNOFF-REPAIR.3.2`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

regions pair/route catchesanySQLerroraspolicyrefusal notstoragefamily, declareemptyidunvalidated (schema mayboundcheckpending). Siteauthfix shouldtypedvalidateauditednoopevidence/unpairreason. Current pairmethods pool transactionsseparate so need in_tx variants.

## R-53-3

- Repair candidates: `SIGNOFF-REPAIR.3.2`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.7.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

resolvers register UPSERTupdatesonlyschemes/egress/sandbox/version/security/maxbytes ignoresmedia/abilities/auth/latency/policies forreplacement, staleadvertises. RegistryclaimssecurityfromselfprovidedJSON withouttrustedqualification; tenantadmincanreplacebuiltins. resolve applies egress declared>=required order needinspect EGRESS_CLASSES alreadyearlier; no locatorpatterns/maxbytes/risk/credentialaudience filtering, ties nondeterministic SQLorder. thirdpartyresolverrankfirst returnsnoacquisition noerror unresolvablefalse silently stopsbuiltin.

## R-53-4

- Repair candidates: `SIGNOFF-REPAIR.4.3`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.7.1`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.11.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

resources globaldeduporiginal_locator ignoresvisibility/credential_binding/hint/risk/owner differences cancross-tenantfirstwriterpoisonresource; schememanualnotderivedoriginalURL R3web+render refhttps expecteddesigndocs; expected_digest nevercompared acquiredbytes API thus referencechecksumnotactualgate.

## R-53-5

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.9.3`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

reviews repeated_waiver triggersafterONEwaiver not repeatedcountthreshold. Fixedreview_id per(pub,trigger), aftermarkdone newoccurrence attemptsINSERTsamePK error swallowed nofuturereviewever; scheduleallSQLinserterrorsignored successfulemptyresult couldmaskstoragefailure. User/windowneedsdurableown.

## R-54-1

- Repair candidates: `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

rls.rs explicitlydevsuperuserbypassesRLS; lifecycle functionsSQLlacktenant predicates meansconfirmedcross-tenant proposal/threadreference visibleunderactualdevconnection; genericreadwithclaim does notenforce onprivilegedrole. Need querypredicates + bootleastprivilegeprofile+migrationrole separation notfalseRLSclaim.

## R-54-2

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.7.1`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.3`.
- State: open source-review record; runtime pending.

snapshots submit trustsbyte_length/origin/ref/resolver/network/auth/disclosurepolicy metadata withoutcrosscheckingactual reference/request; objectinsert thenrowinsert separateautocommits leavesorphans. Replay ignoresfresh_until update contraryhorizonresetscomment, refreshed_atnow doesn'taffectstale filter; snapshotrefetchcannotrestorepreviouslytombstonedliveentry andoldproviderreceiptsunchanged. ignoresrefreshwriteerror.

## R-54-3

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

snapshots expire_due usesSQLnow fordeletioneveninjectedat, twoautocommitclassupdates canpartiallydelete thenerr; unknownretentionclass neverexpires despite helper30daydefault. Tombstoningpreservesrawobjectbytes: compliesmetadatahistorybutnotactualdeleteifuserexpectcleanup;needbookexplicit.

## R-54-4

- Repair candidates: `SIGNOFF-REPAIR.4.2`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

secret_store claimsallkeyreadsrouteprofile butonlyCAloaderimplemented; nodekeys escrow directtable. Externalstore needs codeimplementation despitecomment 'configurationchange notcodemigration'. Scopehonesty docs.

## R-55-1

- Repair candidates: `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.3`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

ssrf IPv6 classifier publicdefault outsidefewblockedranges means non-global reserved/deprecatedsite-local fec0::/10, IPv4compatible::/96, discard100::/64, Teredo2001::/32/6to42002:: privateembedded, newerIANA docs ranges mayclassifypublic. Need browse officialIANAcurrentallocationswhenfix, conservativegloballyroutableallow ratherthanpartialdeny.

## R-55-2

- Repair candidates: `SIGNOFF-REPAIR.7.4`.
- State: open source-review record; runtime pending.

thread module topcomments staleautoaccept/unqualifiedclassificationdespitecurrentbehaviorchanges. Semanticsfullreadongoing; evidenceonlynodedefects track separately.
