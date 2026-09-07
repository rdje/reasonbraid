-- The rule-based routing policy (`.5.2`, ADR-031): the deterministic table —
-- one arm per case class (the §13.8 rows). The arm names a REGISTERED
-- workflow profile. The resolution is a lookup; every resolution rides the
-- append-only audit table (the routing decision is never a silent default).
CREATE TABLE routing_rules (
    rule_id    TEXT    NOT NULL,
    case_class TEXT    NOT NULL,
    arm        TEXT    NOT NULL,
    built_in   BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (case_class)
);

-- The §13.8 rows as the built-in rules (ADR-031's table).
INSERT INTO routing_rules (rule_id, case_class, arm, built_in) VALUES
    ('rule_simple',      'simple',         'quick_advice',       true),
    ('rule_factual',     'factual',        'evidence_review',    true),
    ('rule_uncertain',   'uncertain',      'independent_panel',  true),
    ('rule_design',      'design_policy',  'critique',           true),
    ('rule_governed',    'governed',       'policy_proposal',    true),
    ('rule_correlated',  'correlated',     'independent_panel',  true),
    ('rule_diminishing', 'diminishing',    'quick_advice',       true);

-- The resolution audit (APPEND-ONLY): the class, the arm, the rule id, the
-- caller, and the surface (the `resolve` verb or the create boundary).
CREATE TABLE routing_resolutions (
    case_class  TEXT        NOT NULL,
    arm         TEXT        NOT NULL,
    rule_id     TEXT        NOT NULL,
    caller      TEXT        NOT NULL,
    surface     TEXT        NOT NULL,
    resolved_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
