-- ============================================================
-- BariaTrack — 007: Moods
-- ============================================================

CREATE TABLE moods (
  id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id      UUID        NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  mood            mood_value  NOT NULL,
  note            TEXT,
  logged_at       DATE        NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

  CONSTRAINT uq_moods_patient_day UNIQUE (patient_id, logged_at)
);

CREATE INDEX idx_moods_patient_date ON moods (patient_id, logged_at DESC);

COMMENT ON TABLE moods IS '1 entrée max par patient par jour — saisie manuelle uniquement.';
