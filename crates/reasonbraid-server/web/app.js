"use strict";

// ReasonBraid inspection console (`PHASE-1.6.2`) — read-only vanilla JS over the
// existing GET surfaces. The CLI stays the primary surface: this page adds no
// write path and no authority; every fetch is same-origin with the dev-profile
// principal header, so the server's gates apply exactly as they do to the CLI.
//
// XSS discipline: every datum renders through `el()`/`textContent` — HTML is
// never assembled from data. Thread content, evidence URIs, and provider text
// are untrusted; they stay inert text. JSON bodies render inside <pre> text
// nodes.

const IDENTITY_KEY = "reasonbraid-console-identity";

const state = { principal: "", tenant: "" };
const views = {
  threads: "Threads",
  thread: "Thread",
  events: "Timeline",
  audit: "Audit",
  budget: "Budget",
  presence: "Node presence",
  inbox: "Node inbox (admin)",
};

function $id(id) {
  return document.getElementById(id);
}

function el(tag, attrs, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs || {})) {
    if (value === null || value === undefined) continue;
    if (key === "class") node.className = value;
    else if (key.startsWith("on")) node.addEventListener(key.slice(2), value);
    else node.setAttribute(key, String(value));
  }
  for (const child of children) {
    if (child === null || child === undefined) continue;
    node.appendChild(
      typeof child === "string" ? document.createTextNode(child) : child,
    );
  }
  return node;
}

function saveIdentity() {
  state.principal = $id("principal").value.trim();
  state.tenant = $id("tenant").value.trim();
  localStorage.setItem(IDENTITY_KEY, JSON.stringify(state));
  $id("auth-status").textContent =
    state.principal && state.tenant ? "saved (this browser only)" : "";
}

function loadIdentity() {
  try {
    const saved = JSON.parse(localStorage.getItem(IDENTITY_KEY) || "null");
    if (saved && saved.principal && saved.tenant) {
      state.principal = saved.principal;
      state.tenant = saved.tenant;
      $id("principal").value = saved.principal;
      $id("tenant").value = saved.tenant;
    }
  } catch (_) {
    localStorage.removeItem(IDENTITY_KEY);
  }
}

// One GET over the existing surface, with the dev-profile principal header.
// The tenant travels in the query string, exactly as the CLI sends it.
async function api(path) {
  const response = await fetch(path, {
    headers: { "x-reasonbraid-principal": state.principal },
  });
  let body = null;
  try {
    body = await response.json();
  } catch (_) {
    body = null;
  }
  return { status: response.status, body };
}

function tenantQuery() {
  return "tenant_id=" + encodeURIComponent(state.tenant);
}

// ── Rendering helpers ──────────────────────────────────────────────────────────

function showError(status, body) {
  const message =
    body && body.message
      ? body.message
      : body && body.code
        ? body.code
        : "request failed";
  return el(
    "div",
    { class: "error card" },
    el("strong", null, `HTTP ${status}: `),
    String(message),
  );
}

function renderKeyValue(details, pairs) {
  for (const [label, value] of pairs) {
    if (value === null || value === undefined || value === "") continue;
    details.appendChild(
      el("div", { class: "kv" }, el("span", { class: "k" }, label), el("span", null, String(value))),
    );
  }
}

function jsonPre(value) {
  return el(
    "pre",
    { class: "json" },
    JSON.stringify(value, null, 2),
  );
}

function empty(text) {
  return el("p", { class: "muted" }, text);
}

function table(headers, rows) {
  return el(
    "table",
    null,
    el(
      "thead",
      null,
      el("tr", null, ...headers.map((h) => el("th", null, h))),
    ),
    el(
      "tbody",
      null,
      ...rows.map((cells) =>
        el("tr", null, ...cells.map((c) => el("td", null, c))),
      ),
    ),
  );
}

// ── Views ──────────────────────────────────────────────────────────────────────

async function viewThreads(out) {
  out.appendChild(el("h2", null, "Threads"));
  const { status, body } = await api("/v1/threads?" + tenantQuery());
  if (status !== 200) {
    out.appendChild(showError(status, body));
    return;
  }
  const threads = (body && body.threads) || [];
  if (threads.length === 0) {
    out.appendChild(empty("no threads in this tenant yet"));
    return;
  }
  out.appendChild(
    table(
      ["thread", "subject", "state"],
      threads.map((t) => [
        el(
          "button",
          { class: "link", onclick: () => openThread(t.thread_id) },
          t.thread_id,
        ),
        t.subject,
        t.state,
      ]),
    ),
  );
}

async function viewThread(out) {
  out.appendChild(el("h2", null, "Thread"));
  if (!state.threadId) {
    out.appendChild(empty("pick a thread from the Threads view"));
    return;
  }
  const path = "/v1/threads/" + encodeURIComponent(state.threadId) + "?" + tenantQuery();
  const { status, body } = await api(path);
  if (status !== 200) {
    out.appendChild(showError(status, body));
    return;
  }
  const st = body && body.state ? body.state : {};
  const details = el("div", { class: "kv-card" });
  renderKeyValue(details, [
    ["subject", st.subject],
    ["objective", st.objective],
    ["state", st.state],
    ["close reason", st.close_reason],
    ["cancel reason", st.cancel_reason],
    ["classification", st.classification],
    ["workflow profile", st.workflow_profile],
    ["current round", st.current_round],
    ["ceiling", st.ceiling_id],
    ["counters", st.contributions !== undefined
      ? `contributions=${st.contributions} revisions=${st.revisions} open_challenges=${st.open_challenges}`
      : null],
  ]);
  out.appendChild(details);
  if (st.participants && Object.keys(st.participants).length > 0) {
    out.appendChild(el("h3", null, "Participants"));
    out.appendChild(
      table(
        ["principal", "state"],
        Object.entries(st.participants).map(([id, s]) => [id, s]),
      ),
    );
  }
  if (st.invitations && Object.keys(st.invitations).length > 0) {
    out.appendChild(el("h3", null, "Invitations"));
    out.appendChild(
      table(
        ["role", "facts"],
        Object.entries(st.invitations).map(([id, meta]) => [id, jsonPre(meta)]),
      ),
    );
  }
  out.appendChild(el("h3", null, "Budget dimensions"));
  out.appendChild(jsonPre(st.budget));
}

async function viewEvents(out) {
  out.appendChild(el("h2", null, "Timeline (events)"));
  if (!state.threadId) {
    out.appendChild(empty("pick a thread first"));
    return;
  }
  const path =
    "/v1/threads/" + encodeURIComponent(state.threadId) + "/events?" + tenantQuery();
  const { status, body } = await api(path);
  if (status !== 200) {
    out.appendChild(showError(status, body));
    return;
  }
  const events = (body && body.events) || [];
  if (events.length === 0) {
    out.appendChild(empty("no events"));
    return;
  }
  out.appendChild(
    table(
      ["#", "event", "committed at", "body"],
      events.map((e) => [
        e.aggregate_version,
        e.event_type,
        e.committed_at,
        jsonPre(e.body),
      ]),
    ),
  );
}

async function viewAudit(out) {
  out.appendChild(el("h2", null, "Audit (authorization records)"));
  if (!state.threadId) {
    out.appendChild(empty("pick a thread first"));
    return;
  }
  const path =
    "/v1/threads/" + encodeURIComponent(state.threadId) + "/audit?" + tenantQuery();
  const { status, body } = await api(path);
  if (status !== 200) {
    out.appendChild(showError(status, body));
    return;
  }
  const records = (body && body.records) || [];
  if (records.length === 0) {
    out.appendChild(empty("no audit records"));
    return;
  }
  out.appendChild(
    table(
      ["record", "decision", "action", "actor", "policy digest"],
      records.map((r) => [
        r.record_id,
        r.decision,
        r.action,
        r.actor,
        r.policy_digest,
      ]),
    ),
  );
}

async function viewBudget(out) {
  out.appendChild(el("h2", null, "Budget (the `.1.6.1` read surface)"));
  if (!state.threadId) {
    out.appendChild(empty("pick a thread first"));
    return;
  }
  const path =
    "/v1/threads/" + encodeURIComponent(state.threadId) + "/budget?" + tenantQuery();
  const { status, body } = await api(path);
  if (status !== 200) {
    out.appendChild(showError(status, body));
    return;
  }
  const details = el("div", { class: "kv-card" });
  const ceiling = body.ceiling || {};
  renderKeyValue(details, [
    ["ceiling", ceiling.ceiling_id],
    ["policy version", ceiling.policy_version],
    ["created", ceiling.created_at],
  ]);
  out.appendChild(details);
  out.appendChild(el("h3", null, "Ceiling dimensions"));
  out.appendChild(jsonPre(ceiling.dimensions));
  const rows = body.reservations || [];
  out.appendChild(el("h3", null, `Reservations (${rows.length})`));
  if (rows.length === 0) {
    out.appendChild(empty("no reservation rows yet — no dispatch was attempted"));
    return;
  }
  out.appendChild(
    table(
      ["reservation", "status", "held", "usage", "reason", "settled"],
      rows.map((r) => [
        r.reservation_id,
        r.status,
        jsonPre(r.dimensions),
        r.usage ? jsonPre(r.usage) : "—",
        r.reason || "—",
        r.settled_at || "—",
      ]),
    ),
  );
}

async function viewPresence(out) {
  out.appendChild(el("h2", null, "Node presence"));
  const row = el("div", { class: "views-row" });
  const input = el("input", {
    type: "text",
    placeholder: "node id (nod_… or rol_…)",
  });
  row.appendChild(input);
  row.appendChild(
    el("button", {
      onclick: async () => {
        const nodeId = input.value.trim();
        if (!nodeId) return;
        const { status, body } = await api(
          "/v1/nodes/presence?node_id=" + encodeURIComponent(nodeId),
        );
        out.querySelectorAll(".presence-result").forEach((n) => n.remove());
        if (status === 200) {
          out.appendChild(
            el("div", { class: "presence-result" }, jsonPre(body)),
          );
        } else {
          out.appendChild(
            el("div", { class: "presence-result" }, showError(status, body)),
          );
        }
      },
    }, "Check presence"),
  );
  out.appendChild(row);
  out.appendChild(
    empty("presence is a read-only observability fact, derived from the lease clock"),
  );
}

async function viewInbox(out) {
  out.appendChild(el("h2", null, "Node inbox (tenant_admin)"));
  const row = el("div", { class: "views-row" });
  const input = el("input", {
    type: "text",
    placeholder: "node id (nod_… or rol_…)",
  });
  row.appendChild(input);
  row.appendChild(
    el("button", {
      onclick: async () => {
        const nodeId = input.value.trim();
        if (!nodeId) return;
        const { status, body } = await api(
          "/v1/nodes/inbox?node=" +
            encodeURIComponent(nodeId) +
            "&" +
            tenantQuery(),
        );
        out.querySelectorAll(".inbox-result").forEach((n) => n.remove());
        if (status === 200) {
          const rows = (body && body.rows) || [];
          out.appendChild(
            el(
              "div",
              { class: "inbox-result" },
              table(
                ["command", "state", "quarantine"],
                rows.map((r) => [
                  r.command_id,
                  r.state,
                  r.quarantined_at
                    ? `${r.quarantined_at} — ${r.quarantine_reason || ""}`
                    : "—",
                ]),
              ),
            ),
          );
        } else {
          out.appendChild(
            el("div", { class: "inbox-result" }, showError(status, body)),
          );
        }
      },
    }, "Inspect inbox"),
  );
  out.appendChild(row);
  out.appendChild(
    empty("a role without tenant_admin gets the typed 403 — the same gate the CLI shows"),
  );
}

const renderers = {
  threads: viewThreads,
  thread: viewThread,
  events: viewEvents,
  audit: viewAudit,
  budget: viewBudget,
  presence: viewPresence,
  inbox: viewInbox,
};

function openThread(threadId) {
  state.threadId = threadId;
  $id("sub-views").hidden = false;
  $id("current-thread").textContent = threadId;
  render("thread");
}

async function render(view) {
  const out = $id("output");
  out.replaceChildren();
  try {
    await renderers[view](out);
  } catch (error) {
    out.appendChild(showError(0, { message: `client error: ${error}` }));
  }
}

function wire() {
  loadIdentity();
  $id("auth-form").addEventListener("submit", (event) => {
    event.preventDefault();
    saveIdentity();
    $id("views").hidden = false;
    render("threads");
  });
  for (const button of document.querySelectorAll("button[data-view]")) {
    button.addEventListener("click", () => render(button.dataset.view));
  }
  if (state.principal && state.tenant) {
    $id("views").hidden = false;
    $id("auth-status").textContent =
      "using " + state.principal + " (tenant " + state.tenant + ")";
    render("threads");
  }
}

document.addEventListener("DOMContentLoaded", wire);
