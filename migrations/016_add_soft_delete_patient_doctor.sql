-- BariaTrack — 016: Soft delete sur patients et doctors
ALTER TABLE patients ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE doctors  ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
