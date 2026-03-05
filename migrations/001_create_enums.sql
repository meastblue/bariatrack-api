-- ============================================================
-- BariaTrack — 001: Enums
-- ============================================================

CREATE TYPE user_role AS ENUM ('patient', 'doctor', 'admin');

CREATE TYPE surgery_type AS ENUM (
  'sleeve_gastrectomy',
  'gastric_bypass_rygb',
  'gastric_band',
  'duodenal_switch',
  'mini_bypass_oagb',
  'other'
);

CREATE TYPE gender AS ENUM ('male', 'female', 'non_binary', 'prefer_not_to_say');

CREATE TYPE blood_type AS ENUM (
  'A_pos', 'A_neg', 'B_pos', 'B_neg',
  'AB_pos', 'AB_neg', 'O_pos', 'O_neg', 'unknown'
);

CREATE TYPE relation_status AS ENUM ('active', 'inactive', 'transferred');

CREATE TYPE entry_source AS ENUM ('manual', 'healthkit', 'doctor');

CREATE TYPE meal_type AS ENUM ('breakfast', 'lunch', 'dinner', 'snack', 'drink');

CREATE TYPE supplement_frequency AS ENUM ('daily', 'twice_daily', 'weekly', 'as_needed');

CREATE TYPE appointment_type AS ENUM (
  'consultation', 'follow_up', 'emergency',
  'teleconsultation', 'nutritionist', 'psychologist'
);

CREATE TYPE appointment_status AS ENUM (
  'scheduled', 'confirmed', 'completed',
  'cancelled', 'no_show', 'rescheduled'
);

CREATE TYPE document_type AS ENUM (
  'lab_result', 'scan_mri_ct', 'prescription',
  'surgery_report', 'consent_form', 'photo_progress',
  'nutrition_plan', 'other'
);

CREATE TYPE mood_value AS ENUM ('terrible', 'bad', 'neutral', 'good', 'awesome');

CREATE TYPE diet_phase_status AS ENUM ('active', 'completed', 'skipped');

CREATE TYPE notification_type AS ENUM (
  'appointment_reminder', 'appointment_confirmed', 'appointment_cancelled',
  'weight_goal_reached', 'hydration_reminder', 'supplement_reminder',
  'new_document', 'doctor_message', 'phase_change', 'system_alert'
);

CREATE TYPE task_category AS ENUM (
  'mental_health', 'physical', 'nutrition', 'education', 'breathing'
);

CREATE TYPE task_status AS ENUM ('pending', 'completed', 'skipped');

CREATE TYPE reminder_period AS ENUM ('morning', 'afternoon', 'evening');
