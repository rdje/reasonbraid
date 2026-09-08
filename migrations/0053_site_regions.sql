-- 0053_site_regions.sql — the regional-routing machinery (`PHASE-8.5.2`,
-- ADR-035 §20.10): the regions are DECLARED (the declaration is the
-- fail-closed seam — an undeclared region refuses at the routing
-- boundary) and the cross-region delivery rides the EXPLICIT pair
-- allowlist (a region pair without a row is the typed refusal — the
-- ADR-027 allowlist pattern applied to the region vocabulary).
--
-- The dev profile declares the single region `dev-local` with no pairs
-- (the single-site deployment; the `.5.3` store-and-forward consumes
-- this routing).

CREATE TABLE site_regions (
    region_id   TEXT        PRIMARY KEY,
    declared_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE region_pairs (
    from_region TEXT NOT NULL REFERENCES site_regions (region_id),
    to_region   TEXT NOT NULL REFERENCES site_regions (region_id),
    paired_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (from_region, to_region)
);

INSERT INTO site_regions (region_id) VALUES ('dev-local');
