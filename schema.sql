---- Schema ----
CREATE DATABASE shereebot;

\c shereebot;
\set ON_ERROR_STOP true

DO $schema$ BEGIN 
    RAISE INFO 'Populating schema';

    RAISE INFO 'Creating tables';
    CREATE TABLE IF NOT EXISTS roles (
        id              UUID PRIMARY KEY,
        current         TEXT NOT NULL DEFAULT 'AE',
        -- The film/student who worked this role.
        ae              TEXT,
        editor          TEXT,
        sound           TEXT,
        finish          TEXT,
        updated_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS films (
        id              UUID PRIMARY KEY,
        roles_id        UUID REFERENCES roles,
        name            TEXT NOT NULL UNIQUE,
        priority        TEXT NOT NULL DEFAULT 'HIGH',
        group_number    INTEGER NOT NULL,
        created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS students (
        id              UUID PRIMARY KEY,
        name            TEXT NOT NULL DEFAULT '',
        roles_id        UUID REFERENCES roles,
        slack_id        TEXT NOT NULL DEFAULT '',
        current_film    TEXT,
        group_number    INTEGER NOT NULL DEFAULT 0,
        class           TEXT NOT NULL DEFAULT '0',
        created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS jobs_q (
        id                  UUID PRIMARY KEY,
        student_slack_id    TEXT NOT NULL,
        film_name           TEXT NOT NULL,
        role                TEXT NOT NULL,
        priority            TEXT DEFAULT 'High',
        msg_ts              TEXT, -- not relevant
        channel             TEXT, -- not relevant
        created_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS wait_q (
        id                  UUID PRIMARY KEY,
        student_slack_id    TEXT NOT NULL,
        film_name           TEXT NOT NULL,
        role                TEXT NOT NULL,
        priority            TEXT, -- not relevant
        msg_ts              TEXT,
        channel             TEXT,
        created_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );


    ---- Join tables ----

    CREATE TABLE IF NOT EXISTS students_films (
        student_id  UUID REFERENCES students,
        film_id     UUID REFERENCES films,
        role        TEXT,
        CONSTRAINT students_films_pk PRIMARY KEY (student_id, film_id)
    );


    ---- Indices ----
    RAISE INFO 'Creating indices';

    CREATE UNIQUE INDEX IF NOT EXISTS film_name_idx
        ON films(name);

    CREATE INDEX IF NOT EXISTS std_slack_id_idx
        ON students(slack_id);


    ---- Triggers ----
    RAISE INFO 'Creating triggers';

    CREATE OR REPLACE FUNCTION update_timestamp()
    RETURNS TRIGGER AS $$
    BEGIN
        NEW.updated_at = now();
        RETURN NEW;
    END;
    $$ language 'plpgsql';

    DROP TRIGGER IF EXISTS auto_update_films_timestamp ON films;
    CREATE TRIGGER auto_update_films_timestamp BEFORE UPDATE
        ON films 
        FOR EACH ROW
        EXECUTE PROCEDURE update_timestamp();

    DROP TRIGGER IF EXISTS auto_update_roles_timestamp ON roles;
    CREATE TRIGGER auto_update_roles_timestamp BEFORE UPDATE
        ON roles 
        FOR EACH ROW
        EXECUTE PROCEDURE update_timestamp();

    DROP TRIGGER IF EXISTS auto_update_students_timestamp ON students;
    CREATE TRIGGER auto_update_students_timestamp BEFORE UPDATE
        ON students 
        FOR EACH ROW
        EXECUTE PROCEDURE update_timestamp();
END $schema$;
