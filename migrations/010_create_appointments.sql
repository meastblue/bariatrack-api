-- ============================================================
-- BariaTrack — 010: Appointments
-- ============================================================

CREATE TABLE appointments (
  id                  UUID                PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id          UUID                NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  doctor_id           UUID                REFERENCES doctors(id) ON DELETE SET NULL,
  appointment_type    appointment_type    NOT NULL,
  title               VARCHAR(200)        NOT NULL,
  scheduled_at        TIMESTAMPTZ         NOT NULL,
  duration_min        INTEGER             NOT NULL DEFAULT 30,
  location            VARCHAR(300),
  teleconsult_url     TEXT,
  status              appointment_status  NOT NULL DEFAULT 'scheduled',
  notes               TEXT,
  created_at          TIMESTAMPTZ         NOT NULL DEFAULT now(),
  updated_at          TIMESTAMPTZ         NOT NULL DEFAULT now(),

  CONSTRAINT chk_appointments_duration CHECK (duration_min > 0)
);

CREATE INDEX idx_apt_patient_id ON appointments (patient_id);
CREATE INDEX idx_apt_doctor_id ON appointments (doctor_id);
CREATE INDEX idx_apt_patient_date ON appointments (patient_id, scheduled_at DESC);
CREATE INDEX idx_apt_status ON appointments (status);
CREATE INDEX idx_apt_scheduled ON appointments (scheduled_at);
