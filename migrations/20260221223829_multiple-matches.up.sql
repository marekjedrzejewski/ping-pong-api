-- Forgot to add that initially, silly me.
ALTER TABLE game_state ADD PRIMARY KEY (id);

CREATE TABLE match(
    uid VARCHAR(6) PRIMARY KEY CHECK (uid ~ '^[a-z0-9]+$'),
    game_state_id BIGINT UNIQUE NOT NULL REFERENCES game_state (id)
);

-- If there was previous state, it will be first one in the table as that's what
-- code touches. Other ones might be there, but if that's the case, they must have been
-- created manually - we want to delete them.
DELETE FROM game_state
WHERE id NOT IN (
    SELECT id FROM game_state
    ORDER BY id ASC
    LIMIT 1
);

-- This creates 'og' match for it - if there was no state, nothing happens.
INSERT INTO match (uid, game_state_id)
    SELECT 'og', id
    FROM game_state;