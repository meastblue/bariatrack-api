-- ============================================================
-- BariaTrack — 006: Daily health summaries (sync depuis HealthKit)
-- Agrégats journaliers — une ligne par patient par jour
-- ============================================================

-- Poids (une mesure par jour suffit)
CREATE TABLE weights (
  id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id      UUID         NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  weight_kg       NUMERIC(5,2) NOT NULL,
  measured_at     DATE         NOT NULL,
  source          entry_source NOT NULL DEFAULT 'healthkit',
  note            TEXT,
  created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),

  CONSTRAINT uq_weights_patient_day UNIQUE (patient_id, measured_at),
  CONSTRAINT chk_weights_weight CHECK (weight_kg BETWEEN 20 AND 500)
);

CREATE INDEX idx_weights_patient_date ON weights (patient_id, measured_at DESC);

-- Hydratation journalière (total du jour)
CREATE TABLE hydration_daily (
  id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id      UUID         NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  total_ml        INTEGER      NOT NULL,
  goal_ml         INTEGER      NOT NULL DEFAULT 1500,
  logged_at       DATE         NOT NULL,
  source          entry_source NOT NULL DEFAULT 'healthkit',
  created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),

  CONSTRAINT uq_hydration_patient_day UNIQUE (patient_id, logged_at),
  CONSTRAINT chk_hydration_total CHECK (total_ml >= 0)
);

CREATE INDEX idx_hydration_patient_date ON hydration_daily (patient_id, logged_at DESC);

-- Nutrition journalière (agrégat macros du jour)
CREATE TABLE nutrition_daily (
  id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id      UUID         NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  calories        INTEGER,
  protein_g       NUMERIC(6,2),
  carbs_g         NUMERIC(6,2),
  fat_g           NUMERIC(6,2),
  fiber_g         NUMERIC(6,2),
  goal_calories   INTEGER,
  logged_at       DATE         NOT NULL,
  source          entry_source NOT NULL DEFAULT 'healthkit',
  created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),

  CONSTRAINT uq_nutrition_patient_day UNIQUE (patient_id, logged_at)
);

CREATE INDEX idx_nutrition_patient_date ON nutrition_daily (patient_id, logged_at DESC);

-- Activité journalière
CREATE TABLE activity_daily (
  id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id          UUID         NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  steps               INTEGER,
  active_calories     INTEGER,
  training_min        INTEGER,
  goal_steps          INTEGER,
  goal_active_cal     INTEGER,
  logged_at           DATE         NOT NULL,
  source              entry_source NOT NULL DEFAULT 'healthkit',
  created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),

  CONSTRAINT uq_activity_patient_day UNIQUE (patient_id, logged_at)
);

CREATE INDEX idx_activity_patient_date ON activity_daily (patient_id, logged_at DESC);

-- Sommeil journalier
CREATE TABLE sleep_daily (
  id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id          UUID         NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  duration_min        INTEGER      NOT NULL,
  deep_min            INTEGER,
  rem_min             INTEGER,
  light_min           INTEGER,
  awake_min           INTEGER,
  sleep_start         TIMESTAMPTZ,
  sleep_end           TIMESTAMPTZ,
  logged_at           DATE         NOT NULL,
  source              entry_source NOT NULL DEFAULT 'healthkit',
  created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),

  CONSTRAINT uq_sleep_patient_day UNIQUE (patient_id, logged_at),
  CONSTRAINT chk_sleep_duration CHECK (duration_min > 0)
);

CREATE INDEX idx_sleep_patient_date ON sleep_daily (patient_id, logged_at DESC);

-- Fréquence cardiaque journalière (moyenne/min/max)
CREATE TABLE heart_rate_daily (
  id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id      UUID         NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  avg_bpm         INTEGER      NOT NULL,
  min_bpm         INTEGER,
  max_bpm         INTEGER,
  resting_bpm     INTEGER,
  logged_at       DATE         NOT NULL,
  source          entry_source NOT NULL DEFAULT 'healthkit',
  created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),

  CONSTRAINT uq_hr_patient_day UNIQUE (patient_id, logged_at),
  CONSTRAINT chk_hr_avg CHECK (avg_bpm BETWEEN 30 AND 250)
);

CREATE INDEX idx_hr_patient_date ON heart_rate_daily (patient_id, logged_at DESC);

COMMENT ON TABLE weights IS 'Sync HealthKit — une pesée par jour par patient.';
COMMENT ON TABLE hydration_daily IS 'Sync HealthKit — total hydratation journalière.';
COMMENT ON TABLE nutrition_daily IS 'Sync HealthKit — agrégat nutritionnel journalier.';
COMMENT ON TABLE activity_daily IS 'Sync HealthKit — activité physique journalière.';
COMMENT ON TABLE sleep_daily IS 'Sync HealthKit — analyse du sommeil par nuit.';
COMMENT ON TABLE heart_rate_daily IS 'Sync HealthKit — fréquence cardiaque journalière.';
