-- ============================================================
-- BariaTrack — 002: Profiles (pont auth ↔ métier)
-- ============================================================

CREATE TABLE profiles (
  id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  auth_uid    UUID         NOT NULL UNIQUE,
  email           VARCHAR(320) NOT NULL,
  locale          VARCHAR(10)  NOT NULL DEFAULT 'fr',
  role            user_role    NOT NULL,
  created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
  deleted_at      TIMESTAMPTZ,

  CONSTRAINT uq_profiles_email UNIQUE (email)
);

CREATE INDEX idx_profiles_email ON profiles (email);
CREATE INDEX idx_profiles_role ON profiles (role);

COMMENT ON TABLE profiles IS 'Pont entre Supabase auth et la BDD métier. Infos utilisateur (nom, avatar) dans le JWT Supabase.';
COMMENT ON COLUMN profiles.deleted_at IS 'Soft delete RGPD.';
