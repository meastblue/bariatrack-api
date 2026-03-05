-- ============================================================
-- BariaTrack — 013: Insights (générés par l'app)
-- ============================================================

CREATE TABLE insights (
  id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id          UUID         NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  message             TEXT         NOT NULL,
  insight_type        VARCHAR(50)  NOT NULL,
  is_dismissed        BOOLEAN      NOT NULL DEFAULT false,
  generated_at        DATE         NOT NULL DEFAULT CURRENT_DATE,
  created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),

  CONSTRAINT uq_insights_patient_type_day UNIQUE (patient_id, insight_type, generated_at)
);

CREATE INDEX idx_insights_patient_date ON insights (patient_id, generated_at DESC);
CREATE INDEX idx_insights_dismissed ON insights (patient_id, is_dismissed);

COMMENT ON TABLE insights IS 'Messages générés automatiquement basés sur les données HealthKit + mood. Ex: "Tu sembles stressé ces derniers jours".';
COMMENT ON COLUMN insights.insight_type IS 'Ex: mood_drop, hydration_low, weight_plateau, sleep_deficit, activity_drop.';
