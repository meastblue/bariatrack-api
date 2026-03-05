-- ============================================================
-- BariaTrack — 003: Doctors
-- ============================================================

CREATE TABLE doctors (
  id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  profile_id          UUID         NOT NULL UNIQUE REFERENCES profiles(id) ON DELETE CASCADE,
  specialty           VARCHAR(100) NOT NULL DEFAULT 'chirurgie_bariatrique',
  license_number      VARCHAR(50)  UNIQUE,
  hospital_name       VARCHAR(200),
  phone_professional  VARCHAR(20),
  timezone            VARCHAR(50)  NOT NULL DEFAULT 'Europe/Paris',
  is_verified         BOOLEAN      NOT NULL DEFAULT false,
  created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
  updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_doctors_profile_id ON doctors (profile_id);
CREATE INDEX idx_doctors_license ON doctors (license_number);
CREATE INDEX idx_doctors_verified ON doctors (is_verified);

COMMENT ON COLUMN doctors.license_number IS 'Numéro RPPS France.';
COMMENT ON COLUMN doctors.is_verified IS 'Validé par admin avant accès aux dossiers patients.';
