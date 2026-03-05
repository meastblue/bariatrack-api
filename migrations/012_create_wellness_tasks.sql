-- ============================================================
-- BariaTrack — 012: Wellness tasks
-- ============================================================

-- Catalogue de tâches (défini par l'app / les médecins)
CREATE TABLE wellness_tasks (
  id              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
  title           VARCHAR(200)    NOT NULL,
  description     TEXT,
  category        task_category   NOT NULL,
  duration_min    INTEGER,
  is_active       BOOLEAN         NOT NULL DEFAULT true,
  created_at      TIMESTAMPTZ     NOT NULL DEFAULT now()
);

INSERT INTO wellness_tasks (title, description, category, duration_min) VALUES
  ('Test Extraversion/Introversion',  'Évalue ton profil de personnalité', 'mental_health', 10),
  ('Beck Anxiety Inventory',          'Questionnaire d anxiété de Beck', 'mental_health', 15),
  ('Exercices de respiration & yoga', 'Séance de relaxation guidée', 'breathing', 20),
  ('Lecture de développement',        '7 compétences de leadership', 'education', 30);

-- Tâches assignées aux patients (journalières)
CREATE TABLE patient_tasks (
  id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id      UUID         NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  task_id         UUID         NOT NULL REFERENCES wellness_tasks(id),
  assigned_by     UUID         REFERENCES doctors(id) ON DELETE SET NULL,
  status          task_status  NOT NULL DEFAULT 'pending',
  assigned_at     DATE         NOT NULL DEFAULT CURRENT_DATE,
  completed_at    TIMESTAMPTZ,
  note            TEXT,
  created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),

  CONSTRAINT uq_patient_task_day UNIQUE (patient_id, task_id, assigned_at)
);

CREATE INDEX idx_patient_tasks_patient_id ON patient_tasks (patient_id);
CREATE INDEX idx_patient_tasks_date ON patient_tasks (patient_id, assigned_at DESC);
CREATE INDEX idx_patient_tasks_status ON patient_tasks (patient_id, status);
