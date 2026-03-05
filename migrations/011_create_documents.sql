-- ============================================================
-- BariaTrack — 011: Documents
-- ============================================================

CREATE TABLE documents (
  id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id          UUID            NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  uploaded_by         UUID            NOT NULL REFERENCES profiles(id),
  appointment_id      UUID            REFERENCES appointments(id) ON DELETE SET NULL,
  title               VARCHAR(300)    NOT NULL,
  description         TEXT,
  file_url            TEXT            NOT NULL,
  file_type           document_type   NOT NULL,
  mime_type           VARCHAR(100),
  file_size_bytes     INTEGER,
  uploaded_at         TIMESTAMPTZ     NOT NULL DEFAULT now()
);

CREATE INDEX idx_docs_patient_id ON documents (patient_id);
CREATE INDEX idx_docs_appointment_id ON documents (appointment_id);
CREATE INDEX idx_docs_uploaded_by ON documents (uploaded_by);
CREATE INDEX idx_docs_file_type ON documents (patient_id, file_type);

COMMENT ON COLUMN documents.file_url IS 'URL Supabase Storage.';
COMMENT ON COLUMN documents.uploaded_by IS 'Patient ou médecin.';
