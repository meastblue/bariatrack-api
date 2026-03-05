-- ============================================================
-- BariaTrack — 005: Doctor-Patient relationships
-- ============================================================

CREATE TABLE doctor_patients (
  id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
  doctor_id       UUID            NOT NULL REFERENCES doctors(id) ON DELETE CASCADE,
  patient_id      UUID            NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  status          relation_status NOT NULL DEFAULT 'active',
  assigned_at     TIMESTAMPTZ     NOT NULL DEFAULT now(),
  ended_at        TIMESTAMPTZ,
  transfer_note   TEXT
);

CREATE INDEX idx_dp_doctor_id ON doctor_patients (doctor_id);
CREATE INDEX idx_dp_patient_id ON doctor_patients (patient_id);
CREATE INDEX idx_dp_status ON doctor_patients (status);

-- Un seul médecin actif par patient à la fois
CREATE UNIQUE INDEX uq_dp_active_doctor_patient
  ON doctor_patients (doctor_id, patient_id)
  WHERE status = 'active';

COMMENT ON TABLE doctor_patients IS 'Relation médecin-patient avec historique des transferts.';
