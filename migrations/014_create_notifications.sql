-- ============================================================
-- BariaTrack — 014: Notifications & Preferences
-- ============================================================

CREATE TABLE notifications (
  id                  UUID                PRIMARY KEY DEFAULT gen_random_uuid(),
  profile_id          UUID                NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  notification_type   notification_type   NOT NULL,
  title               VARCHAR(300)        NOT NULL,
  body                TEXT,
  is_read             BOOLEAN             NOT NULL DEFAULT false,
  related_entity_id   UUID,
  related_entity_type VARCHAR(50),
  scheduled_at        TIMESTAMPTZ,
  sent_at             TIMESTAMPTZ,
  idempotency_key     VARCHAR(255)        UNIQUE,
  created_at          TIMESTAMPTZ         NOT NULL DEFAULT now()
);

CREATE INDEX idx_notif_profile_id ON notifications (profile_id);
CREATE INDEX idx_notif_profile_read ON notifications (profile_id, is_read);
CREATE INDEX idx_notif_scheduled ON notifications (scheduled_at) WHERE sent_at IS NULL;

-- Préférences de rappel par patient
CREATE TABLE notification_preferences (
  id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id      UUID            NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  notif_type      notification_type NOT NULL,
  enabled         BOOLEAN         NOT NULL DEFAULT true,
  periods         reminder_period[] DEFAULT ARRAY['morning']::reminder_period[],
  created_at      TIMESTAMPTZ     NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ     NOT NULL DEFAULT now(),

  CONSTRAINT uq_notif_prefs_patient_type UNIQUE (patient_id, notif_type)
);

CREATE INDEX idx_notif_prefs_patient ON notification_preferences (patient_id);

COMMENT ON COLUMN notification_preferences.periods IS 'Moments de la journée pour les rappels: morning, afternoon, evening.';
