-- ============================================================
-- BariaTrack — 004: Patients
-- ============================================================

CREATE TABLE patients (
  id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  profile_id          UUID         NOT NULL UNIQUE REFERENCES profiles(id) ON DELETE CASCADE,
  surgery_type        surgery_type NOT NULL,
  surgery_date        DATE         NOT NULL,
  initial_weight_kg   NUMERIC(5,2) NOT NULL,
  target_weight_kg    NUMERIC(5,2),
  height_cm           NUMERIC(5,1) NOT NULL,
  date_of_birth       DATE,
  gender              gender,
  blood_type          blood_type   DEFAULT 'unknown',
  primary_doctor_id   UUID         REFERENCES doctors(id) ON DELETE SET NULL,
  allergies           TEXT,
  notes               TEXT,
  created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
  updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),

  CONSTRAINT chk_patients_initial_weight CHECK (initial_weight_kg > 0),
  CONSTRAINT chk_patients_target_weight  CHECK (target_weight_kg IS NULL OR target_weight_kg > 0),
  CONSTRAINT chk_patients_height         CHECK (height_cm BETWEEN 100 AND 250)
);

CREATE INDEX idx_patients_profile_id ON patients (profile_id);
CREATE INDEX idx_patients_primary_doctor ON patients (primary_doctor_id);
CREATE INDEX idx_patients_surgery_date ON patients (surgery_date);

COMMENT ON COLUMN patients.height_cm IS 'Utilisée pour calculer le BMI côté applicatif — pas stocké en BDD.';
