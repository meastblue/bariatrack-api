-- ============================================================
-- BariaTrack — 009: Diet phases
-- ============================================================

CREATE TABLE diet_phases (
  id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  name            VARCHAR(100) NOT NULL,
  phase_order     INTEGER      NOT NULL,
  description     TEXT,
  duration_days   INTEGER,
  allowed_foods   TEXT,
  forbidden_foods TEXT,

  CONSTRAINT uq_diet_phases_order UNIQUE (phase_order)
);

INSERT INTO diet_phases (name, phase_order, description, duration_days, allowed_foods, forbidden_foods) VALUES
  ('Liquides clairs',           1, 'Première phase post-opératoire', 7,  'Eau, bouillons, jus clairs', 'Tout solide, sucreries'),
  ('Liquides épais',            2, 'Introduction de liquides consistants', 7, 'Lait, yaourt liquide, soupe mixée', 'Solides, fibres'),
  ('Purées',                    3, 'Aliments mixés finement', 14, 'Purées lisses, fromage blanc', 'Morceaux, aliments fibreux'),
  ('Aliments mous',             4, 'Texture molle et facile à mâcher', 30, 'Poisson, œufs, légumes cuits', 'Viandes dures, pain, crudités'),
  ('Alimentation normale adaptée', 5, 'Alimentation définitive post-bariatrique', NULL, 'Tout aliment bien mâché, portions réduites', 'Boissons sucrées, snacking, ultra-transformés');

CREATE TABLE patient_diet_phases (
  id              UUID              PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id      UUID              NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  phase_id        UUID              NOT NULL REFERENCES diet_phases(id),
  prescribed_by   UUID              REFERENCES doctors(id) ON DELETE SET NULL,
  started_at      DATE              NOT NULL,
  ended_at        DATE,
  status          diet_phase_status NOT NULL DEFAULT 'active',
  created_at      TIMESTAMPTZ       NOT NULL DEFAULT now()
);

CREATE INDEX idx_pdp_patient_id ON patient_diet_phases (patient_id);
CREATE INDEX idx_pdp_patient_status ON patient_diet_phases (patient_id, status);
