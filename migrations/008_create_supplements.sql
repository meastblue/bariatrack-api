-- ============================================================
-- BariaTrack — 008: Supplements & Supplement logs
-- ============================================================

CREATE TABLE supplements (
  id              UUID                  PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id      UUID                  NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  name            VARCHAR(150)          NOT NULL,
  dosage          VARCHAR(100),
  frequency       supplement_frequency  NOT NULL DEFAULT 'daily',
  prescribed_by   UUID                  REFERENCES doctors(id) ON DELETE SET NULL,
  is_active       BOOLEAN               NOT NULL DEFAULT true,
  started_at      DATE,
  ended_at        DATE,
  created_at      TIMESTAMPTZ           NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ           NOT NULL DEFAULT now()
);

CREATE INDEX idx_supplements_patient_id ON supplements (patient_id);
CREATE INDEX idx_supplements_active ON supplements (patient_id, is_active);

CREATE TABLE supplement_logs (
  id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  supplement_id   UUID        NOT NULL REFERENCES supplements(id) ON DELETE CASCADE,
  patient_id      UUID        NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  taken_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_suplog_supplement_id ON supplement_logs (supplement_id);
CREATE INDEX idx_suplog_patient_date ON supplement_logs (patient_id, taken_at DESC);
