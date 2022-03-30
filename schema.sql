---- Schema ----
CREATE DATABASE shereebot;

\c shereebot;
\set ON_ERROR_STOP true

DO $schema$ BEGIN 
    RAISE INFO 'Populating schema';

    RAISE INFO 'Creating tables';
    CREATE TABLE IF NOT EXISTS roles (
        id          UUID PRIMARY KEY,
        ae          TIMESTAMPTZ,
        editor      TIMESTAMPTZ,
        sound       TIMESTAMPTZ,
        color       TIMESTAMPTZ,
        updated_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS films (
        id              UUID PRIMARY KEY,
        roles_id        UUID REFERENCES roles,
        name            VARCHAR(255) NOT NULL UNIQUE,
        created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS students (
        id              UUID PRIMARY KEY,
        roles_id        UUID REFERENCES roles,
        email           VARCHAR(255) NOT NULL UNIQUE,
        created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );


    ---- Join tables ----

    CREATE TABLE IF NOT EXISTS students_films (
        student_id  UUID REFERENCES students,
        film_id     UUID REFERENCES films,
        role        VARCHAR(255),
        CONSTRAINT students_films_pk PRIMARY KEY (student_id, film_id)
    );


    ---- Indices ----
    RAISE INFO 'Creating indices';

    CREATE UNIQUE INDEX IF NOT EXISTS film_name_idx
        ON films(name);

    CREATE UNIQUE INDEX IF NOT EXISTS std_email_idx
        ON students(email);


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

    DROP TRIGGER IF EXISTS auto_update_roles_timestamp ON films;
    CREATE TRIGGER auto_update_roles_timestamp BEFORE UPDATE
        ON roles 
        FOR EACH ROW
        EXECUTE PROCEDURE update_timestamp();

    DROP TRIGGER IF EXISTS auto_update_students_timestamp ON films;
    CREATE TRIGGER auto_update_students_timestamp BEFORE UPDATE
        ON students 
        FOR EACH ROW
        EXECUTE PROCEDURE update_timestamp();
END $schema$;
