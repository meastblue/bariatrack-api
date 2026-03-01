CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE user_role AS ENUM ('patient', 'doctor', 'admin');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    supabase_uid UUID NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    role user_role NOT NULL DEFAULT 'patient',
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_users_supabase_uid ON users(supabase_uid);
CREATE INDEX idx_users_email ON users(email);
